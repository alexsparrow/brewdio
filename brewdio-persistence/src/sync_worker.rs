use std::collections::{HashMap, HashSet};
use std::fmt;
use std::time::Duration;

use futures_util::stream::SplitSink;
use futures_util::{pin_mut, SinkExt, StreamExt};
use futures_util::future::{select, Either};
use tokio_tungstenite_wasm::{connect, Message, WebSocketStream};

#[cfg(feature = "native")]
use tokio::time::{interval, sleep};
#[cfg(feature = "wasm")]
use wasmtimer::tokio::interval;

use crate::connection::Connection;
use crate::db;
use crate::protocol::{DocType, SyncMessage};
use crate::sync::SyncSession;

const PEER_ID: &str = "server";
const DIRTY_CHECK_INTERVAL: Duration = Duration::from_secs(2);
#[cfg(feature = "native")]
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

/// Persistent sync sessions keyed by (DocType, doc_id), kept alive for the connection.
type Sessions = HashMap<(DocType, String), SyncSession>;

#[derive(Debug)]
pub enum SyncError {
    WebSocket(tokio_tungstenite_wasm::Error),
    Json(serde_json::Error),
    Db(crate::connection::DbError),
    Protocol(String),
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncError::WebSocket(e) => write!(f, "WebSocket error: {}", e),
            SyncError::Json(e) => write!(f, "JSON error: {}", e),
            SyncError::Db(e) => write!(f, "DB error: {}", e),
            SyncError::Protocol(e) => write!(f, "Protocol error: {}", e),
        }
    }
}

impl From<tokio_tungstenite_wasm::Error> for SyncError {
    fn from(e: tokio_tungstenite_wasm::Error) -> Self {
        SyncError::WebSocket(e)
    }
}

impl From<serde_json::Error> for SyncError {
    fn from(e: serde_json::Error) -> Self {
        SyncError::Json(e)
    }
}

impl From<crate::connection::DbError> for SyncError {
    fn from(e: crate::connection::DbError) -> Self {
        SyncError::Db(e)
    }
}

