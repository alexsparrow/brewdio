use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use tokio::time::{interval, Duration};

use brewdio_persistence::db;
use brewdio_persistence::protocol::SyncMessage;
use brewdio_persistence::sync::SyncSession;

const PEER_ID: &str = "client";
const DIRTY_CHECK_INTERVAL: Duration = Duration::from_secs(2);

type AppState = Arc<Mutex<rusqlite::Connection>>;
type WsSender = SplitSink<WebSocket, Message>;

/// Persistent sync sessions keyed by recipe ID, kept alive for the connection.
type Sessions = HashMap<String, SyncSession>;

pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(conn): State<AppState>,
) -> impl IntoResponse {
    eprintln!("[server] client connected");
    ws.on_upgrade(move |socket| handle_socket(socket, conn))
}

async fn handle_socket(socket: WebSocket, conn: AppState) {
    if let Err(e) = run_sync(socket, conn).await {
        eprintln!("[server] sync error: {}", e);
    }
    eprintln!("[server] client disconnected");
}

async fn run_sync(
    socket: WebSocket,
    conn: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let mut sessions: Sessions = HashMap::new();

    // --- Wait for client Hello ---
    let client_hello = loop {
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

    let client_ids: HashSet<String> = match client_hello {
        SyncMessage::Hello { recipe_ids } => {
            eprintln!("[server] received Hello with {} recipe(s)", recipe_ids.len());
            recipe_ids.into_iter().collect()
        }
        _ => return Err("Expected Hello from client".into()),
    };

    // Clear stale sync states from any previous connection
    {
        let c = conn.lock().unwrap_or_else(|e| e.into_inner());
        db::clear_sync_states_for_peer(&*c, PEER_ID)?;
    }

    // --- Send server Hello ---
    let local_ids = {
        let c = conn.lock().unwrap_or_else(|e| e.into_inner());
        db::list_all_recipe_ids(&*c)?
    };
    eprintln!("[server] sending Hello with {} recipe(s)", local_ids.len());
    let hello = SyncMessage::Hello {
        recipe_ids: local_ids.clone(),
    };
    ws_tx
        .send(Message::Text(serde_json::to_string(&hello)?.into()))
        .await?;

    let local_id_set: HashSet<String> = local_ids.into_iter().collect();

    // Send NewDoc for server-only recipes
    let server_only: Vec<_> = local_id_set.difference(&client_ids).cloned().collect();
    if !server_only.is_empty() {
        eprintln!("[server] sending {} NewDoc(s) for server-only recipes", server_only.len());
    }
    for id in &server_only {
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

    // Initiate sync for shared recipes
    let shared_ids: Vec<String> = local_id_set.intersection(&client_ids).cloned().collect();
    if !shared_ids.is_empty() {
        eprintln!("[server] initiating sync for {} shared recipe(s)", shared_ids.len());
    }
    for id in &shared_ids {
        send_sync_for_recipe(&conn, &mut sessions, &mut ws_tx, id).await?;
    }

    eprintln!("[server] entering steady-state sync loop");

    // --- Steady-state loop ---
    let mut dirty_interval = interval(DIRTY_CHECK_INTERVAL);

    loop {
        tokio::select! {
            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_incoming(&conn, &mut sessions, &mut ws_tx, &text).await?;
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
                if !dirty.is_empty() {
                    eprintln!("[server] pushing {} dirty recipe(s)", dirty.len());
                }
                for row in &dirty {
                    send_sync_for_recipe(&conn, &mut sessions, &mut ws_tx, &row.id).await?;
                }
            }
        }
    }
}

/// Get or create a persistent SyncSession for a recipe.
fn get_or_create_session<'a>(
    conn: &AppState,
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

async fn send_sync_for_recipe(
    conn: &AppState,
    sessions: &mut Sessions,
    ws_tx: &mut WsSender,
    recipe_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        eprintln!("[server] sending SyncDoc for {}", recipe_id);
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

async fn handle_incoming(
    conn: &AppState,
    sessions: &mut Sessions,
    ws_tx: &mut WsSender,
    text: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let msg: SyncMessage = serde_json::from_str(text)?;

    match msg {
        SyncMessage::NewDoc { recipe_id, ref am_data } => {
            eprintln!("[server] received NewDoc for {} ({} bytes)", recipe_id, am_data.len());
            let c = conn.lock().unwrap_or_else(|e| e.into_inner());
            db::apply_remote_merge(&*c, &recipe_id, am_data)?;
            // Create a session for this newly received recipe
            sessions.insert(
                recipe_id,
                SyncSession::from_doc_bytes(am_data),
            );
        }
        SyncMessage::SyncDoc { recipe_id, data } => {
            eprintln!("[server] received SyncDoc for {}", recipe_id);

            let session = match get_or_create_session(conn, sessions, &recipe_id) {
                Some(s) => s,
                None => return Ok(()),
            };

            let reply = session.receive_sync_message(&data)?;

            if let Some(ref reply_bytes) = reply {
                eprintln!("[server] replying SyncDoc for {}", recipe_id);
                let reply_msg = SyncMessage::SyncDoc {
                    recipe_id: recipe_id.clone(),
                    data: reply_bytes.clone(),
                };
                ws_tx
                    .send(Message::Text(serde_json::to_string(&reply_msg)?.into()))
                    .await?;
            } else {
                eprintln!("[server] converged for {}", recipe_id);
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
            eprintln!("[server] unexpected mid-session Hello, ignoring");
        }
    }

    Ok(())
}
