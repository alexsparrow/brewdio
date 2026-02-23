use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use tokio::time::{interval, Duration};

use brewdio_persistence::db;
use brewdio_persistence::protocol::{DocType, SyncMessage};
use brewdio_persistence::sync::SyncSession;

const PEER_ID: &str = "client";
const DIRTY_CHECK_INTERVAL: Duration = Duration::from_secs(2);

type AppState = Arc<Mutex<rusqlite::Connection>>;
type WsSender = SplitSink<WebSocket, Message>;

/// Persistent sync sessions keyed by (DocType, doc_id), kept alive for the connection.
type Sessions = HashMap<(DocType, String), SyncSession>;

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

    let (client_recipe_ids, client_batch_ids, client_settings_ids) = match client_hello {
        SyncMessage::Hello {
            recipe_ids,
            batch_ids,
            settings_ids,
        } => {
            eprintln!(
                "[server] received Hello with {} recipe(s), {} batch(es), {} settings",
                recipe_ids.len(),
                batch_ids.len(),
                settings_ids.len()
            );
            (
                recipe_ids.into_iter().collect::<HashSet<_>>(),
                batch_ids.into_iter().collect::<HashSet<_>>(),
                settings_ids.into_iter().collect::<HashSet<_>>(),
            )
        }
        _ => return Err("Expected Hello from client".into()),
    };

    // Clear stale sync states from any previous connection
    {
        let c = conn.lock().unwrap_or_else(|e| e.into_inner());
        db::clear_sync_states_for_peer(&*c, PEER_ID)?;
    }

    // --- Send server Hello ---
    let (local_recipe_ids, local_batch_ids, local_settings_ids) = {
        let c = conn.lock().unwrap_or_else(|e| e.into_inner());
        (
            db::list_all_doc_ids(&*c, DocType::Recipe)?,
            db::list_all_doc_ids(&*c, DocType::Batch)?,
            db::list_all_doc_ids(&*c, DocType::Settings)?,
        )
    };
    eprintln!(
        "[server] sending Hello with {} recipe(s), {} batch(es), {} settings",
        local_recipe_ids.len(),
        local_batch_ids.len(),
        local_settings_ids.len()
    );
    let hello = SyncMessage::Hello {
        recipe_ids: local_recipe_ids.clone(),
        batch_ids: local_batch_ids.clone(),
        settings_ids: local_settings_ids.clone(),
    };
    ws_tx
        .send(Message::Text(serde_json::to_string(&hello)?.into()))
        .await?;

    // Send NewDoc for server-only docs and initiate sync for shared docs
    let local_sets: [(DocType, HashSet<String>, &HashSet<String>); 3] = [
        (DocType::Recipe, local_recipe_ids.into_iter().collect(), &client_recipe_ids),
        (DocType::Batch, local_batch_ids.into_iter().collect(), &client_batch_ids),
        (DocType::Settings, local_settings_ids.into_iter().collect(), &client_settings_ids),
    ];

    for (doc_type, local_ids, client_ids) in &local_sets {
        // Send NewDoc for server-only docs
        let server_only: Vec<_> = local_ids.difference(client_ids).cloned().collect();
        if !server_only.is_empty() {
            eprintln!(
                "[server] sending {} NewDoc(s) for server-only {:?}",
                server_only.len(),
                doc_type
            );
        }
        for id in &server_only {
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

        // Initiate sync for shared docs
        let shared_ids: Vec<String> = local_ids.intersection(client_ids).cloned().collect();
        if !shared_ids.is_empty() {
            eprintln!(
                "[server] initiating sync for {} shared {:?}(s)",
                shared_ids.len(),
                doc_type
            );
        }
        for id in &shared_ids {
            send_sync_for_doc(&conn, &mut sessions, &mut ws_tx, *doc_type, id).await?;
        }
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
                    db::list_dirty_docs(&*c)?
                };
                if !dirty.is_empty() {
                    eprintln!("[server] pushing {} dirty doc(s)", dirty.len());
                }
                for (doc_type, id) in &dirty {
                    send_sync_for_doc(&conn, &mut sessions, &mut ws_tx, *doc_type, id).await?;
                }
            }
        }
    }
}

/// Get or create a persistent SyncSession for a document.
fn get_or_create_session<'a>(
    conn: &AppState,
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

async fn send_sync_for_doc(
    conn: &AppState,
    sessions: &mut Sessions,
    ws_tx: &mut WsSender,
    doc_type: DocType,
    doc_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        eprintln!("[server] sending SyncDoc for {:?}/{}", doc_type, doc_id);
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

async fn handle_incoming(
    conn: &AppState,
    sessions: &mut Sessions,
    ws_tx: &mut WsSender,
    text: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let msg: SyncMessage = serde_json::from_str(text)?;

    match msg {
        SyncMessage::NewDoc {
            doc_type,
            doc_id,
            ref am_data,
        } => {
            eprintln!(
                "[server] received NewDoc for {:?}/{} ({} bytes)",
                doc_type,
                doc_id,
                am_data.len()
            );
            let c = conn.lock().unwrap_or_else(|e| e.into_inner());
            db::apply_remote_merge_doc(&*c, doc_type, &doc_id, am_data)?;
            sessions.insert(
                (doc_type, doc_id),
                SyncSession::from_doc_bytes(am_data),
            );
        }
        SyncMessage::SyncDoc {
            doc_type,
            doc_id,
            data,
        } => {
            eprintln!("[server] received SyncDoc for {:?}/{}", doc_type, doc_id);

            let session = match get_or_create_session(conn, sessions, doc_type, &doc_id) {
                Some(s) => s,
                None => return Ok(()),
            };

            let reply = session.receive_sync_message(&data)?;

            if let Some(ref reply_bytes) = reply {
                eprintln!("[server] replying SyncDoc for {:?}/{}", doc_type, doc_id);
                let reply_msg = SyncMessage::SyncDoc {
                    doc_type,
                    doc_id: doc_id.clone(),
                    data: reply_bytes.clone(),
                };
                ws_tx
                    .send(Message::Text(serde_json::to_string(&reply_msg)?.into()))
                    .await?;
            } else {
                eprintln!("[server] converged for {:?}/{}", doc_type, doc_id);
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
            eprintln!("[server] unexpected mid-session Hello, ignoring");
        }
    }

    Ok(())
}
