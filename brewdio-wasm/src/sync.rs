use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;
use brewdio_persistence::db;
use brewdio_persistence::protocol::{DocType, SyncMessage};
use brewdio_persistence::sync::SyncSession as InnerSyncSession;
use crate::db::RecipeDb;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);
}

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

// ---------------------------------------------------------------------------
// SyncWorker — encapsulates the full sync protocol for the WebUI
// ---------------------------------------------------------------------------

const PEER_ID: &str = "server";

type SessionKey = (DocType, String);
type Sessions = HashMap<SessionKey, InnerSyncSession>;

#[wasm_bindgen]
pub struct SyncWorker {
    sessions: Sessions,
}

fn notify_for_doc_type(db: &RecipeDb, doc_type: DocType) {
    console_log(&format!("[sync] notify_for_doc_type: {:?}", doc_type));
    match doc_type {
        DocType::Recipe => db.notify(),
        DocType::Batch => db.notify_batches(),
        DocType::Settings => db.notify_settings(),
        DocType::EquipmentProfile => db.notify_equipment(),
    }
}

fn get_or_create_session<'a>(
    conn: &impl brewdio_persistence::connection::Connection,
    sessions: &'a mut Sessions,
    doc_type: DocType,
    doc_id: &str,
) -> Option<&'a mut InnerSyncSession> {
    let key = (doc_type, doc_id.to_string());
    if !sessions.contains_key(&key) {
        let am_data = db::get_doc_am_data(conn, doc_type, doc_id).ok().flatten();
        if let Some(am_data) = am_data {
            sessions.insert(key.clone(), InnerSyncSession::from_doc_bytes(&am_data));
        } else {
            return None;
        }
    }
    sessions.get_mut(&key)
}