/// Run the sync protocol against a WebSocket server. Returns when the connection
/// drops or an unrecoverable error occurs. The caller should reconnect after a delay.
///
/// - `conn`: any `Connection + Sync` — `&Mutex<rusqlite::Connection>` on native,
///   `&WasmConnection` on WASM (which is `Sync` because WASM is single-threaded).
/// - `on_status(connected)`: called when connection status changes.
/// - `on_change(doc_type)`: called when a remote change is applied to the DB.
pub async fn run_sync(
    conn: &(impl Connection + Sync),
    server_url: &str,
    on_status: impl Fn(bool),
    on_change: impl Fn(DocType),
) -> Result<(), SyncError> {
    log::info!("[sync] connecting to {}", server_url);
    let ws_stream = connect(server_url).await?;
    let (mut ws_tx, mut ws_rx) = ws_stream.split();
    let mut sessions: Sessions = HashMap::new();

    // Clear stale sync states from any previous connection
    db::clear_sync_states_for_peer(conn, PEER_ID)?;

    // --- Hello handshake ---
    let local_recipe_ids = db::list_all_doc_ids(conn, DocType::Recipe)?;
    let local_batch_ids = db::list_all_doc_ids(conn, DocType::Batch)?;
    let local_settings_ids = db::list_all_doc_ids(conn, DocType::Settings)?;
    let local_equipment_profile_ids = db::list_all_doc_ids(conn, DocType::EquipmentProfile)?;

    let hello = SyncMessage::Hello {
        recipe_ids: local_recipe_ids.clone(),
        batch_ids: local_batch_ids.clone(),
        settings_ids: local_settings_ids.clone(),
        equipment_profile_ids: local_equipment_profile_ids.clone(),
    };
    log::debug!(
        "[sync] sending Hello (recipes={}, batches={}, settings={}, equipment={})",
        local_recipe_ids.len(),
        local_batch_ids.len(),
        local_settings_ids.len(),
        local_equipment_profile_ids.len(),
    );
    ws_tx
        .send(Message::Text(serde_json::to_string(&hello)?.into()))
        .await?;

    // Wait for server Hello
    let server_hello = loop {
        match ws_rx.next().await {
            Some(Ok(Message::Text(text))) => {
                let msg: SyncMessage = serde_json::from_str(&text)?;
                break msg;
            }
            Some(Ok(_)) => continue,
            Some(Err(e)) => return Err(e.into()),
            None => return Ok(()),
        }
    };

    let (server_recipe_ids, server_batch_ids, server_settings_ids, server_equipment_profile_ids) =
        match server_hello {
            SyncMessage::Hello {
                recipe_ids,
                batch_ids,
                settings_ids,
                equipment_profile_ids,
            } => (
                recipe_ids.into_iter().collect::<HashSet<_>>(),
                batch_ids.into_iter().collect::<HashSet<_>>(),
                settings_ids.into_iter().collect::<HashSet<_>>(),
                equipment_profile_ids.into_iter().collect::<HashSet<_>>(),
            ),
            _ => return Err(SyncError::Protocol("Expected Hello from server".into())),
        };

    // Handshake complete — mark as connected
    log::info!("[sync] connected, handshake complete");
    on_status(true);

    // Send NewDoc for docs only we have
    let local_sets: [(DocType, HashSet<String>, &HashSet<String>); 4] = [
        (
            DocType::Recipe,
            local_recipe_ids.into_iter().collect(),
            &server_recipe_ids,
        ),
        (
            DocType::Batch,
            local_batch_ids.into_iter().collect(),
            &server_batch_ids,
        ),
        (
            DocType::Settings,
            local_settings_ids.into_iter().collect(),
            &server_settings_ids,
        ),
        (
            DocType::EquipmentProfile,
            local_equipment_profile_ids.into_iter().collect(),
            &server_equipment_profile_ids,
        ),
    ];

    for (doc_type, local_ids, server_ids) in &local_sets {
        for id in local_ids.difference(server_ids) {
            let am_data = db::get_doc_am_data(conn, *doc_type, id)?;
            if let Some(am_data) = am_data {
                log::debug!("[sync] sending NewDoc {:?}/{}", doc_type, id);
                let msg = SyncMessage::NewDoc {
                    doc_type: *doc_type,
                    doc_id: id.clone(),
                    am_data,
                };
                ws_tx
                    .send(Message::Text(serde_json::to_string(&msg)?.into()))
                    .await?;
                db::clear_dirty_doc(conn, *doc_type, id)?;
            }
        }
    }

    // --- Steady-state loop ---
    let mut dirty_interval = interval(DIRTY_CHECK_INTERVAL);

    loop {
        let ws_future = ws_rx.next();
        let tick_future = dirty_interval.tick();
        pin_mut!(ws_future, tick_future);

        match select(ws_future, tick_future).await {
            Either::Left((msg, _)) => match msg {
                Some(Ok(Message::Text(text))) => {
                    handle_incoming(conn, &mut sessions, &mut ws_tx, &text, &on_change).await?;
                }
                Some(Ok(Message::Close(_))) | None => {
                    log::info!("[sync] connection closed");
                    on_status(false);
                    return Ok(());
                }
                Some(Err(e)) => {
                    log::info!("[sync] connection error: {}", e);
                    on_status(false);
                    return Err(e.into());
                }
                _ => {}
            },
            Either::Right((_, _)) => {
                let dirty = db::list_dirty_docs(conn)?;
                if !dirty.is_empty() {
                    log::debug!("[sync] dirty check: {} docs", dirty.len());
                }
                for (doc_type, id) in &dirty {
                    send_sync_for_doc(conn, &mut sessions, &mut ws_tx, *doc_type, id).await?;
                }
            }
        }
    }
}

/// Get or create a persistent SyncSession for a document.
fn get_or_create_session<'a>(
    conn: &(impl Connection + Sync),
    sessions: &'a mut Sessions,
    doc_type: DocType,
    doc_id: &str,
) -> Option<&'a mut SyncSession> {
    let key = (doc_type, doc_id.to_string());
    if !sessions.contains_key(&key) {
        let am_data = db::get_doc_am_data(conn, doc_type, doc_id).ok().flatten();
        if let Some(am_data) = am_data {
            sessions.insert(key.clone(), SyncSession::from_doc_bytes(&am_data));
        } else {
            return None;
        }
    }
    sessions.get_mut(&key)
}

