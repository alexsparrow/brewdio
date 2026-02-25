#![cfg(feature = "wasm")]

use sqlite_wasm_rs::export::{
    sqlite3, sqlite3_stmt, SQLITE_DONE, SQLITE_OK, SQLITE_ROW,
};
use sqlite_wasm_rs::{
    SQLITE_TRANSIENT, sqlite3_bind_blob, sqlite3_bind_int, sqlite3_bind_null, sqlite3_bind_text,
    sqlite3_close, sqlite3_column_blob, sqlite3_column_bytes, sqlite3_column_count,
    sqlite3_column_int, sqlite3_column_text, sqlite3_errmsg, sqlite3_exec, sqlite3_finalize,
    sqlite3_open, sqlite3_prepare_v2, sqlite3_step,
};
use std::ffi::CStr;

use crate::connection::{run_migrations, Connection, DbError, Row, Value};

/// Install the relaxed-idb VFS (IndexedDB-backed) and set it as the default VFS.
/// Must be called once before opening any persistent database.
/// After installation, `WasmConnection::open("dbname")` will persist to IndexedDB.
pub async fn install_persistent_vfs() -> Result<(), DbError> {
    sqlite_wasm_rs::relaxed_idb_vfs::install(None, true)
        .await
        .map(|_| ())
        .map_err(|e| DbError(format!("Failed to install relaxed-idb VFS: {:?}", e)))
}

/// WASM SQLite connection using sqlite-wasm-rs.
pub struct WasmConnection {
    db: *mut sqlite3,
}

// WASM is single-threaded, so this is safe.
unsafe impl Send for WasmConnection {}
unsafe impl Sync for WasmConnection {}

unsafe fn last_error(db: *mut sqlite3) -> String {
    let msg = sqlite3_errmsg(db);
    if msg.is_null() {
        return "Unknown error".to_string();
    }
    CStr::from_ptr(msg as *const i8)
        .to_string_lossy()
        .into_owned()
}

unsafe fn prepare(db: *mut sqlite3, sql: &[u8]) -> Result<*mut sqlite3_stmt, DbError> {
    let mut stmt: *mut sqlite3_stmt = std::ptr::null_mut();
    let rc = sqlite3_prepare_v2(
        db,
        sql.as_ptr() as *const i8,
        sql.len() as i32,
        &mut stmt,
        std::ptr::null_mut(),
    );
    if rc != SQLITE_OK {
        return Err(DbError(last_error(db)));
    }
    Ok(stmt)
}

unsafe fn bind_text(stmt: *mut sqlite3_stmt, idx: i32, val: &str) -> i32 {
    sqlite3_bind_text(
        stmt,
        idx,
        val.as_ptr() as *const i8,
        val.len() as i32,
        SQLITE_TRANSIENT(),
    )
}

unsafe fn bind_blob(stmt: *mut sqlite3_stmt, idx: i32, val: &[u8]) -> i32 {
    sqlite3_bind_blob(
        stmt,
        idx,
        val.as_ptr() as *const std::ffi::c_void,
        val.len() as i32,
        SQLITE_TRANSIENT(),
    )
}

unsafe fn column_blob(stmt: *mut sqlite3_stmt, idx: i32) -> Vec<u8> {
    let ptr = sqlite3_column_blob(stmt, idx);
    let len = sqlite3_column_bytes(stmt, idx) as usize;
    if ptr.is_null() || len == 0 {
        return Vec::new();
    }
    std::slice::from_raw_parts(ptr as *const u8, len).to_vec()
}

/// A row extracted from a WASM SQLite statement.
/// Stores all representations per column so the caller can access any type.
struct WasmRow {
    cols: Vec<WasmCol>,
}

struct WasmCol {
    text: Option<String>,
    blob: Vec<u8>,
    int: i32,
}

impl Row for WasmRow {
    fn get_text(&self, idx: usize) -> String {
        self.cols[idx].text.clone().unwrap_or_default()
    }

    fn get_optional_text(&self, idx: usize) -> Option<String> {
        self.cols[idx].text.clone()
    }

    fn get_blob(&self, idx: usize) -> Vec<u8> {
        self.cols[idx].blob.clone()
    }

    fn get_bool(&self, idx: usize) -> bool {
        self.cols[idx].int != 0
    }
}

impl WasmConnection {
    /// Open an in-memory database and run migrations.
    pub fn open_memory() -> Result<Self, DbError> {
        Self::open_raw(b":memory:\0")
    }

    /// Open a named database (for use with OPFS VFS).
    pub fn open(path: &str) -> Result<Self, DbError> {
        let mut path_bytes = path.as_bytes().to_vec();
        path_bytes.push(0);
        Self::open_raw(&path_bytes)
    }

