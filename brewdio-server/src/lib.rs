use std::sync::{Arc, Mutex};

use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

mod ws_handler;

pub fn start_server(
    listener: TcpListener,
    conn: Arc<Mutex<rusqlite::Connection>>,
) -> JoinHandle<()> {
    let (notify_tx, _) = tokio::sync::broadcast::channel(64);
    let state = ws_handler::ServerState { conn, notify_tx };

    let app = Router::new()
        .route("/ws", get(ws_handler::ws_upgrade))
        .with_state(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    })
}
