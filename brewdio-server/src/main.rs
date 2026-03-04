use std::sync::{Arc, Mutex};

use brewdio_persistence::connection_native;
use tokio::net::TcpListener;

use brewdio_server::auth::config::AuthConfig;
use brewdio_server::auth::password;
use brewdio_server::db;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let no_auth = args.iter().any(|a| a == "--no-auth");

    let db_path = std::env::var("BREWDIO_DB").unwrap_or_else(|_| "brewdio.db".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let conn = connection_native::open(&db_path).expect("Failed to open database");

    // Run server-only migrations (user table, user_id columns)
    db::run_server_migrations(&conn).expect("Failed to run server migrations");

    let mut auth_config = AuthConfig::from_env();
    if no_auth {
        auth_config.no_auth = true;
        eprintln!("[auth] WARNING: Authentication disabled (--no-auth)");
    }

    // Bootstrap admin user if env vars are set
    if let (Some(email), Some(pw)) = (&auth_config.admin_email, &auth_config.admin_password) {
        let hash = password::hash_password(pw).expect("Failed to hash admin password");
        let id = ulid::Ulid::new().to_string().to_lowercase();
        db::upsert_admin(&conn, &id, email, &hash).expect("Failed to bootstrap admin user");
        eprintln!("[auth] Admin user bootstrapped: {}", email);
    }

    let conn = Arc::new(Mutex::new(conn));

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind");

    eprintln!("Listening on {}", listener.local_addr().unwrap());

    let handle = brewdio_server::start_server(listener, conn, auth_config);
    handle.await.unwrap();
}
