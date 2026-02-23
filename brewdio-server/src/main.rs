use std::sync::{Arc, Mutex};

use brewdio_persistence::connection_native;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let db_path = std::env::var("BREWDIO_DB").unwrap_or_else(|_| "brewdio.db".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let conn = connection_native::open(&db_path).expect("Failed to open database");
    let conn = Arc::new(Mutex::new(conn));

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind");

    eprintln!("Listening on {}", listener.local_addr().unwrap());

    let handle = brewdio_server::start_server(listener, conn);
    handle.await.unwrap();
}
