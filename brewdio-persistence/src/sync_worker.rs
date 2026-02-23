#![cfg(feature = "native")]

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::db;
use crate::protocol::{DocType, SyncMessage};
use crate::sync::SyncSession;

const PEER_ID: &str = "server";
const DIRTY_CHECK_INTERVAL: Duration = Duration::from_secs(2);
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

/// Persistent sync sessions keyed by (DocType, doc_id), kept alive for the connection.
type Sessions = HashMap<(DocType, String), SyncSession>;

/// Spawn a background sync worker that connects to the given WebSocket server URL.
/// Returns a `JoinHandle` for the spawned task.
///
/// The `connected` flag is set to `true` when the WebSocket connection is established
/// and the Hello handshake completes, and `false` when disconnected or on error.
///
/// The `Connection` is wrapped in `Arc<Mutex<>>` since rusqlite is sync-only.
pub fn spawn_sync(
    conn: Arc<Mutex<rusqlite::Connection>>,
    server_url: String,
    connected: Arc<AtomicBool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match connect_and_sync(&conn, &server_url, &connected).await {
                Ok(()) => {}
                Err(_) => {}
            }
            connected.store(false, Ordering::Relaxed);
            sleep(RECONNECT_DELAY).await;
        }
    })
}

async fn connect_and_sync(
    conn: &Arc<Mutex<rusqlite::Connection>>,
    server_url: &str,
    connected: &Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (ws_stream, _) = connect_async(server_url).await?;
    let (mut ws_tx, mut ws_rx) = ws_stream.split();
    let mut sessions: Sessions = HashMap::new();

    // Clear stale sync states from any previous connection
    {
        let c = conn.lock().unwrap_or_else(|e| e.into_inner());
        db::clear_sync_states_for_peer(&*c, PEER_ID)?;
    }

    // --- Hello handshake ---
    let (local_recipe_ids, local_batch_ids, local_settings_ids) = {
        let c = conn.lock().unwrap_or_else(|e| e.into_inner());
        (
            db::list_all_doc_ids(&*c, DocType::Recipe)?,
            db::list_all_doc_ids(&*c, DocType::Batch)?,
            db::list_all_doc_ids(&*c, DocType::Settings)?,
        )
    };
    let hello = SyncMessage::Hello {
        recipe_ids: local_recipe_ids.clone(),
        batch_ids: local_batch_ids.clone(),
        settings_ids: local_settings_ids.clone(),
    };
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

    let (server_recipe_ids, server_batch_ids, server_settings_ids) = match server_hello {
        SyncMessage::Hello {
            recipe_ids,
            batch_ids,
            settings_ids,
        } => (
            recipe_ids.into_iter().collect::<HashSet<_>>(),
            batch_ids.into_iter().collect::<HashSet<_>>(),
            settings_ids.into_iter().collect::<HashSet<_>>(),
        ),
        _ => return Err("Expected Hello from server".into()),
    };

    // Handshake complete — mark as connected
    connected.store(true, Ordering::Relaxed);

    // Send NewDoc for docs only we have
    let local_sets: [(DocType, HashSet<String>, &HashSet<String>); 3] = [
        (DocType::Recipe, local_recipe_ids.into_iter().collect(), &server_recipe_ids),
        (DocType::Batch, local_batch_ids.into_iter().collect(), &server_batch_ids),
        (DocType::Settings, local_settings_ids.into_iter().collect(), &server_settings_ids),
    ];

    for (doc_type, local_ids, server_ids) in &local_sets {
        for id in local_ids.difference(server_ids) {
            let am_data = {
                let c = conn.lock().unwrap_or_else(|e| e.into_inner());
                db::get_doc_am_data(&*c, *doc_type, id)?
            };
            if let Some(am_data) = am_data {
                let msg = SyncMessage::NewDoc {
                    doc_type: *doc_type,
                    doc_id: id.clone(),
                    am_data,
                };
                ws_tx
                    .send(Message::Text(serde_json::to_string(&msg)?.into()))
                    .await?;
                let c = conn.lock().unwrap_or_else(|e| e.into_inner());
                db::clear_dirty_doc(&*c, *doc_type, id)?;
            }
        }
    }

    // Server initiates sync for shared docs; client just responds.
    // --- Steady-state loop ---
    let mut dirty_interval = tokio::time::interval(DIRTY_CHECK_INTERVAL);

    loop {
        tokio::select! {
            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_incoming(conn, &mut sessions, &mut ws_tx, &text).await?;
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        return Ok(());
                    }
                    Some(Err(e)) => return Err(e.into()),
                    _ => {}
                }
            }
            _ = dirty_interval.tick() => {
                let dirty = {
                    let c = conn.lock().unwrap_or_else(|e| e.into_inner());
                    db::list_dirty_docs(&*c)?
                };
                for (doc_type, id) in &dirty {
                    send_sync_for_doc(conn, &mut sessions, &mut ws_tx, *doc_type, id).await?;
                }
            }
        }
    }
}

