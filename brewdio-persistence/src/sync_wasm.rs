#![cfg(feature = "wasm")]

// WASM sync support.
//
// The full sync protocol (Hello handshake, NewDoc exchange, dirty doc
// checking, incoming message handling) lives in the `SyncWorker` struct
// in `brewdio-wasm/src/sync.rs`. It operates directly on `RecipeDb` and
// calls the same `db::*` functions as the native `sync_worker.rs`.
//
// JavaScript handles only WebSocket transport and timer scheduling
// (`brewdio-webui/src/lib/sync.ts`, ~100 lines). This keeps browser-specific
// concerns out of Rust while keeping all CRDT/protocol logic in WASM.
