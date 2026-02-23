#![cfg(feature = "native")]

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use rusqlite::Connection;
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

/// Spawn a background sync worker that connects to the given WebSocket server URL.
/// Returns a `JoinHandle` for the spawned task.
///
/// The `Connection` is wrapped in `Arc<Mutex<>>` since rusqlite is sync-only.
pub fn spawn_sync(conn: Arc<Mutex<Connection>>, server_url: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            eprintln!("[sync] connecting to {}", server_url);
            match connect_and_sync(&conn, &server_url).await {
                Ok(()) => eprintln!("[sync] connection closed cleanly"),
                Err(e) => eprintln!("[sync] connection error: {}", e),
            }
            eprintln!("[sync] reconnecting in {}s...", RECONNECT_DELAY.as_secs());
            sleep(RECONNECT_DELAY).await;
        }
    })
}

async fn connect_and_sync(
    conn: &Arc<Mutex<Connection>>,
    server_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (ws_stream, _) = connect_async(server_url).await?;
    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    // --- Hello handshake ---
    let local_ids = {
        let c = conn.lock().unwrap();
        db::list_all_recipe_ids(&c)?
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

    // Send NewDoc for recipes only we have
    for id in local_id_set.difference(&server_ids) {
        let row = {
            let c = conn.lock().unwrap();
            db::get_recipe(&c, id)?
        };
        if let Some(row) = row {
            let msg = SyncMessage::NewDoc {
                recipe_id: id.clone(),
                am_data: row.am_data,
            };
            ws_tx
                .send(Message::Text(serde_json::to_string(&msg)?.into()))
                .await?;
        }
    }

    // IDs that both sides share — need sync
    let shared_ids: Vec<String> = local_id_set.intersection(&server_ids).cloned().collect();

    // Initiate sync for shared recipes
    for id in &shared_ids {
        send_sync_for_recipe(conn, &mut ws_tx, id).await?;
    }

    // --- Steady-state loop ---
    let mut dirty_interval = tokio::time::interval(DIRTY_CHECK_INTERVAL);

    loop {
        tokio::select! {
            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_incoming(conn, &mut ws_tx, &text).await?;
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
                    let c = conn.lock().unwrap();
                    db::list_dirty_recipes(&c)?
                };
                for row in &dirty {
                    send_sync_for_recipe(conn, &mut ws_tx, &row.id).await?;
                }
            }
        }
    }
}

/// Generate and send a sync message for a single recipe using `SyncSession`.
async fn send_sync_for_recipe<S>(
    conn: &Arc<Mutex<Connection>>,
    ws_tx: &mut S,
    recipe_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    S: SinkExt<Message> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    let (row, state_bytes) = {
        let c = conn.lock().unwrap();
        let row = match db::get_recipe(&c, recipe_id)? {
            Some(r) => r,
            None => return Ok(()),
        };
        let state = db::get_sync_state(&c, recipe_id, PEER_ID)?;
        (row, state)
    };

    let mut session = match state_bytes {
        Some(sb) => SyncSession::from_doc_and_state(&row.am_data, &sb),
        None => SyncSession::from_doc_bytes(&row.am_data),
    };

    if let Some(sync_msg_bytes) = session.generate_sync_message() {
        let msg = SyncMessage::SyncDoc {
            recipe_id: recipe_id.to_string(),
            data: sync_msg_bytes,
        };
        ws_tx
            .send(Message::Text(serde_json::to_string(&msg)?.into()))
            .await?;

        // Persist sync state
        let c = conn.lock().unwrap();
        db::save_sync_state(&c, recipe_id, PEER_ID, &session.save_state())?;
    } else {
        // Already converged — clear dirty
        let c = conn.lock().unwrap();
        db::clear_dirty(&c, recipe_id)?;
    }

    Ok(())
}

/// Handle an incoming WebSocket message from the server.
async fn handle_incoming<S>(
    conn: &Arc<Mutex<Connection>>,
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
            let c = conn.lock().unwrap();
            db::apply_remote_merge(&c, &recipe_id, &am_data)?;
        }
        SyncMessage::SyncDoc { recipe_id, data } => {
            let (row, state_bytes) = {
                let c = conn.lock().unwrap();
                let row = match db::get_recipe(&c, &recipe_id)? {
                    Some(r) => r,
                    None => return Ok(()),
                };
                let state = db::get_sync_state(&c, &recipe_id, PEER_ID)?;
                (row, state)
            };

            let mut session = match state_bytes {
                Some(sb) => SyncSession::from_doc_and_state(&row.am_data, &sb),
                None => SyncSession::from_doc_bytes(&row.am_data),
            };

            let reply = session.receive_sync_message(&data)?;

            match reply {
                Some(reply_bytes) => {
                    let reply_msg = SyncMessage::SyncDoc {
                        recipe_id: recipe_id.clone(),
                        data: reply_bytes,
                    };
                    ws_tx
                        .send(Message::Text(serde_json::to_string(&reply_msg)?.into()))
                        .await?;
                }
                None => {
                    // Converged — save merged doc and clear dirty
                    let saved = session.save_doc();
                    let c = conn.lock().unwrap();
                    db::apply_remote_merge(&c, &recipe_id, &saved)?;
                    db::clear_dirty(&c, &recipe_id)?;
                }
            }

            // Persist sync state
            let c = conn.lock().unwrap();
            db::save_sync_state(&c, &recipe_id, PEER_ID, &session.save_state())?;
        }
        SyncMessage::Hello { .. } => {
            // Unexpected mid-session Hello, ignore
        }
    }

    Ok(())
}