    fn open_raw(path: &[u8]) -> Result<Self, DbError> {
        unsafe {
            let mut db: *mut sqlite3 = std::ptr::null_mut();
            let rc = sqlite3_open(path.as_ptr() as *const i8, &mut db);
            if rc != 0 {
                return Err(DbError(format!(
                    "Failed to open database: error code {}",
                    rc
                )));
            }

            let conn = Self { db };
            run_migrations(&conn)?;
            Ok(conn)
        }
    }

    /// Get the raw database pointer for use with sqlite-wasm-rs functions.
    pub fn as_ptr(&self) -> *mut sqlite3 {
        self.db
    }
}

impl Connection for WasmConnection {
    fn execute(&self, sql: &str, params: &[Value]) -> Result<(), DbError> {
        unsafe {
            let mut sql_bytes = sql.as_bytes().to_vec();
            sql_bytes.push(0);
            let stmt = prepare(self.db, &sql_bytes)?;

            for (i, param) in params.iter().enumerate() {
                let idx = (i + 1) as i32;
                match param {
                    Value::Text(s) => {
                        bind_text(stmt, idx, s);
                    }
                    Value::OptionalText(Some(s)) => {
                        bind_text(stmt, idx, s);
                    }
                    Value::OptionalText(None) => {
                        sqlite3_bind_null(stmt, idx);
                    }
                    Value::Blob(b) => {
                        bind_blob(stmt, idx, b);
                    }
                    Value::Int(n) => {
                        sqlite3_bind_int(stmt, idx, *n);
                    }
                    Value::Bool(b) => {
                        sqlite3_bind_int(stmt, idx, *b as i32);
                    }
                }
            }

            let rc = sqlite3_step(stmt);
            sqlite3_finalize(stmt);
            if rc != SQLITE_DONE {
                return Err(DbError(last_error(self.db)));
            }
            Ok(())
        }
    }

    fn execute_batch(&self, sql: &str) -> Result<(), DbError> {
        unsafe {
            let mut sql_bytes = sql.as_bytes().to_vec();
            sql_bytes.push(0);
            let rc = sqlite3_exec(
                self.db,
                sql_bytes.as_ptr() as *const i8,
                None,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            if rc != 0 {
                return Err(DbError(last_error(self.db)));
            }
            Ok(())
        }
    }

    fn query_map<T, F>(&self, sql: &str, params: &[Value], mut f: F) -> Result<Vec<T>, DbError>
    where
        F: FnMut(&dyn Row) -> T,
    {
        unsafe {
            let mut sql_bytes = sql.as_bytes().to_vec();
            sql_bytes.push(0);
            let stmt = prepare(self.db, &sql_bytes)?;

            for (i, param) in params.iter().enumerate() {
                let idx = (i + 1) as i32;
                match param {
                    Value::Text(s) => {
                        bind_text(stmt, idx, s);
                    }
                    Value::OptionalText(Some(s)) => {
                        bind_text(stmt, idx, s);
                    }
                    Value::OptionalText(None) => {
                        sqlite3_bind_null(stmt, idx);
                    }
                    Value::Blob(b) => {
                        bind_blob(stmt, idx, b);
                    }
                    Value::Int(n) => {
                        sqlite3_bind_int(stmt, idx, *n);
                    }
                    Value::Bool(b) => {
                        sqlite3_bind_int(stmt, idx, *b as i32);
                    }
                }
            }

            let col_count = sqlite3_column_count(stmt) as usize;
            let mut results = Vec::new();
            loop {
                let rc = sqlite3_step(stmt);
                if rc == SQLITE_ROW {
                    let mut cols = Vec::with_capacity(col_count);
                    for c in 0..col_count {
                        let ci = c as i32;
                        // Read blob BEFORE text: calling column_text on a BLOB
                        // column triggers an internal BLOB→TEXT conversion in
                        // SQLite, which can corrupt the data that column_blob
                        // would subsequently return.
                        let blob = column_blob(stmt, ci);
                        let text_ptr = sqlite3_column_text(stmt, ci);
                        let text = if text_ptr.is_null() {
                            None
                        } else {
                            Some(
                                CStr::from_ptr(text_ptr as *const i8)
                                    .to_string_lossy()
                                    .into_owned(),
                            )
                        };
                        let int = sqlite3_column_int(stmt, ci);
                        cols.push(WasmCol { text, blob, int });
                    }
                    let row = WasmRow { cols };
                    results.push(f(&row));
                } else if rc == SQLITE_DONE {
                    break;
                } else {
                    let err = last_error(self.db);
                    sqlite3_finalize(stmt);
                    return Err(DbError(err));
                }
            }
            sqlite3_finalize(stmt);
            Ok(results)
        }
    }
}

impl Drop for WasmConnection {
    fn drop(&mut self) {
        unsafe {
            sqlite3_close(self.db);
        }
    }
}
