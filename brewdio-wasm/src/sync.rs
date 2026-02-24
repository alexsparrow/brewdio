use wasm_bindgen::prelude::*;
use brewdio_persistence::sync::SyncSession as InnerSyncSession;

#[wasm_bindgen]
pub struct SyncSession {
    inner: InnerSyncSession,
}

#[wasm_bindgen]
impl SyncSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: InnerSyncSession::new(),
        }
    }

    #[wasm_bindgen(js_name = "fromBytes")]
    pub fn from_bytes(am_bytes: &[u8]) -> Self {
        Self {
            inner: InnerSyncSession::from_doc_bytes(am_bytes),
        }
    }

    #[wasm_bindgen(js_name = "fromDocAndState")]
    pub fn from_doc_and_state(am_bytes: &[u8], state_bytes: &[u8]) -> Self {
        Self {
            inner: InnerSyncSession::from_doc_and_state(am_bytes, state_bytes),
        }
    }

    #[wasm_bindgen(js_name = "reconcileJson")]
    pub fn reconcile_json(&mut self, recipe_json: &str) {
        let doc: brewdio_persistence::recipe::RecipeDocument =
            serde_json::from_str(recipe_json).expect("Failed to parse RecipeDocument JSON");
        self.inner.reconcile(&doc);
    }

    #[wasm_bindgen(js_name = "hydrateJson")]
    pub fn hydrate_json(&self) -> String {
        let doc = self.inner.hydrate();
        serde_json::to_string(&doc).expect("Failed to serialize RecipeDocument")
    }

    #[wasm_bindgen(js_name = "generateSyncMessage")]
    pub fn generate_sync_message(&mut self) -> Option<Vec<u8>> {
        self.inner.generate_sync_message()
    }

    #[wasm_bindgen(js_name = "receiveSyncMessage")]
    pub fn receive_sync_message(&mut self, data: &[u8]) -> Result<Option<Vec<u8>>, JsError> {
        self.inner
            .receive_sync_message(data)
            .map_err(|e| JsError::new(&format!("Sync error: {}", e)))
    }

    #[wasm_bindgen(js_name = "saveDoc")]
    pub fn save_doc(&mut self) -> Vec<u8> {
        self.inner.save_doc()
    }

    #[wasm_bindgen(js_name = "saveState")]
    pub fn save_state(&self) -> Vec<u8> {
        self.inner.save_state()
    }

    #[wasm_bindgen(js_name = "loadState")]
    pub fn load_state(&mut self, bytes: &[u8]) {
        self.inner.load_state(bytes);
    }

    #[wasm_bindgen(js_name = "isSynced")]
    pub fn is_synced(&mut self) -> bool {
        self.inner.is_synced()
    }
}
