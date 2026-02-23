#![cfg(feature = "wasm")]

// WASM sync worker using gloo-net WebSocket.
// This module mirrors sync_worker.rs but uses browser-compatible APIs.
//
// For the initial implementation, the sync logic is exposed through SyncSession
// in the brewdio-wasm crate, and JavaScript controls the WebSocket connection.
// A full gloo-net based worker can be added later if needed.
//
// The reason: gloo-net requires a browser environment with WebSocket API,
// which makes it untestable in bun/node. Instead, we expose SyncSession
// methods via wasm-bindgen and let the JS layer handle WebSocket transport.
