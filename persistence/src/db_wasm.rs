#![cfg(feature = "wasm")]

use sqlite_wasm_rs::export::sqlite3;

const MIGRATION_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS recipe (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    recipe TEXT NOT NULL,
    am_data BLOB NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    is_dirty BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS sync_state (
    recipe_id TEXT NOT NULL,
    peer_id TEXT NOT NULL DEFAULT 'server',
    state BLOB NOT NULL,
    PRIMARY KEY (recipe_id, peer_id),
    FOREIGN KEY (recipe_id) REFERENCES recipe(id)
);
"#;

/// WASM SQLite database handle using sqlite-wasm-rs.
/// Uses OPFS (Origin Private File System) for persistent storage when available,
/// falls back to in-memory otherwise.
pub struct WasmDb {
    db: *mut sqlite3,
}

// WASM is single-threaded, so this is safe.
unsafe impl Send for WasmDb {}
unsafe impl Sync for WasmDb {}

impl WasmDb {
    /// Open an in-memory database and run migrations.
    pub fn open_memory() -> Result<Self, String> {
        unsafe {
            let mut db: *mut sqlite3 = std::ptr::null_mut();
            let rc = sqlite_wasm_rs::export::sqlite3_open(
                b":memory:\0".as_ptr() as *const i8,
                &mut db,
            );
            if rc != 0 {
                return Err(format!("Failed to open database: error code {}", rc));
            }

            let rc = sqlite_wasm_rs::export::sqlite3_exec(
                db,
                MIGRATION_SQL.as_ptr() as *const i8,
                None,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            if rc != 0 {
                return Err(format!("Failed to run migrations: error code {}", rc));
            }

            Ok(Self { db })
        }
    }

    /// Get the raw database pointer for use with sqlite-wasm-rs functions.
    pub fn as_ptr(&self) -> *mut sqlite3 {
        self.db
    }
}

impl Drop for WasmDb {
    fn drop(&mut self) {
        unsafe {
            sqlite_wasm_rs::export::sqlite3_close(self.db);
        }
    }
}

// TODO: Implement full CRUD functions mirroring db.rs using sqlite-wasm-rs raw API.
// For now, persistence in WASM is primarily handled through SyncSession exports
// and JavaScript-side storage. Once the sqlite-wasm-rs API is fleshed out here,
// OPFS-backed SQLite can be used for full offline recipe storage.