/// Get or create a persistent SyncSession for a document.
fn get_or_create_session<'a>(
    conn: &Arc<Mutex<rusqlite::Connection>>,
    sessions: &'a mut Sessions,
    doc_type: DocType,
    doc_id: &str,
) -> Option<&'a mut SyncSession> {
    let key = (doc_type, doc_id.to_string());
    if !sessions.contains_key(&key) {
        let am_data = {
            let c = conn.lock().unwrap_or_else(|e| e.into_inner());
            db::get_doc_am_data(&*c, doc_type, doc_id).ok().flatten()
        };
        if let Some(am_data) = am_data {
            sessions.insert(key.clone(), SyncSession::from_doc_bytes(&am_data));
        } else {
            return None;
        }
    }
    sessions.get_mut(&key)
}

/// Generate and send a sync message for a single document using a persistent session.
async fn send_sync_for_doc<S>(
    conn: &Arc<Mutex<rusqlite::Connection>>,
    sessions: &mut Sessions,
    ws_tx: &mut S,
    doc_type: DocType,
    doc_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    S: SinkExt<Message> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    let session = match get_or_create_session(conn, sessions, doc_type, doc_id) {
        Some(s) => s,
        None => return Ok(()),
    };

    // Merge latest DB state into the session (picks up local edits)
    let am_data = {
        let c = conn.lock().unwrap_or_else(|e| e.into_inner());
        db::get_doc_am_data(&*c, doc_type, doc_id)?
    };
    if let Some(am_data) = am_data {
        session.merge_doc(&am_data);
    }

    if let Some(sync_msg_bytes) = session.generate_sync_message() {
        let msg = SyncMessage::SyncDoc {
            doc_type,
            doc_id: doc_id.to_string(),
            data: sync_msg_bytes,
        };
        ws_tx
            .send(Message::Text(serde_json::to_string(&msg)?.into()))
            .await?;
    }

    {
        let c = conn.lock().unwrap_or_else(|e| e.into_inner());
        db::clear_dirty_doc(&*c, doc_type, doc_id)?;
    }

    Ok(())
}

/// Handle an incoming WebSocket message from the server using persistent sessions.
async fn handle_incoming<S>(
    conn: &Arc<Mutex<rusqlite::Connection>>,
    sessions: &mut Sessions,
    ws_tx: &mut S,
    text: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    S: SinkExt<Message> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    let msg: SyncMessage = serde_json::from_str(text)?;

    match msg {
        SyncMessage::NewDoc {
            doc_type,
            doc_id,
            am_data,
        } => {
            let c = conn.lock().unwrap_or_else(|e| e.into_inner());
            db::apply_remote_merge_doc(&*c, doc_type, &doc_id, &am_data)?;
            sessions.insert(
                (doc_type, doc_id),
                SyncSession::from_doc_bytes(&am_data),
            );
        }
        SyncMessage::SyncDoc {
            doc_type,
            doc_id,
            data,
        } => {
            let session = match get_or_create_session(conn, sessions, doc_type, &doc_id) {
                Some(s) => s,
                None => return Ok(()),
            };

            let reply = session.receive_sync_message(&data)?;

            if let Some(reply_bytes) = &reply {
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
            let c = conn.lock().unwrap_or_else(|e| e.into_inner());
            db::apply_remote_merge_doc(&*c, doc_type, &doc_id, &saved)?;
            if reply.is_none() {
                db::clear_dirty_doc(&*c, doc_type, &doc_id)?;
            }
        }
        SyncMessage::Hello { .. } => {
            // Unexpected mid-session Hello, ignore
        }
    }

    Ok(())
}