#[wasm_bindgen]
impl SyncWorker {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Called when the WebSocket connects. Clears stale sync states and returns
    /// a serialized Hello message to send to the server.
    #[wasm_bindgen(js_name = "onConnected")]
    pub fn on_connected(&mut self, db: &RecipeDb) -> Result<String, JsError> {
        let conn = db.conn();
        self.sessions.clear();
        db::clear_sync_states_for_peer(conn, PEER_ID)
            .map_err(|e| JsError::new(&e.to_string()))?;

        let recipe_ids = db::list_all_doc_ids(conn, DocType::Recipe)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let batch_ids = db::list_all_doc_ids(conn, DocType::Batch)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let settings_ids = db::list_all_doc_ids(conn, DocType::Settings)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let equipment_profile_ids = db::list_all_doc_ids(conn, DocType::EquipmentProfile)
            .map_err(|e| JsError::new(&e.to_string()))?;

        let hello = SyncMessage::Hello {
            recipe_ids,
            batch_ids,
            settings_ids,
            equipment_profile_ids,
        };
        serde_json::to_string(&hello).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Process the server's Hello message. Returns a JS array of serialized
    /// NewDoc JSON strings for docs the server doesn't have.
    #[wasm_bindgen(js_name = "processHello")]
    pub fn process_hello(&mut self, db: &RecipeDb, hello_json: &str) -> Result<JsValue, JsError> {
        let conn = db.conn();
        let server_hello: SyncMessage =
            serde_json::from_str(hello_json).map_err(|e| JsError::new(&e.to_string()))?;

        let (server_recipe_ids, server_batch_ids, server_settings_ids, server_equipment_ids) =
            match server_hello {
                SyncMessage::Hello {
                    recipe_ids,
                    batch_ids,
                    settings_ids,
                    equipment_profile_ids,
                } => (
                    recipe_ids.into_iter().collect::<HashSet<_>>(),
                    batch_ids.into_iter().collect::<HashSet<_>>(),
                    settings_ids.into_iter().collect::<HashSet<_>>(),
                    equipment_profile_ids.into_iter().collect::<HashSet<_>>(),
                ),
                _ => return Err(JsError::new("Expected Hello message from server")),
            };

        let local_recipe_ids = db::list_all_doc_ids(conn, DocType::Recipe)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let local_batch_ids = db::list_all_doc_ids(conn, DocType::Batch)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let local_settings_ids = db::list_all_doc_ids(conn, DocType::Settings)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let local_equipment_ids = db::list_all_doc_ids(conn, DocType::EquipmentProfile)
            .map_err(|e| JsError::new(&e.to_string()))?;

        let local_sets: [(DocType, HashSet<String>, &HashSet<String>); 4] = [
            (DocType::Recipe, local_recipe_ids.into_iter().collect(), &server_recipe_ids),
            (DocType::Batch, local_batch_ids.into_iter().collect(), &server_batch_ids),
            (DocType::Settings, local_settings_ids.into_iter().collect(), &server_settings_ids),
            (DocType::EquipmentProfile, local_equipment_ids.into_iter().collect(), &server_equipment_ids),
        ];

        let result = js_sys::Array::new();
        for (doc_type, local_ids, server_ids) in &local_sets {
            for id in local_ids.difference(server_ids) {
                let am_data = db::get_doc_am_data(conn, *doc_type, id)
                    .map_err(|e| JsError::new(&e.to_string()))?;
                if let Some(am_data) = am_data {
                    let msg = SyncMessage::NewDoc {
                        doc_type: *doc_type,
                        doc_id: id.clone(),
                        am_data: am_data.clone(),
                    };
                    let json = serde_json::to_string(&msg)
                        .map_err(|e| JsError::new(&e.to_string()))?;
                    result.push(&JsValue::from_str(&json));

                    db::clear_dirty_doc(conn, *doc_type, id)
                        .map_err(|e| JsError::new(&e.to_string()))?;

                    // Create session for the sent doc
                    let key = (*doc_type, id.clone());
                    self.sessions.insert(key, InnerSyncSession::from_doc_bytes(&am_data));
                }
            }
        }

        Ok(result.into())
    }

    /// Handle an incoming message from the server. Returns a JS array of
    /// serialized reply messages to send back.
    #[wasm_bindgen(js_name = "onMessage")]
    pub fn on_message(&mut self, db: &RecipeDb, msg_json: &str) -> Result<JsValue, JsError> {
        let conn = db.conn();
        let msg: SyncMessage =
            serde_json::from_str(msg_json).map_err(|e| JsError::new(&e.to_string()))?;

        let result = js_sys::Array::new();

        match msg {
            SyncMessage::NewDoc {
                doc_type,
                doc_id,
                am_data,
            } => {
                console_log(&format!("[sync] received NewDoc {:?}/{}", doc_type, doc_id));
                db::apply_remote_merge_doc(conn, doc_type, &doc_id, &am_data)
                    .map_err(|e| JsError::new(&e.to_string()))?;
                let key = (doc_type, doc_id);
                self.sessions
                    .insert(key, InnerSyncSession::from_doc_bytes(&am_data));
                notify_for_doc_type(db, doc_type);
            }
            SyncMessage::SyncDoc {
                doc_type,
                doc_id,
                data,
            } => {
                console_log(&format!("[sync] received SyncDoc {:?}/{} ({} bytes)", doc_type, doc_id, data.len()));
                let session =
                    match get_or_create_session(conn, &mut self.sessions, doc_type, &doc_id) {
                        Some(s) => s,
                        None => return Ok(result.into()),
                    };

                let reply = session
                    .receive_sync_message(&data)
                    .map_err(|e| JsError::new(&format!("Sync error: {}", e)))?;

                if let Some(ref reply_bytes) = reply {
                    let reply_msg = SyncMessage::SyncDoc {
                        doc_type,
                        doc_id: doc_id.clone(),
                        data: reply_bytes.clone(),
                    };
                    let json = serde_json::to_string(&reply_msg)
                        .map_err(|e| JsError::new(&e.to_string()))?;
                    result.push(&JsValue::from_str(&json));
                }

                // Save merged doc to DB
                let saved = session.save_doc();
                db::apply_remote_merge_doc(conn, doc_type, &doc_id, &saved)
                    .map_err(|e| JsError::new(&e.to_string()))?;
                if reply.is_none() {
                    db::clear_dirty_doc(conn, doc_type, &doc_id)
                        .map_err(|e| JsError::new(&e.to_string()))?;
                }
                notify_for_doc_type(db, doc_type);
            }
            SyncMessage::Hello { .. } => {
                // Unexpected mid-session Hello, ignore
            }
        }

        Ok(result.into())
    }

    /// Check for locally-dirty docs and generate sync messages for them.
    /// Returns a JS array of serialized SyncDoc JSON strings.
    #[wasm_bindgen(js_name = "checkDirty")]
    pub fn check_dirty(&mut self, db: &RecipeDb) -> Result<JsValue, JsError> {
        let conn = db.conn();
        let dirty = db::list_dirty_docs(conn).map_err(|e| JsError::new(&e.to_string()))?;
        let result = js_sys::Array::new();

        for (doc_type, doc_id) in &dirty {
            let session =
                match get_or_create_session(conn, &mut self.sessions, *doc_type, doc_id) {
                    Some(s) => s,
                    None => continue,
                };

            // Merge latest DB state into session
            let am_data = db::get_doc_am_data(conn, *doc_type, doc_id)
                .map_err(|e| JsError::new(&e.to_string()))?;
            if let Some(am_data) = am_data {
                session.merge_doc(&am_data);
            }

            if let Some(sync_msg_bytes) = session.generate_sync_message() {
                let msg = SyncMessage::SyncDoc {
                    doc_type: *doc_type,
                    doc_id: doc_id.clone(),
                    data: sync_msg_bytes,
                };
                let json = serde_json::to_string(&msg)
                    .map_err(|e| JsError::new(&e.to_string()))?;
                result.push(&JsValue::from_str(&json));
            }

            db::clear_dirty_doc(conn, *doc_type, doc_id)
                .map_err(|e| JsError::new(&e.to_string()))?;
        }

        Ok(result.into())
    }

    /// Called when the WebSocket disconnects. Clears all sessions.
    #[wasm_bindgen(js_name = "onDisconnected")]
    pub fn on_disconnected(&mut self) {
        self.sessions.clear();
    }
}
