use automerge::sync;
use automerge::sync::SyncDoc;
use automerge::AutoCommit;

use crate::recipe::RecipeDocument;

/// Platform-independent Automerge sync state machine.
/// Used by both native (`sync_worker.rs`) and WASM (`sync_wasm.rs`) sync implementations.
pub struct SyncSession {
    doc: AutoCommit,
    state: sync::State,
}

impl SyncSession {
    /// Create a new empty sync session.
    pub fn new() -> Self {
        Self {
            doc: AutoCommit::new(),
            state: sync::State::new(),
        }
    }

    /// Create a sync session from an existing Automerge document.
    pub fn from_doc_bytes(am_bytes: &[u8]) -> Self {
        let doc = AutoCommit::load(am_bytes).expect("Failed to load Automerge doc");
        Self {
            doc,
            state: sync::State::new(),
        }
    }

    /// Create a sync session from existing doc and sync state bytes.
    pub fn from_doc_and_state(am_bytes: &[u8], state_bytes: &[u8]) -> Self {
        let doc = AutoCommit::load(am_bytes).expect("Failed to load Automerge doc");
        let state =
            sync::State::decode(state_bytes).unwrap_or_else(|_| sync::State::new());
        Self { doc, state }
    }

    /// Reconcile a `RecipeDocument` into the Automerge doc.
    pub fn reconcile(&mut self, recipe_doc: &RecipeDocument) {
        autosurgeon::reconcile(&mut self.doc, recipe_doc)
            .expect("Failed to reconcile document");
    }

    /// Hydrate a `RecipeDocument` from the current Automerge doc.
    pub fn hydrate(&self) -> RecipeDocument {
        autosurgeon::hydrate(&self.doc).expect("Failed to hydrate document")
    }

    /// Generate the next sync message to send to the peer.
    /// Returns `None` if already synced.
    pub fn generate_sync_message(&mut self) -> Option<Vec<u8>> {
        self.doc
            .sync()
            .generate_sync_message(&mut self.state)
            .map(|msg| msg.encode())
    }

    /// Receive a sync message from the peer.
    /// Returns the next sync message to reply with, or `None` if converged.
    pub fn receive_sync_message(
        &mut self,
        msg_bytes: &[u8],
    ) -> Result<Option<Vec<u8>>, automerge::AutomergeError> {
        let msg = sync::Message::decode(msg_bytes).expect("Failed to decode sync message");
        self.doc
            .sync()
            .receive_sync_message(&mut self.state, msg)?;

        Ok(self.generate_sync_message())
    }

    /// Save the Automerge document to bytes.
    pub fn save_doc(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    /// Save the sync state to bytes for persistence.
    pub fn save_state(&self) -> Vec<u8> {
        self.state.encode()
    }

    /// Load sync state from bytes.
    pub fn load_state(&mut self, bytes: &[u8]) {
        self.state =
            sync::State::decode(bytes).unwrap_or_else(|_| sync::State::new());
    }

    /// Returns `true` when `generate_sync_message` would return `None`,
    /// indicating the sync is converged.
    pub fn is_synced(&mut self) -> bool {
        self.doc
            .sync()
            .generate_sync_message(&mut self.state)
            .is_none()
    }
}

impl Default for SyncSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use brewdio_core::beerjson_types::RecipeType;
    use crate::recipe::reconcile_to_automerge;

    fn sample_recipe() -> RecipeType {
        serde_json::from_str(
            r#"{
                "name": "Sync Test IPA",
                "type": "all grain",
                "author": "Tester",
                "batch_size": { "unit": "l", "value": 20.0 },
                "efficiency": {
                    "brewhouse": { "unit": "%", "value": 72.0 }
                },
                "ingredients": {
                    "fermentable_additions": [{
                        "name": "Pale Malt",
                        "type": "grain",
                        "amount": { "unit": "kg", "value": 5.0 },
                        "color": { "unit": "SRM", "value": 2.0 },
                        "yield": { "fine_grind": { "unit": "%", "value": 80.0 } }
                    }],
                    "hop_additions": [{
                        "name": "Cascade",
                        "timing": {
                            "duration": { "unit": "min", "value": 60 },
                            "use": "add_to_boil"
                        },
                        "amount": { "unit": "g", "value": 30.0 },
                        "form": "pellet",
                        "alpha_acid": { "unit": "%", "value": 5.5 }
                    }]
                }
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn sync_session_roundtrip() {
        let doc = RecipeDocument {
            id: "test-1".to_string(),
            name: "Test Beer".to_string(),
            recipe: sample_recipe(),
            is_deleted: false,
        };

        let am_bytes = reconcile_to_automerge(&doc, None);
        let session = SyncSession::from_doc_bytes(&am_bytes);
        let hydrated = session.hydrate();

        assert_eq!(hydrated.id, "test-1");
        assert_eq!(hydrated.name, "Test Beer");
    }

    #[test]
    fn two_sessions_converge() {
        // Session A has a recipe
        let doc_a = RecipeDocument {
            id: "recipe-1".to_string(),
            name: "Original Beer".to_string(),
            recipe: sample_recipe(),
            is_deleted: false,
        };
        let am_a = reconcile_to_automerge(&doc_a, None);
        let mut session_a = SyncSession::from_doc_bytes(&am_a);

        // Session B starts from same doc but with a different name
        let mut doc_b = doc_a.clone();
        doc_b.name = "Modified Beer".to_string();
        let am_b = reconcile_to_automerge(&doc_b, Some(&am_a));
        let mut session_b = SyncSession::from_doc_bytes(&am_b);

        // Exchange sync messages until convergence
        let mut msg = session_a.generate_sync_message();
        for _ in 0..20 {
            if msg.is_none() {
                break;
            }
            let reply = session_b
                .receive_sync_message(msg.as_ref().unwrap())
                .unwrap();
            if let Some(reply_bytes) = reply {
                msg = session_a.receive_sync_message(&reply_bytes).unwrap();
            } else {
                msg = None;
            }
        }

        // Both should now have the same document
        let hydrated_a = session_a.hydrate();
        let hydrated_b = session_b.hydrate();
        assert_eq!(hydrated_a.name, hydrated_b.name);
    }

    #[test]
    fn save_and_restore_state() {
        let doc = RecipeDocument {
            id: "test-1".to_string(),
            name: "Stateful Beer".to_string(),
            recipe: sample_recipe(),
            is_deleted: false,
        };
        let am_bytes = reconcile_to_automerge(&doc, None);

        let mut session = SyncSession::from_doc_bytes(&am_bytes);
        let state_bytes = session.save_state();
        let doc_bytes = session.save_doc();

        let restored = SyncSession::from_doc_and_state(&doc_bytes, &state_bytes);
        let hydrated = restored.hydrate();
        assert_eq!(hydrated.name, "Stateful Beer");
    }
}