/// Generate and send a sync message for a single document using a persistent session.
async fn send_sync_for_doc(
    conn: &(impl Connection + Sync),
    sessions: &mut Sessions,
    ws_tx: &mut SplitSink<WebSocketStream, Message>,
    doc_type: DocType,
    doc_id: &str,
) -> Result<(), SyncError> {
    let session = match get_or_create_session(conn, sessions, doc_type, doc_id) {
        Some(s) => s,
        None => return Ok(()),
    };

    // Merge latest DB state into the session (picks up local edits)
    let am_data = db::get_doc_am_data(conn, doc_type, doc_id)?;
    if let Some(am_data) = am_data {
        session.merge_doc(&am_data);
    }

    // Log the session state when sending
    if doc_type == DocType::Recipe {
        let snap = session.save_doc();
        match crate::automerge::hydrate_from_automerge::<crate::recipe::RecipeDocument>(&snap) {
            Ok(doc) => log::info!(
                "[sync] send_sync_for_doc {:?}/{}: batch_size.value={}",
                doc_type, doc_id, doc.recipe.batch_size.value
            ),
            Err(e) => log::warn!(
                "[sync] send_sync_for_doc {:?}/{}: hydration FAILED: {:?}",
                doc_type, doc_id, e
            ),
        }
        // Restore session state — save_doc was only for logging
        session.merge_doc(&snap);
    }

    if let Some(sync_msg_bytes) = session.generate_sync_message() {
        log::debug!("[sync] sending SyncDoc {:?}/{}", doc_type, doc_id);
        let msg = SyncMessage::SyncDoc {
            doc_type,
            doc_id: doc_id.to_string(),
            data: sync_msg_bytes,
        };
        ws_tx
            .send(Message::Text(serde_json::to_string(&msg)?.into()))
            .await?;
    } else {
        log::debug!("[sync] send_sync_for_doc {:?}/{}: already synced, nothing to send", doc_type, doc_id);
    }

    db::clear_dirty_doc(conn, doc_type, doc_id)?;

    Ok(())
}

/// Handle an incoming WebSocket message from the server using persistent sessions.
async fn handle_incoming(
    conn: &(impl Connection + Sync),
    sessions: &mut Sessions,
    ws_tx: &mut SplitSink<WebSocketStream, Message>,
    text: &str,
    on_change: &impl Fn(DocType),
) -> Result<(), SyncError> {
    let msg: SyncMessage = serde_json::from_str(text)?;

    match msg {
        SyncMessage::NewDoc {
            doc_type,
            doc_id,
            am_data,
        } => {
            log::debug!("[sync] received NewDoc {:?}/{}", doc_type, doc_id);
            db::apply_remote_merge_doc(conn, doc_type, &doc_id, &am_data)?;
            sessions.insert(
                (doc_type, doc_id),
                SyncSession::from_doc_bytes(&am_data),
            );
            on_change(doc_type);
        }
        SyncMessage::SyncDoc {
            doc_type,
            doc_id,
            data,
        } => {
            log::debug!(
                "[sync] received SyncDoc {:?}/{} ({} bytes)",
                doc_type,
                doc_id,
                data.len()
            );
            let session = match get_or_create_session(conn, sessions, doc_type, &doc_id) {
                Some(s) => s,
                None => return Ok(()),
            };

            let reply = session.receive_sync_message(&data).map_err(|e| {
                SyncError::Protocol(format!("Sync error: {}", e))
            })?;

            if let Some(ref reply_bytes) = reply {
                let reply_msg = SyncMessage::SyncDoc {
                    doc_type,
                    doc_id: doc_id.clone(),
                    data: reply_bytes.clone(),
                };
                ws_tx
                    .send(Message::Text(serde_json::to_string(&reply_msg)?.into()))
                    .await?;
            }

            // Save the merged doc to DB
            let saved = session.save_doc();

            // Log the hydrated state of the merged session doc
            if doc_type == DocType::Recipe {
                match crate::automerge::hydrate_from_automerge::<crate::recipe::RecipeDocument>(&saved) {
                    Ok(doc) => log::info!(
                        "[sync] handle_incoming SyncDoc {:?}/{}: session hydrates to batch_size.value={}, reply={}",
                        doc_type, doc_id, doc.recipe.batch_size.value, reply.is_some()
                    ),
                    Err(e) => log::warn!(
                        "[sync] handle_incoming SyncDoc {:?}/{}: session hydration FAILED: {:?}",
                        doc_type, doc_id, e
                    ),
                }
            }

            db::apply_remote_merge_doc(conn, doc_type, &doc_id, &saved)?;
            if reply.is_none() {
                db::clear_dirty_doc(conn, doc_type, &doc_id)?;
            }
            on_change(doc_type);
        }
        SyncMessage::Hello { .. } => {
            // Unexpected mid-session Hello, ignore
        }
    }

    Ok(())
}

/// Spawn a background sync worker (native only).
/// Connects to the given WebSocket server URL with automatic reconnection.
#[cfg(feature = "native")]
pub fn spawn_sync(
    conn: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
    server_url: String,
    connected: std::sync::Arc<std::sync::atomic::AtomicBool>,
    changed: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let _ = run_sync(
                &*conn,
                &server_url,
                |c| connected.store(c, std::sync::atomic::Ordering::Relaxed),
                |_| changed.store(true, std::sync::atomic::Ordering::Relaxed),
            )
            .await;
            connected.store(false, std::sync::atomic::Ordering::Relaxed);
            sleep(RECONNECT_DELAY).await;
        }
    })
}
