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
use crate::protocol::SyncMessage;
use crate::sync::SyncSession;

const PEER_ID: &str = "server";
const DIRTY_CHECK_INTERVAL: Duration = Duration::from_secs(2);
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

/// Persistent sync sessions keyed by recipe ID, kept alive for the connection.
type Sessions = HashMap<String, SyncSession>;

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
    let local_ids = {
        let c = conn.lock().unwrap_or_else(|e| e.into_inner());
        db::list_all_recipe_ids(&*c)?
    };
    let hello = SyncMessage::Hello {
        recipe_ids: local_ids.clone(),
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

    let server_ids: HashSet<String> = match server_hello {
        SyncMessage::Hello { recipe_ids } => recipe_ids.into_iter().collect(),
        _ => return Err("Expected Hello from server".into()),
    };
    let local_id_set: HashSet<String> = local_ids.into_iter().collect();

    // Handshake complete — mark as connected
    connected.store(true, Ordering::Relaxed);

    // Send NewDoc for recipes only we have
    for id in local_id_set.difference(&server_ids) {
        let row = {
            let c = conn.lock().unwrap_or_else(|e| e.into_inner());
            db::get_recipe(&*c, id)?
        };
        if let Some(row) = row {
            let msg = SyncMessage::NewDoc {
                recipe_id: id.clone(),
                am_data: row.am_data,
            };
            ws_tx
                .send(Message::Text(serde_json::to_string(&msg)?.into()))
                .await?;
            let c = conn.lock().unwrap_or_else(|e| e.into_inner());
            db::clear_dirty(&*c, id)?;
        }
    }

    // Server initiates sync for shared recipes; client just responds.
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
                    db::list_dirty_recipes(&*c)?
                };
                for row in &dirty {
                    send_sync_for_recipe(conn, &mut sessions, &mut ws_tx, &row.id).await?;
                }
            }
        }
    }
}

/// Get or create a persistent SyncSession for a recipe.
fn get_or_create_session<'a>(
    conn: &Arc<Mutex<rusqlite::Connection>>,
    sessions: &'a mut Sessions,
    recipe_id: &str,
) -> Option<&'a mut SyncSession> {
    if !sessions.contains_key(recipe_id) {
        let row = {
            let c = conn.lock().unwrap_or_else(|e| e.into_inner());
            db::get_recipe(&*c, recipe_id).ok().flatten()
        };
        if let Some(row) = row {
            sessions.insert(
                recipe_id.to_string(),
                SyncSession::from_doc_bytes(&row.am_data),
            );
        } else {
            return None;
        }
    }
    sessions.get_mut(recipe_id)
}

/// Generate and send a sync message for a single recipe using a persistent session.
async fn send_sync_for_recipe<S>(
    conn: &Arc<Mutex<rusqlite::Connection>>,
    sessions: &mut Sessions,
    ws_tx: &mut S,
    recipe_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    S: SinkExt<Message> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    let session = match get_or_create_session(conn, sessions, recipe_id) {
        Some(s) => s,
        None => return Ok(()),
    };

    // Merge latest DB state into the session (picks up local edits)
    let am_data = {
        let c = conn.lock().unwrap_or_else(|e| e.into_inner());
        db::get_recipe(&*c, recipe_id)?.map(|r| r.am_data)
    };
    if let Some(am_data) = am_data {
        session.merge_doc(&am_data);
    }

    if let Some(sync_msg_bytes) = session.generate_sync_message() {
        let msg = SyncMessage::SyncDoc {
            recipe_id: recipe_id.to_string(),
            data: sync_msg_bytes,
        };
        ws_tx
            .send(Message::Text(serde_json::to_string(&msg)?.into()))
            .await?;
    }

    {
        let c = conn.lock().unwrap_or_else(|e| e.into_inner());
        db::clear_dirty(&*c, recipe_id)?;
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
        SyncMessage::NewDoc { recipe_id, am_data } => {
            let c = conn.lock().unwrap_or_else(|e| e.into_inner());
            db::apply_remote_merge(&*c, &recipe_id, &am_data)?;
            // Create a session for this newly received recipe
            sessions.insert(
                recipe_id,
                SyncSession::from_doc_bytes(&am_data),
            );
        }
        SyncMessage::SyncDoc { recipe_id, data } => {
            let session = match get_or_create_session(conn, sessions, &recipe_id) {
                Some(s) => s,
                None => return Ok(()),
            };

            let reply = session.receive_sync_message(&data)?;

            if let Some(reply_bytes) = &reply {
                let reply_msg = SyncMessage::SyncDoc {
                    recipe_id: recipe_id.clone(),
                    data: reply_bytes.clone(),
                };
                ws_tx
                    .send(Message::Text(serde_json::to_string(&reply_msg)?.into()))
                    .await?;
            }

            // Save the merged doc to DB
            let saved = session.save_doc();
            let c = conn.lock().unwrap_or_else(|e| e.into_inner());
            db::apply_remote_merge(&*c, &recipe_id, &saved)?;
            if reply.is_none() {
                db::clear_dirty(&*c, &recipe_id)?;
            }
        }
        SyncMessage::Hello { .. } => {
            // Unexpected mid-session Hello, ignore
        }
    }

    Ok(())
}
