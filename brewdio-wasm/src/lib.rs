//! WASM bindings for the core brewing calculations.
//! Types are automatically marshalled via tsify/wasm-bindgen.

use wasm_bindgen::prelude::*;

mod calculations;
mod conversions;
mod data;
mod db;
mod sync;

/// Install the panic hook so WASM panics print readable messages to console.error.
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Serialize to JsValue using JSON-compatible mode (produces plain objects, not Maps).
pub(crate) fn to_js<T: serde::Serialize>(value: &T) -> Result<JsValue, serde_wasm_bindgen::Error> {
    value.serialize(&serde_wasm_bindgen::Serializer::json_compatible())
}
