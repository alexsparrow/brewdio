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

    /// Merge changes from another Automerge document into this session's document.
    /// Preserves the sync state so the session can continue tracking what the peer has seen.
    pub fn merge_doc(&mut self, am_bytes: &[u8]) {
        let mut other = AutoCommit::load(am_bytes).expect("Failed to load Automerge doc for merge");
        self.doc
            .merge(&mut other)
            .expect("Failed to merge Automerge docs");
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
    use crate::automerge::reconcile_to_automerge;

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
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };

        let am_bytes = reconcile_to_automerge(&doc, None).unwrap();
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
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let am_a = reconcile_to_automerge(&doc_a, None).unwrap();
        let mut session_a = SyncSession::from_doc_bytes(&am_a);

        // Session B starts from same doc but with a different name
        let mut doc_b = doc_a.clone();
        doc_b.name = "Modified Beer".to_string();
        let am_b = reconcile_to_automerge(&doc_b, Some(&am_a)).unwrap();
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

    /// Simulates the pattern used by ws_handler/sync_worker:
    /// persistent SyncSessions are kept alive for the connection duration.
    /// Doc changes are saved to a simulated "DB" after each message,
    /// but the sessions themselves persist (preserving sync state in memory).
    #[test]
    fn persistent_sessions_converge_with_different_docs() {
        let doc_a = RecipeDocument {
            id: "recipe-1".to_string(),
            name: "Server Beer".to_string(),
            recipe: sample_recipe(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let doc_b = RecipeDocument {
            id: "recipe-1".to_string(),
            name: "Client Beer".to_string(),
            recipe: sample_recipe(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };

        // Both sides have the same recipe but with different names (concurrent edits)
        let am_a = reconcile_to_automerge(&doc_a, None).unwrap();
        let am_b = reconcile_to_automerge(&doc_b, None).unwrap();

        // Persistent sessions (kept alive for the connection)
        let mut session_a = SyncSession::from_doc_bytes(&am_a);
        let mut session_b = SyncSession::from_doc_bytes(&am_b);

        // Simulated "DB" storage (save_doc writes are for realism)
        let mut _db_doc_a = am_a;
        let mut _db_doc_b = am_b;

        // Server initiates
        let msg = session_a.generate_sync_message();
        assert!(msg.is_some(), "Server should have an initial message to send");
        let mut pending_to_b = msg;
        let mut pending_to_a: Option<Vec<u8>> = None;

        let mut rounds = 0;
        let max_rounds = 20;

        loop {
            rounds += 1;
            if rounds > max_rounds {
                panic!("Sync did not converge after {} rounds", max_rounds);
            }

            let mut made_progress = false;

            if let Some(msg_bytes) = pending_to_b.take() {
                made_progress = true;
                let reply = session_b.receive_sync_message(&msg_bytes).unwrap();
                _db_doc_b = session_b.save_doc();
                if reply.is_some() {
                    pending_to_a = reply;
                }
            }

            if let Some(msg_bytes) = pending_to_a.take() {
                made_progress = true;
                let reply = session_a.receive_sync_message(&msg_bytes).unwrap();
                _db_doc_a = session_a.save_doc();
                if reply.is_some() {
                    pending_to_b = reply;
                }
            }

            if !made_progress {
                break;
            }
        }

        eprintln!("Converged in {} rounds", rounds);
        assert!(rounds <= 10, "Should converge quickly, took {} rounds", rounds);

        // Verify both sides have the same document
        let hydrated_a = session_a.hydrate();
        let hydrated_b = session_b.hydrate();
        assert_eq!(hydrated_a.name, hydrated_b.name);
        assert_eq!(hydrated_a.id, hydrated_b.id);

        // Verify no more messages needed
        assert!(session_a.generate_sync_message().is_none(), "A should have nothing to send");
        assert!(session_b.generate_sync_message().is_none(), "B should have nothing to send");
    }

    /// Persistent sessions converge when both sides start with identical docs.
    /// This is the reconnection scenario: same data, fresh sync states.
    #[test]
    fn persistent_sessions_converge_when_already_identical() {
        let doc = RecipeDocument {
            id: "recipe-1".to_string(),
            name: "Same Beer".to_string(),
            recipe: sample_recipe(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let am_bytes = reconcile_to_automerge(&doc, None).unwrap();

        let mut session_a = SyncSession::from_doc_bytes(&am_bytes);
        let mut session_b = SyncSession::from_doc_bytes(&am_bytes);

        let msg = session_a.generate_sync_message();
        assert!(msg.is_some(), "Should send initial hash summary even when docs match");
        let mut pending_to_b = msg;
        let mut pending_to_a: Option<Vec<u8>> = None;

        let mut rounds = 0;
        loop {
            rounds += 1;
            if rounds > 20 {
                panic!("Sync did not converge after 20 rounds");
            }

            let mut made_progress = false;

            if let Some(msg_bytes) = pending_to_b.take() {
                made_progress = true;
                pending_to_a = session_b.receive_sync_message(&msg_bytes).unwrap();
            }

            if let Some(msg_bytes) = pending_to_a.take() {
                made_progress = true;
                pending_to_b = session_a.receive_sync_message(&msg_bytes).unwrap();
            }

            if !made_progress {
                break;
            }
        }

        eprintln!("Identical docs converged in {} rounds", rounds);
        assert!(rounds <= 5, "Identical docs should converge very quickly, took {}", rounds);
    }

    /// Tests that merge_doc correctly incorporates external changes
    /// while preserving sync state.
    #[test]
    fn merge_doc_incorporates_changes() {
        let doc_v1 = RecipeDocument {
            id: "recipe-1".to_string(),
            name: "Original Beer".to_string(),
            recipe: sample_recipe(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let am_v1 = reconcile_to_automerge(&doc_v1, None).unwrap();

        // Create a session and start syncing
        let mut session = SyncSession::from_doc_bytes(&am_v1);
        let _msg = session.generate_sync_message(); // advances sync state

        // Simulate a local edit: reconcile v2 on top of v1
        let doc_v2 = RecipeDocument {
            id: "recipe-1".to_string(),
            name: "Updated Beer".to_string(),
            recipe: sample_recipe(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let am_v2 = reconcile_to_automerge(&doc_v2, Some(&am_v1)).unwrap();

        // Merge the updated doc into the session
        session.merge_doc(&am_v2);
        let hydrated = session.hydrate();
        assert_eq!(hydrated.name, "Updated Beer");

        // Session should have new changes to send
        assert!(session.generate_sync_message().is_some(), "Should have new data after merge");
    }

    /// Simulates the dirty-check pattern: a persistent session gets new local
    /// edits merged in via merge_doc, then generates sync messages to push them.
    #[test]
    fn persistent_sessions_handle_mid_sync_local_edits() {
        let doc = RecipeDocument {
            id: "recipe-1".to_string(),
            name: "Original Beer".to_string(),
            recipe: sample_recipe(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let am_bytes = reconcile_to_automerge(&doc, None).unwrap();

        let mut session_a = SyncSession::from_doc_bytes(&am_bytes);
        let mut session_b = SyncSession::from_doc_bytes(&am_bytes);

        // First: converge the identical docs
        let mut msg = session_a.generate_sync_message();
        for _ in 0..20 {
            if msg.is_none() { break; }
            let reply = session_b.receive_sync_message(msg.as_ref().unwrap()).unwrap();
            if let Some(reply_bytes) = reply {
                msg = session_a.receive_sync_message(&reply_bytes).unwrap();
            } else {
                msg = None;
            }
        }
        assert!(session_a.generate_sync_message().is_none(), "Should be converged");

        // Now simulate a local edit on side A (dirty-check picks it up)
        let doc_v2 = RecipeDocument {
            id: "recipe-1".to_string(),
            name: "Edited Beer".to_string(),
            recipe: sample_recipe(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let am_v2 = reconcile_to_automerge(&doc_v2, Some(&am_bytes)).unwrap();
        session_a.merge_doc(&am_v2);

        // A should now have something to send
        let mut msg = session_a.generate_sync_message();
        assert!(msg.is_some(), "Should have edit to send");

        // Exchange until converged again
        for _ in 0..20 {
            if msg.is_none() { break; }
            let reply = session_b.receive_sync_message(msg.as_ref().unwrap()).unwrap();
            if let Some(reply_bytes) = reply {
                msg = session_a.receive_sync_message(&reply_bytes).unwrap();
            } else {
                msg = None;
            }
        }

        let hydrated_b = session_b.hydrate();
        assert_eq!(hydrated_b.name, "Edited Beer");
    }

    /// Verify that is_deleted stays false through the full sync + merge_doc + save_doc cycle.
    /// Reproduces the scenario where apply_remote_merge saves the session's doc back to DB.
    #[test]
    fn is_deleted_preserved_through_sync_and_save() {
        use crate::automerge::hydrate_from_automerge;

        let doc_a = RecipeDocument {
            id: "recipe-1".to_string(),
            name: "Server Beer".to_string(),
            recipe: sample_recipe(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let doc_b = RecipeDocument {
            id: "recipe-1".to_string(),
            name: "Client Beer".to_string(),
            recipe: sample_recipe(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };

        // Independently created docs (different actors)
        let am_a = reconcile_to_automerge(&doc_a, None).unwrap();
        let am_b = reconcile_to_automerge(&doc_b, None).unwrap();

        // Verify initial docs have is_deleted = false
        let h: RecipeDocument = hydrate_from_automerge(&am_a).unwrap();
        assert!(!h.is_deleted, "Server doc should start with is_deleted=false");
        let h: RecipeDocument = hydrate_from_automerge(&am_b).unwrap();
        assert!(!h.is_deleted, "Client doc should start with is_deleted=false");

        let mut session_a = SyncSession::from_doc_bytes(&am_a);
        let mut session_b = SyncSession::from_doc_bytes(&am_b);

        // Sync until converged
        let mut msg = session_a.generate_sync_message();
        for _ in 0..20 {
            if msg.is_none() { break; }
            let reply = session_b.receive_sync_message(msg.as_ref().unwrap()).unwrap();
            if let Some(reply_bytes) = reply {
                msg = session_a.receive_sync_message(&reply_bytes).unwrap();
            } else {
                msg = None;
            }
        }

        // Check is_deleted after sync on both sessions
        let hydrated_a = session_a.hydrate();
        let hydrated_b = session_b.hydrate();
        assert!(!hydrated_a.is_deleted, "Server should not be deleted after sync");
        assert!(!hydrated_b.is_deleted, "Client should not be deleted after sync");

        // Simulate what ws_handler does: save_doc -> apply_remote_merge (hydrate saved bytes)
        let saved_a = session_a.save_doc();
        let saved_b = session_b.save_doc();
        let h: RecipeDocument = hydrate_from_automerge(&saved_a).unwrap();
        assert!(!h.is_deleted, "Server saved doc should have is_deleted=false");
        let h: RecipeDocument = hydrate_from_automerge(&saved_b).unwrap();
        assert!(!h.is_deleted, "Client saved doc should have is_deleted=false");

        // Simulate merge_doc (dirty-check picks up DB changes)
        session_a.merge_doc(&saved_a);
        session_b.merge_doc(&saved_b);
        let hydrated_a = session_a.hydrate();
        let hydrated_b = session_b.hydrate();
        assert!(!hydrated_a.is_deleted, "Server should not be deleted after merge_doc");
        assert!(!hydrated_b.is_deleted, "Client should not be deleted after merge_doc");

        // Verify no more sync needed
        assert!(session_a.generate_sync_message().is_none());
        assert!(session_b.generate_sync_message().is_none());
    }

    /// Verify is_deleted stays false through the full DB-backed sync flow:
    /// create_recipe -> sync sessions -> apply_remote_merge -> verify DB state.
    #[test]
    fn is_deleted_preserved_through_db_sync() {
        use crate::automerge::hydrate_from_automerge;

        let conn_a = crate::connection_native::open(":memory:").unwrap();
        let conn_b = crate::connection_native::open(":memory:").unwrap();

        // Both sides create a recipe independently
        let row_a = crate::db::create_recipe(&conn_a, "Server Beer", &sample_recipe(), None, None).unwrap();
        let row_b = crate::db::create_recipe(&conn_b, "Client Beer", &sample_recipe(), None, None).unwrap();

        // Verify both start non-deleted
        assert!(!row_a.is_deleted);
        assert!(!row_b.is_deleted);

        // Now simulate the sync: server sends NewDoc with its recipe to client
        crate::db::apply_remote_merge(&conn_b, &row_a.id, &row_a.am_data).unwrap();
        // Client sends NewDoc with its recipe to server
        crate::db::apply_remote_merge(&conn_a, &row_b.id, &row_b.am_data).unwrap();

        // Verify both recipes on both sides are not deleted
        let a_on_a = crate::db::get_recipe(&conn_a, &row_a.id).unwrap().unwrap();
        let b_on_a = crate::db::get_recipe(&conn_a, &row_b.id).unwrap().unwrap();
        let a_on_b = crate::db::get_recipe(&conn_b, &row_a.id).unwrap().unwrap();
        let b_on_b = crate::db::get_recipe(&conn_b, &row_b.id).unwrap().unwrap();
        assert!(!a_on_a.is_deleted, "Server's recipe on server should not be deleted");
        assert!(!b_on_a.is_deleted, "Client's recipe on server should not be deleted");
        assert!(!a_on_b.is_deleted, "Server's recipe on client should not be deleted");
        assert!(!b_on_b.is_deleted, "Client's recipe on client should not be deleted");

        // Now simulate SyncDoc for a shared recipe (same ID, different docs)
        // This is the case where both sides already have the recipe
        let mut session_a = SyncSession::from_doc_bytes(&a_on_a.am_data);
        let mut session_b = SyncSession::from_doc_bytes(&a_on_b.am_data);

        let mut msg = session_a.generate_sync_message();
        for _ in 0..20 {
            if msg.is_none() { break; }
            let reply = session_b.receive_sync_message(msg.as_ref().unwrap()).unwrap();
            // Simulate apply_remote_merge after receiving
            let saved_b = session_b.save_doc();
            let h: RecipeDocument = hydrate_from_automerge(&saved_b).unwrap();
            assert!(!h.is_deleted, "Client hydrated doc should not be deleted during sync");

            if let Some(reply_bytes) = reply {
                msg = session_a.receive_sync_message(&reply_bytes).unwrap();
                let saved_a = session_a.save_doc();
                let h: RecipeDocument = hydrate_from_automerge(&saved_a).unwrap();
                assert!(!h.is_deleted, "Server hydrated doc should not be deleted during sync");
            } else {
                msg = None;
            }
        }
    }

    #[test]
    fn save_and_restore_state() {
        let doc = RecipeDocument {
            id: "test-1".to_string(),
            name: "Stateful Beer".to_string(),
            recipe: sample_recipe(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let am_bytes = reconcile_to_automerge(&doc, None).unwrap();

        let mut session = SyncSession::from_doc_bytes(&am_bytes);
        let state_bytes = session.save_state();
        let doc_bytes = session.save_doc();

        let restored = SyncSession::from_doc_and_state(&doc_bytes, &state_bytes);
        let hydrated = restored.hydrate();
        assert_eq!(hydrated.name, "Stateful Beer");
    }

    /// Helper: exchange sync messages between two sessions until convergence.
    /// Returns the number of rounds taken. Panics if not converged after max_rounds.
    fn converge(a: &mut SyncSession, b: &mut SyncSession, max_rounds: usize) -> usize {
        let mut msg = a.generate_sync_message();
        let mut rounds = 0;
        loop {
            rounds += 1;
            if rounds > max_rounds {
                panic!("Sync did not converge after {} rounds", max_rounds);
            }
            let mut made_progress = false;
            if let Some(msg_bytes) = msg.take() {
                made_progress = true;
                let reply = b.receive_sync_message(&msg_bytes).unwrap();
                if let Some(reply_bytes) = reply {
                    msg = a.receive_sync_message(&reply_bytes).unwrap();
                }
            }
            if !made_progress {
                break;
            }
        }
        rounds
    }

    /// Tests the exact save_doc → merge_doc cycle used by send_sync_for_doc.
    /// After convergence, merging the saved bytes back into the session should
    /// NOT produce new sync messages (the merge should be a no-op).
    #[test]
    fn merge_doc_with_own_save_is_stable() {
        let doc_a = RecipeDocument {
            id: "recipe-1".to_string(),
            name: "Original Beer".to_string(),
            recipe: sample_recipe(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let doc_b = RecipeDocument {
            id: "recipe-1".to_string(),
            name: "Edited Beer".to_string(),
            recipe: {
                let mut r = sample_recipe();
                r.batch_size.value = 25.0;
                r
            },
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };

        let am_a = reconcile_to_automerge(&doc_a, None).unwrap();
        let am_b = reconcile_to_automerge(&doc_b, None).unwrap();

        let mut session_a = SyncSession::from_doc_bytes(&am_a);
        let mut session_b = SyncSession::from_doc_bytes(&am_b);

        // Converge
        converge(&mut session_a, &mut session_b, 20);
        assert!(session_a.generate_sync_message().is_none());
        assert!(session_b.generate_sync_message().is_none());

        // Now simulate the send_sync_for_doc pattern:
        // save_doc() → write to "DB" → read from "DB" → merge_doc()
        let saved_a = session_a.save_doc();
        session_a.merge_doc(&saved_a);
        assert!(
            session_a.generate_sync_message().is_none(),
            "merge_doc(own save_doc bytes) should not produce new sync messages"
        );

        let saved_b = session_b.save_doc();
        session_b.merge_doc(&saved_b);
        assert!(
            session_b.generate_sync_message().is_none(),
            "merge_doc(own save_doc bytes) should not produce new sync messages"
        );
    }

    /// Simulates the server broadcast relay pattern with two clients.
    ///
    /// The server maintains one session per client. When client A sends a SyncDoc,
    /// the server processes it, saves to a shared "DB", then the OTHER client's
    /// handler reads the DB and calls merge_doc + generate_sync_message.
    ///
    /// This tests whether the cross-session merge_doc(other_session.save_doc())
    /// pattern converges or creates an infinite loop.
    #[test]
    fn server_relay_pattern_converges() {
        // All parties start from the same doc (simulating NewDoc exchange)
        let doc = RecipeDocument {
            id: "recipe-1".to_string(),
            name: "Original Beer".to_string(),
            recipe: sample_recipe(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let base_am = reconcile_to_automerge(&doc, None).unwrap();

        // Server has two sessions (one per client)
        let mut server_for_a = SyncSession::from_doc_bytes(&base_am);
        let mut server_for_b = SyncSession::from_doc_bytes(&base_am);
        // Clients have their own sessions
        let mut client_a = SyncSession::from_doc_bytes(&base_am);
        let mut client_b = SyncSession::from_doc_bytes(&base_am);
        // Shared server "DB"
        let mut db_am = base_am.clone();

        // Initial convergence: all parties sync to establish baseline
        converge(&mut client_a, &mut server_for_a, 20);
        converge(&mut client_b, &mut server_for_b, 20);

        // Client A edits the recipe (simulating update_recipe)
        let edited_doc = RecipeDocument {
            id: "recipe-1".to_string(),
            name: "Edited Beer".to_string(),
            recipe: {
                let mut r = sample_recipe();
                r.batch_size.value = 25.0;
                r
            },
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let edited_am = reconcile_to_automerge(&edited_doc, Some(&base_am)).unwrap();

        // Client A's dirty check: merge_doc picks up the edit
        client_a.merge_doc(&edited_am);

        // --- Simulate the server relay loop ---
        let max_total_rounds = 50;
        let mut total_rounds = 0;

        // Step 1: Client A sends SyncDoc to server
        let mut a_to_server = client_a.generate_sync_message();
        assert!(a_to_server.is_some(), "Client A should have edit to send");

        loop {
            total_rounds += 1;
            if total_rounds > max_total_rounds {
                panic!(
                    "Server relay pattern did not converge after {} rounds",
                    max_total_rounds
                );
            }

            let mut made_progress = false;

            // Server processes message from A (if any)
            if let Some(msg_bytes) = a_to_server.take() {
                made_progress = true;
                let reply = server_for_a.receive_sync_message(&msg_bytes).unwrap();
                // Server saves to DB
                db_am = server_for_a.save_doc();
                // Server sends reply to A (if any)
                if let Some(reply_bytes) = reply {
                    let a_reply = client_a.receive_sync_message(&reply_bytes).unwrap();
                    if a_reply.is_some() {
                        a_to_server = a_reply;
                    }
                }

                // Server broadcasts: session_for_b picks up DB changes
                server_for_b.merge_doc(&db_am);
                let to_b = server_for_b.generate_sync_message();
                if let Some(to_b_bytes) = to_b {
                    // Server sends to client B
                    let b_reply = client_b.receive_sync_message(&to_b_bytes).unwrap();
                    if let Some(b_reply_bytes) = b_reply {
                        // Server processes B's reply
                        let b_reply2 = server_for_b
                            .receive_sync_message(&b_reply_bytes)
                            .unwrap();
                        db_am = server_for_b.save_doc();

                        // Server broadcasts again (notify all)
                        // Self-broadcast for B's handler:
                        server_for_b.merge_doc(&db_am);
                        // Cross-broadcast for A's handler:
                        server_for_a.merge_doc(&db_am);
                        let to_a = server_for_a.generate_sync_message();
                        if to_a.is_some() {
                            a_to_server = to_a; // will be processed next round
                            // This is suspicious — A already sent its edit,
                            // the cross-broadcast shouldn't produce new messages
                        }

                        if let Some(reply2_bytes) = b_reply2 {
                            let b_reply3 =
                                client_b.receive_sync_message(&reply2_bytes).unwrap();
                            if let Some(b_reply3_bytes) = b_reply3 {
                                // Continue B's sync
                                let _ = server_for_b
                                    .receive_sync_message(&b_reply3_bytes)
                                    .unwrap();
                                db_am = server_for_b.save_doc();
                            }
                        }
                    }
                }

                // Self-broadcast for A's handler:
                server_for_a.merge_doc(&db_am);
                let self_msg = server_for_a.generate_sync_message();
                if let Some(self_msg_bytes) = self_msg {
                    // This means the self-broadcast produced a message — problematic
                    eprintln!(
                        "WARNING: self-broadcast produced message at round {}",
                        total_rounds
                    );
                    let a_reply = client_a.receive_sync_message(&self_msg_bytes).unwrap();
                    if a_reply.is_some() {
                        a_to_server = a_reply;
                    }
                }
            }

            if !made_progress {
                break;
            }
        }

        eprintln!("Server relay converged in {} rounds", total_rounds);

        // Verify all parties have the same document
        let hydrated_a = client_a.hydrate();
        let hydrated_b = client_b.hydrate();
        let hydrated_server_a = server_for_a.hydrate();
        let hydrated_server_b = server_for_b.hydrate();

        assert_eq!(hydrated_a.name, hydrated_b.name, "Clients should agree on name");
        assert_eq!(
            hydrated_server_a.name, hydrated_server_b.name,
            "Server sessions should agree on name"
        );
        assert_eq!(
            hydrated_a.name, hydrated_server_a.name,
            "Client and server should agree on name"
        );
        assert_eq!(
            hydrated_a.recipe.batch_size.value,
            25.0,
            "Client A's edit should be reflected everywhere"
        );
        assert_eq!(
            hydrated_b.recipe.batch_size.value,
            25.0,
            "Client B should have the edited batch_size"
        );
    }

    /// Tests the full DB-backed sync flow: create_recipe → edit → sync via
    /// the exact patterns used by sync_worker.rs and ws_handler.rs.
    /// Uses real in-memory SQLite databases.
    #[test]
    fn db_backed_sync_converges() {
        use crate::automerge::hydrate_from_automerge;

        // Three "nodes": client_a DB, server DB, client_b DB
        let conn_a = crate::connection_native::open(":memory:").unwrap();
        let conn_server = crate::connection_native::open(":memory:").unwrap();
        let conn_b = crate::connection_native::open(":memory:").unwrap();

        // Client A creates a recipe
        let row = crate::db::create_recipe(
            &conn_a,
            "Original Beer",
            &sample_recipe(),
            None,
            None,
        )
        .unwrap();
        let recipe_id = row.id.clone();

        // Simulate NewDoc to server and client B
        crate::db::apply_remote_merge_doc(
            &conn_server,
            crate::protocol::DocType::Recipe,
            &recipe_id,
            &row.am_data,
        )
        .unwrap();
        crate::db::apply_remote_merge_doc(
            &conn_b,
            crate::protocol::DocType::Recipe,
            &recipe_id,
            &row.am_data,
        )
        .unwrap();

        // Verify all three have the recipe
        let r_a = crate::db::get_recipe(&conn_a, &recipe_id).unwrap().unwrap();
        let r_s = crate::db::get_recipe(&conn_server, &recipe_id).unwrap().unwrap();
        let r_b = crate::db::get_recipe(&conn_b, &recipe_id).unwrap().unwrap();
        assert_eq!(r_a.name, "Original Beer");
        assert_eq!(r_s.name, "Original Beer");
        assert_eq!(r_b.name, "Original Beer");

        // Create persistent sessions (as sync_worker/ws_handler would)
        let mut server_for_a = SyncSession::from_doc_bytes(&r_s.am_data);
        let mut server_for_b = SyncSession::from_doc_bytes(&r_s.am_data);
        let mut client_a_session = SyncSession::from_doc_bytes(&r_a.am_data);
        let mut client_b_session = SyncSession::from_doc_bytes(&r_b.am_data);

        // Initial convergence
        converge(&mut client_a_session, &mut server_for_a, 20);
        converge(&mut client_b_session, &mut server_for_b, 20);

        // Client A edits the recipe
        let mut edited_recipe = sample_recipe();
        edited_recipe.batch_size.value = 25.0;
        crate::db::update_recipe(&conn_a, &recipe_id, "Edited Beer", &edited_recipe, None).unwrap();

        // Client A dirty check: send_sync_for_doc pattern
        let a_am = crate::db::get_doc_am_data(&conn_a, crate::protocol::DocType::Recipe, &recipe_id)
            .unwrap()
            .unwrap();
        client_a_session.merge_doc(&a_am);

        let msg = client_a_session.generate_sync_message();
        assert!(msg.is_some(), "Client A should have edit to sync");

        // --- Simulate the sync loop with DB reads/writes ---
        let mut pending_a_to_server = msg;
        let mut pending_b_to_server: Option<Vec<u8>> = None;
        let max_rounds = 30;

        for round in 0..max_rounds {
            let mut progress = false;

            // Server processes message from A
            if let Some(msg_bytes) = pending_a_to_server.take() {
                progress = true;
                let reply = server_for_a.receive_sync_message(&msg_bytes).unwrap();
                // Server saves to DB (handle_incoming pattern)
                let saved = server_for_a.save_doc();
                crate::db::apply_remote_merge_doc(
                    &conn_server,
                    crate::protocol::DocType::Recipe,
                    &recipe_id,
                    &saved,
                )
                .unwrap();

                // Send reply to A if needed
                if let Some(reply_bytes) = reply {
                    let a_reply = client_a_session
                        .receive_sync_message(&reply_bytes)
                        .unwrap();
                    // Client A saves to DB
                    let a_saved = client_a_session.save_doc();
                    crate::db::apply_remote_merge_doc(
                        &conn_a,
                        crate::protocol::DocType::Recipe,
                        &recipe_id,
                        &a_saved,
                    )
                    .unwrap();
                    if a_reply.is_some() {
                        pending_a_to_server = a_reply;
                    }
                }

                // Server broadcasts to B: send_sync_for_doc pattern
                let db_am = crate::db::get_doc_am_data(
                    &conn_server,
                    crate::protocol::DocType::Recipe,
                    &recipe_id,
                )
                .unwrap()
                .unwrap();
                server_for_b.merge_doc(&db_am);
                if let Some(to_b) = server_for_b.generate_sync_message() {
                    pending_b_to_server =
                        Some(to_b); // will be "sent" to B and B will reply
                }
            }

            // "Send" server's message to B, get B's reply
            if let Some(to_b_bytes) = pending_b_to_server.take() {
                progress = true;
                let b_reply = client_b_session
                    .receive_sync_message(&to_b_bytes)
                    .unwrap();
                // Client B saves to DB
                let b_saved = client_b_session.save_doc();
                crate::db::apply_remote_merge_doc(
                    &conn_b,
                    crate::protocol::DocType::Recipe,
                    &recipe_id,
                    &b_saved,
                )
                .unwrap();

                if let Some(b_reply_bytes) = b_reply {
                    // Server processes B's reply (as incoming SyncDoc)
                    let reply2 = server_for_b
                        .receive_sync_message(&b_reply_bytes)
                        .unwrap();
                    let saved = server_for_b.save_doc();
                    crate::db::apply_remote_merge_doc(
                        &conn_server,
                        crate::protocol::DocType::Recipe,
                        &recipe_id,
                        &saved,
                    )
                    .unwrap();

                    if let Some(reply2_bytes) = reply2 {
                        // Send reply2 to B
                        pending_b_to_server = Some(reply2_bytes);
                    }

                    // Server broadcasts to A (cross-broadcast)
                    let db_am = crate::db::get_doc_am_data(
                        &conn_server,
                        crate::protocol::DocType::Recipe,
                        &recipe_id,
                    )
                    .unwrap()
                    .unwrap();
                    server_for_a.merge_doc(&db_am);
                    if let Some(to_a) = server_for_a.generate_sync_message() {
                        eprintln!(
                            "Round {}: cross-broadcast to A produced a message!",
                            round
                        );
                        pending_a_to_server = Some(to_a);
                    }
                }
            }

            if !progress {
                eprintln!("DB-backed sync converged in {} rounds", round);
                break;
            }
            if round == max_rounds - 1 {
                panic!(
                    "DB-backed sync did not converge after {} rounds",
                    max_rounds
                );
            }
        }

        // Verify all DBs have the correct values
        let final_a = crate::db::get_recipe(&conn_a, &recipe_id).unwrap().unwrap();
        let final_s = crate::db::get_recipe(&conn_server, &recipe_id).unwrap().unwrap();
        let final_b = crate::db::get_recipe(&conn_b, &recipe_id).unwrap().unwrap();

        // Check recipe JSON was updated correctly via apply_remote_merge
        let recipe_b: RecipeType = serde_json::from_str(&final_b.recipe).unwrap();
        assert_eq!(
            recipe_b.batch_size.value, 25.0,
            "Client B's DB should have the edited batch_size"
        );
        assert_eq!(
            final_b.name, "Edited Beer",
            "Client B's DB should have the edited name"
        );
        assert_eq!(
            final_s.name, "Edited Beer",
            "Server DB should have the edited name"
        );

        // Verify automerge docs are consistent
        let h_a: RecipeDocument = hydrate_from_automerge(&final_a.am_data).unwrap();
        let h_s: RecipeDocument = hydrate_from_automerge(&final_s.am_data).unwrap();
        let h_b: RecipeDocument = hydrate_from_automerge(&final_b.am_data).unwrap();
        assert_eq!(h_a.name, h_b.name, "AM docs should agree on name");
        assert_eq!(h_a.name, h_s.name, "AM docs should agree on name");
        assert_eq!(
            h_b.recipe.batch_size.value, 25.0,
            "AM doc batch_size should be updated"
        );
    }
}
