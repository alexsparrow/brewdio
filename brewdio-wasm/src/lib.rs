//! WASM bindings for the core brewing calculations.
//! Types are automatically marshalled via tsify/wasm-bindgen.

use wasm_bindgen::prelude::*;

mod calculations;
mod conversions;
mod data;
mod db;
mod sync;

/// Install the panic hook and logger so WASM panics print readable messages
/// to console.error and `log::info!()` etc. appear in the browser console.
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).ok();
}

#[wasm_bindgen(js_name = "getVersion")]
pub fn get_version() -> String {
    brewdio_core::VERSION.to_string()
}

/// Serialize to JsValue using JSON-compatible mode (produces plain objects, not Maps).
pub(crate) fn to_js<T: serde::Serialize>(value: &T) -> Result<JsValue, serde_wasm_bindgen::Error> {
    value.serialize(&serde_wasm_bindgen::Serializer::json_compatible())
}
