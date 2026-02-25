use brewdio_core::beerjson_types::{EquipmentType, RecipeType};

use crate::batch::{self, BatchDocument};
use crate::connection::{Connection, DbError, Value};
use crate::automerge::{extract_fields_from_automerge, extract_string_field_from_automerge, hydrate_from_automerge, new_ulid, reconcile_to_automerge};
use crate::protocol::DocType;
use crate::recipe::{RecipeDocument, RecipeRow};
use crate::settings::{self, SettingsDocument};

fn row_from_query(row: &dyn crate::connection::Row) -> RecipeRow {
    RecipeRow {
        id: row.get_text(0),
        name: row.get_text(1),
        recipe: row.get_text(2),
        equipment: row.get_optional_text(3),
        am_data: row.get_blob(4),
        is_deleted: row.get_bool(5),
        is_dirty: row.get_bool(6),
    }
}

/// Create a new recipe and return its row.
pub fn create_recipe(
    conn: &(impl Connection + ?Sized),
    name: &str,
    recipe: &RecipeType,
    equipment: Option<&EquipmentType>,
) -> Result<RecipeRow, DbError> {
    let id = new_ulid();
    let recipe_json = serde_json::to_string(recipe).expect("Failed to serialize recipe");
    let equipment_json = equipment.map(|eq| serde_json::to_string(eq).expect("Failed to serialize equipment"));

    let doc = RecipeDocument {
        id: id.clone(),
        name: name.to_string(),
        recipe: recipe.clone(),
        equipment: equipment.cloned(),
        is_deleted: false,
    };
    let am_data = reconcile_to_automerge(&doc, None).map_err(|e| DbError(e))?;

    conn.execute(
        "INSERT INTO recipe (id, name, recipe, equipment, am_data, is_deleted, is_dirty) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        &[
            Value::Text(&id),
            Value::Text(name),
            Value::Text(&recipe_json),
            Value::OptionalText(equipment_json.as_deref()),
            Value::Blob(&am_data),
            Value::Bool(false),
            Value::Bool(true),
        ],
    )?;

    Ok(RecipeRow {
        id,
        name: name.to_string(),
        recipe: recipe_json,
        equipment: equipment_json,
        am_data,
        is_deleted: false,
        is_dirty: true,
    })
}

/// Get a recipe by ID.
pub fn get_recipe(conn: &(impl Connection + ?Sized), id: &str) -> Result<Option<RecipeRow>, DbError> {
    conn.query_one(
        "SELECT id, name, recipe, equipment, am_data, is_deleted, is_dirty FROM recipe WHERE id = ?1",
        &[Value::Text(id)],
        row_from_query,
    )
}

/// List all non-deleted recipes.
pub fn list_recipes(conn: &(impl Connection + ?Sized)) -> Result<Vec<RecipeRow>, DbError> {
    conn.query_map(
        "SELECT id, name, recipe, equipment, am_data, is_deleted, is_dirty FROM recipe WHERE is_deleted = FALSE",
        &[],
        row_from_query,
    )
}

/// Update a recipe's name and content, re-reconciling into the existing Automerge document.
/// Equipment is preserved from the existing row (use `set_recipe_equipment` to change it).
pub fn update_recipe(
    conn: &(impl Connection + ?Sized),
    id: &str,
    name: &str,
    recipe: &RecipeType,
    equipment: Option<&EquipmentType>,
) -> Result<(), DbError> {
    let recipe_json = serde_json::to_string(recipe).expect("Failed to serialize recipe");

    let existing = get_recipe(conn, id)?;
    let existing_am = existing.as_ref().map(|r| r.am_data.as_slice());

    // Preserve existing equipment when not explicitly provided
    let effective_equipment: Option<EquipmentType> = match equipment {
        Some(eq) => Some(eq.clone()),
        None => existing
            .as_ref()
            .and_then(|r| r.equipment.as_ref())
            .and_then(|json| serde_json::from_str(json).ok()),
    };
    let equipment_json = effective_equipment.as_ref().map(|eq| {
        serde_json::to_string(eq).expect("Failed to serialize equipment")
    });

    let doc = RecipeDocument {
        id: id.to_string(),
        name: name.to_string(),
        recipe: recipe.clone(),
        equipment: effective_equipment,
        is_deleted: false,
    };
    let am_data = reconcile_to_automerge(&doc, existing_am).map_err(|e| DbError(e))?;

    conn.execute(
        "UPDATE recipe SET name = ?1, recipe = ?2, equipment = ?3, am_data = ?4, is_dirty = TRUE WHERE id = ?5",
        &[
            Value::Text(name),
            Value::Text(&recipe_json),
            Value::OptionalText(equipment_json.as_deref()),
            Value::Blob(&am_data),
            Value::Text(id),
        ],
    )?;

    Ok(())
}

/// Set or clear a recipe's equipment profile without changing the recipe content.
pub fn set_recipe_equipment(
    conn: &(impl Connection + ?Sized),
    id: &str,
    equipment: Option<&EquipmentType>,
) -> Result<(), DbError> {
    let existing = get_recipe(conn, id)?;
    let row = existing.ok_or_else(|| DbError(format!("Recipe {} not found", id)))?;
    let existing_am = row.am_data.as_slice();

    let recipe: RecipeType = serde_json::from_str(&row.recipe)
        .map_err(|e| DbError(e.to_string()))?;
    let equipment_json = equipment.map(|eq| {
        serde_json::to_string(eq).expect("Failed to serialize equipment")
    });

    let doc = RecipeDocument {
        id: id.to_string(),
        name: row.name.clone(),
        recipe,
        equipment: equipment.cloned(),
        is_deleted: row.is_deleted,
    };
    let am_data = reconcile_to_automerge(&doc, Some(existing_am)).map_err(|e| DbError(e))?;

    conn.execute(
        "UPDATE recipe SET equipment = ?1, am_data = ?2, is_dirty = TRUE WHERE id = ?3",
        &[
            Value::OptionalText(equipment_json.as_deref()),
            Value::Blob(&am_data),
            Value::Text(id),
        ],
    )?;

    Ok(())
}

/// List all soft-deleted recipes.
pub fn list_deleted_recipes(conn: &(impl Connection + ?Sized)) -> Result<Vec<RecipeRow>, DbError> {
    conn.query_map(
        "SELECT id, name, recipe, equipment, am_data, is_deleted, is_dirty FROM recipe WHERE is_deleted = TRUE",
        &[],
        row_from_query,
    )
}

/// List all recipes (including deleted).
pub fn list_all_recipes(conn: &(impl Connection + ?Sized)) -> Result<Vec<RecipeRow>, DbError> {
    conn.query_map(
        "SELECT id, name, recipe, equipment, am_data, is_deleted, is_dirty FROM recipe",
        &[],
        row_from_query,
    )
}

/// Restore a soft-deleted recipe by setting `is_deleted = false`.
pub fn undelete_recipe(conn: &(impl Connection + ?Sized), id: &str) -> Result<(), DbError> {
    let existing = get_recipe(conn, id)?;
    if let Some(row) = existing {
        let mut doc = row.to_document().expect("Failed to deserialize recipe");
        doc.is_deleted = false;
        let am_data = reconcile_to_automerge(&doc, Some(&row.am_data)).map_err(|e| DbError(e))?;

        conn.execute(
            "UPDATE recipe SET is_deleted = FALSE, is_dirty = TRUE, am_data = ?1 WHERE id = ?2",
            &[Value::Blob(&am_data), Value::Text(id)],
        )?;
    }

    Ok(())
}

/// Soft-delete a recipe by setting `is_deleted = true`.
pub fn delete_recipe(conn: &(impl Connection + ?Sized), id: &str) -> Result<(), DbError> {
    let existing = get_recipe(conn, id)?;
    if let Some(row) = existing {
        let mut doc = row.to_document().expect("Failed to deserialize recipe");
        doc.is_deleted = true;
        let am_data = reconcile_to_automerge(&doc, Some(&row.am_data)).map_err(|e| DbError(e))?;

        conn.execute(
            "UPDATE recipe SET is_deleted = TRUE, is_dirty = TRUE, am_data = ?1 WHERE id = ?2",
            &[Value::Blob(&am_data), Value::Text(id)],
        )?;
    }

    Ok(())
}

// --- Sync-related functions ---

/// List all recipes that have local changes not yet synced.
pub fn list_dirty_recipes(conn: &(impl Connection + ?Sized)) -> Result<Vec<RecipeRow>, DbError> {
    conn.query_map(
        "SELECT id, name, recipe, equipment, am_data, is_deleted, is_dirty FROM recipe WHERE is_dirty = TRUE",
        &[],
        row_from_query,
    )
}

/// Clear the dirty flag for a recipe (after successful sync).
pub fn clear_dirty(conn: &(impl Connection + ?Sized), id: &str) -> Result<(), DbError> {
    conn.execute(
        "UPDATE recipe SET is_dirty = FALSE WHERE id = ?1",
        &[Value::Text(id)],
    )
}

/// List all recipe IDs (including deleted).
pub fn list_all_recipe_ids(conn: &(impl Connection + ?Sized)) -> Result<Vec<String>, DbError> {
    conn.query_map("SELECT id FROM recipe", &[], |row| row.get_text(0))
}

/// Get the stored Automerge sync state for a (recipe, peer) pair.
pub fn get_sync_state(
    conn: &(impl Connection + ?Sized),
    recipe_id: &str,
    peer_id: &str,
) -> Result<Option<Vec<u8>>, DbError> {
    conn.query_one(
        "SELECT state FROM sync_state WHERE recipe_id = ?1 AND peer_id = ?2",
        &[Value::Text(recipe_id), Value::Text(peer_id)],
        |row| row.get_blob(0),
    )
}

/// Save (upsert) Automerge sync state for a (recipe, peer) pair.
pub fn save_sync_state(
    conn: &(impl Connection + ?Sized),
    recipe_id: &str,
    peer_id: &str,
    state: &[u8],
) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO sync_state (recipe_id, peer_id, state) VALUES (?1, ?2, ?3)
         ON CONFLICT (recipe_id, peer_id) DO UPDATE SET state = excluded.state",
        &[Value::Text(recipe_id), Value::Text(peer_id), Value::Blob(state)],
    )
}

/// Clear all sync states for a given peer. Called at connection start to avoid
/// stale state from a previous session causing sync loops.
pub fn clear_sync_states_for_peer(conn: &(impl Connection + ?Sized), peer_id: &str) -> Result<(), DbError> {
    conn.execute(
        "DELETE FROM sync_state WHERE peer_id = ?1",
        &[Value::Text(peer_id)],
    )
}

/// Apply a remotely-merged Automerge document. Updates the JSON and name columns
/// from the merged AM state. Does NOT set `is_dirty` since the change came from the server.
/// If the recipe doesn't exist locally, inserts it with `is_dirty = FALSE`.
///
/// If full autosurgeon hydration fails (e.g. due to untagged enum types),
/// falls back to extracting `name` and `is_deleted` directly from the Automerge
/// document and still stores the `am_data` so CRDT sync can continue.
pub fn apply_remote_merge(
    conn: &(impl Connection + ?Sized),
    recipe_id: &str,
    am_data: &[u8],
) -> Result<(), DbError> {
    let existing = get_recipe(conn, recipe_id)?;

    match hydrate_from_automerge::<RecipeDocument>(am_data) {
        Ok(doc) => {
            let recipe_json = serde_json::to_string(&doc.recipe).expect("Failed to serialize recipe");
            let equipment_json = doc.equipment.as_ref().map(|eq| {
                serde_json::to_string(eq).expect("Failed to serialize equipment")
            });
            if existing.is_some() {
                conn.execute(
                    "UPDATE recipe SET name = ?1, recipe = ?2, equipment = ?3, am_data = ?4, is_deleted = ?5 WHERE id = ?6",
                    &[
                        Value::Text(&doc.name),
                        Value::Text(&recipe_json),
                        Value::OptionalText(equipment_json.as_deref()),
                        Value::Blob(am_data),
                        Value::Bool(doc.is_deleted),
                        Value::Text(recipe_id),
                    ],
                )?;
            } else {
                conn.execute(
                    "INSERT INTO recipe (id, name, recipe, equipment, am_data, is_deleted, is_dirty) VALUES (?1, ?2, ?3, ?4, ?5, ?6, FALSE)",
                    &[
                        Value::Text(recipe_id),
                        Value::Text(&doc.name),
                        Value::Text(&recipe_json),
                        Value::OptionalText(equipment_json.as_deref()),
                        Value::Blob(am_data),
                        Value::Bool(doc.is_deleted),
                    ],
                )?;
            }
        }
        Err(_) => {
            // Fallback: extract name/is_deleted directly from automerge doc
            let (name, is_deleted) = extract_fields_from_automerge(am_data);
            if existing.is_some() {
                conn.execute(
                    "UPDATE recipe SET name = ?1, am_data = ?2, is_deleted = ?3 WHERE id = ?4",
                    &[Value::Text(&name), Value::Blob(am_data), Value::Bool(is_deleted), Value::Text(recipe_id)],
                )?;
            } else {
                conn.execute(
                    "INSERT INTO recipe (id, name, recipe, am_data, is_deleted, is_dirty) VALUES (?1, ?2, ?3, ?4, ?5, FALSE)",
                    &[Value::Text(recipe_id), Value::Text(&name), Value::Text("{}"), Value::Blob(am_data), Value::Bool(is_deleted)],
                )?;
            }
        }
    }

    Ok(())
}

// --- Generic sync dispatch functions ---

/// Get `am_data` for any document type by ID.
pub fn get_doc_am_data(
    conn: &(impl Connection + ?Sized),
    doc_type: DocType,
    id: &str,
) -> Result<Option<Vec<u8>>, DbError> {
    match doc_type {
        DocType::Recipe => get_recipe(conn, id).map(|r| r.map(|r| r.am_data)),
        DocType::Batch => batch::get_batch(conn, id).map(|r| r.map(|r| r.am_data)),
        DocType::Settings => settings::get_settings(conn).map(|r| {
            r.map(|r| r.am_data).filter(|am| !am.is_empty())
        }),
    }
}

/// Clear the dirty flag for any document type.
pub fn clear_dirty_doc(
    conn: &(impl Connection + ?Sized),
    doc_type: DocType,
    id: &str,
) -> Result<(), DbError> {
    match doc_type {
        DocType::Recipe => clear_dirty(conn, id),
        DocType::Batch => batch::clear_dirty_batch(conn, id),
        DocType::Settings => settings::clear_dirty_settings(conn, id),
    }
}

/// List all dirty documents across all types.
pub fn list_dirty_docs(
    conn: &(impl Connection + ?Sized),
) -> Result<Vec<(DocType, String)>, DbError> {
    let mut result = Vec::new();
    for row in list_dirty_recipes(conn)? {
        result.push((DocType::Recipe, row.id));
    }
    for row in batch::list_dirty_batches(conn)? {
        result.push((DocType::Batch, row.id));
    }
    for row in settings::list_dirty_settings(conn)? {
        result.push((DocType::Settings, row.id));
    }
    Ok(result)
}

/// List all document IDs for a given type (including deleted).
pub fn list_all_doc_ids(
    conn: &(impl Connection + ?Sized),
    doc_type: DocType,
) -> Result<Vec<String>, DbError> {
    match doc_type {
        DocType::Recipe => list_all_recipe_ids(conn),
        DocType::Batch => batch::list_all_batch_ids(conn),
        DocType::Settings => settings::list_all_settings_ids(conn),
    }
}

/// Apply a remotely-merged Automerge document, dispatching to the right type.
pub fn apply_remote_merge_doc(
    conn: &(impl Connection + ?Sized),
    doc_type: DocType,
    id: &str,
    am_data: &[u8],
) -> Result<(), DbError> {
    match doc_type {
        DocType::Recipe => apply_remote_merge(conn, id, am_data),
        DocType::Batch => apply_remote_merge_batch(conn, id, am_data),
        DocType::Settings => apply_remote_merge_settings(conn, id, am_data),
    }
}

/// Apply a remotely-merged Automerge batch document.
pub fn apply_remote_merge_batch(
    conn: &(impl Connection + ?Sized),
    batch_id: &str,
    am_data: &[u8],
) -> Result<(), DbError> {
    let existing = batch::get_batch(conn, batch_id)?;

    match hydrate_from_automerge::<BatchDocument>(am_data) {
        Ok(doc) => {
            let data_json =
                serde_json::to_string(&doc.data).expect("Failed to serialize batch data");
            if existing.is_some() {
                conn.execute(
                    "UPDATE batch SET name = ?1, data = ?2, am_data = ?3, is_deleted = ?4 WHERE id = ?5",
                    &[
                        Value::Text(&doc.name),
                        Value::Text(&data_json),
                        Value::Blob(am_data),
                        Value::Bool(doc.is_deleted),
                        Value::Text(batch_id),
                    ],
                )?;
            } else {
                conn.execute(
                    "INSERT INTO batch (id, name, recipe_id, data, am_data, is_deleted, is_dirty) VALUES (?1, ?2, ?3, ?4, ?5, ?6, FALSE)",
                    &[
                        Value::Text(batch_id),
                        Value::Text(&doc.name),
                        Value::Text(&doc.recipe_id),
                        Value::Text(&data_json),
                        Value::Blob(am_data),
                        Value::Bool(doc.is_deleted),
                    ],
                )?;
            }
        }
        Err(_) => {
            if existing.is_some() {
                conn.execute(
                    "UPDATE batch SET am_data = ?1 WHERE id = ?2",
                    &[Value::Blob(am_data), Value::Text(batch_id)],
                )?;
            } else {
                conn.execute(
                    "INSERT INTO batch (id, name, recipe_id, data, am_data, is_deleted, is_dirty) VALUES (?1, '', '', '{}', ?2, FALSE, FALSE)",
                    &[Value::Text(batch_id), Value::Blob(am_data)],
                )?;
            }
        }
    }

    Ok(())
}

/// Apply a remotely-merged Automerge settings document.
pub fn apply_remote_merge_settings(
    conn: &(impl Connection + ?Sized),
    settings_id: &str,
    am_data: &[u8],
) -> Result<(), DbError> {
    let existing = settings::get_settings(conn)?;

    // Try full hydration first; on failure, extract `data` field directly from the
    // automerge doc (handles schema changes like removed `is_dirty` field).
    let data_str = match hydrate_from_automerge::<SettingsDocument>(am_data) {
        Ok(doc) => doc.data,
        Err(_) => {
            // Fallback: extract the "data" key directly from the automerge doc
            // (handles schema changes like removed fields)
            extract_string_field_from_automerge(am_data, "data").unwrap_or_else(|| {
                // If we already have good data, keep it rather than overwriting with empty
                if let Some(ref row) = existing {
                    if row.data != "{}" {
                        return row.data.clone();
                    }
                }
                "{}".to_string()
            })
        }
    };

    if existing.is_some() {
        conn.execute(
            "UPDATE settings SET data = ?1, am_data = ?2 WHERE id = ?3",
            &[Value::Text(&data_str), Value::Blob(am_data), Value::Text(settings_id)],
        )?;
    } else {
        conn.execute(
            "INSERT INTO settings (id, data, am_data, is_dirty) VALUES (?1, ?2, ?3, FALSE)",
            &[Value::Text(settings_id), Value::Text(&data_str), Value::Blob(am_data)],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use brewdio_core::beerjson_types::RecipeType;

    fn test_conn() -> rusqlite::Connection {
        crate::connection_native::open(":memory:").unwrap()
    }

    fn sample_recipe() -> RecipeType {
        serde_json::from_str(
            r#"{
                "name": "DB Test IPA",
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
    fn crud_operations() {
        let conn = test_conn();

        // Create
        let row = create_recipe(&conn, "Test IPA", &sample_recipe(), None).unwrap();
        assert_eq!(row.name, "Test IPA");
        assert!(!row.id.is_empty());
        assert!(row.is_dirty);
        assert!(row.equipment.is_none());

        // Get
        let fetched = get_recipe(&conn, &row.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Test IPA");
        assert!(fetched.is_dirty);
        assert!(fetched.equipment.is_none());

        // List
        let recipes = list_recipes(&conn).unwrap();
        assert_eq!(recipes.len(), 1);

        // Update
        let mut updated_recipe = sample_recipe();
        updated_recipe.name = "Updated IPA".to_string();
        update_recipe(&conn, &row.id, "Updated IPA", &updated_recipe, None).unwrap();
        let fetched = get_recipe(&conn, &row.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Updated IPA");
        assert!(fetched.is_dirty);

        // Soft-delete
        delete_recipe(&conn, &row.id).unwrap();
        let recipes = list_recipes(&conn).unwrap();
        assert_eq!(recipes.len(), 0);

        // Still exists in DB, just marked as deleted
        let fetched = get_recipe(&conn, &row.id).unwrap().unwrap();
        assert!(fetched.is_deleted);
        assert!(fetched.is_dirty);
    }

    #[test]
    fn crud_with_equipment() {
        let conn = test_conn();
        let equipment = brewdio_core::data::equipment()[0].clone();

        // Create with equipment
        let row = create_recipe(&conn, "Equipped IPA", &sample_recipe(), Some(&equipment)).unwrap();
        assert!(row.equipment.is_some());

        // Get
        let fetched = get_recipe(&conn, &row.id).unwrap().unwrap();
        assert!(fetched.equipment.is_some());
        let doc = fetched.to_document().unwrap();
        assert_eq!(doc.equipment.as_ref().unwrap().name, "Default Setup");

        // Update with different equipment
        let biab = brewdio_core::data::equipment()[1].clone();
        update_recipe(&conn, &row.id, "BIAB IPA", &sample_recipe(), Some(&biab)).unwrap();
        let fetched = get_recipe(&conn, &row.id).unwrap().unwrap();
        let doc = fetched.to_document().unwrap();
        assert_eq!(doc.equipment.as_ref().unwrap().name, "BIAB (No Sparge)");

        // Update without equipment preserves existing
        update_recipe(&conn, &row.id, "Still Equipped", &sample_recipe(), None).unwrap();
        let fetched = get_recipe(&conn, &row.id).unwrap().unwrap();
        let doc = fetched.to_document().unwrap();
        assert_eq!(doc.equipment.as_ref().unwrap().name, "BIAB (No Sparge)");

        // Explicitly remove equipment
        set_recipe_equipment(&conn, &row.id, None).unwrap();
        let fetched = get_recipe(&conn, &row.id).unwrap().unwrap();
        assert!(fetched.equipment.is_none());
    }

    #[test]
    fn dirty_tracking() {
        let conn = test_conn();

        // Create sets dirty
        let row = create_recipe(&conn, "Dirty Test", &sample_recipe(), None).unwrap();
        let dirty = list_dirty_recipes(&conn).unwrap();
        assert_eq!(dirty.len(), 1);

        // Clear dirty
        clear_dirty(&conn, &row.id).unwrap();
        let dirty = list_dirty_recipes(&conn).unwrap();
        assert_eq!(dirty.len(), 0);

        // Update sets dirty again
        update_recipe(&conn, &row.id, "Still Dirty", &sample_recipe(), None).unwrap();
        let dirty = list_dirty_recipes(&conn).unwrap();
        assert_eq!(dirty.len(), 1);
    }

    #[test]
    fn sync_state_crud() {
        let conn = test_conn();

        // Create a recipe first (foreign key)
        let row = create_recipe(&conn, "Sync Test", &sample_recipe(), None).unwrap();

        // No sync state initially
        let state = get_sync_state(&conn, &row.id, "server").unwrap();
        assert!(state.is_none());

        // Save sync state
        let fake_state = vec![1, 2, 3, 4];
        save_sync_state(&conn, &row.id, "server", &fake_state).unwrap();

        let state = get_sync_state(&conn, &row.id, "server").unwrap().unwrap();
        assert_eq!(state, fake_state);

        // Upsert
        let updated_state = vec![5, 6, 7];
        save_sync_state(&conn, &row.id, "server", &updated_state).unwrap();
        let state = get_sync_state(&conn, &row.id, "server").unwrap().unwrap();
        assert_eq!(state, updated_state);
    }

    #[test]
    fn apply_remote_merge_new_recipe() {
        let conn = test_conn();

        // Create AM data for a recipe that doesn't exist locally
        let doc = RecipeDocument {
            id: "remote-id".to_string(),
            name: "Remote Beer".to_string(),
            recipe: sample_recipe(),
            equipment: None,
            is_deleted: false,
        };
        let am_data = reconcile_to_automerge(&doc, None).unwrap();

        apply_remote_merge(&conn, "remote-id", &am_data).unwrap();

        let fetched = get_recipe(&conn, "remote-id").unwrap().unwrap();
        assert_eq!(fetched.name, "Remote Beer");
        assert!(!fetched.is_dirty); // came from server, not dirty
    }

    #[test]
    fn apply_remote_merge_existing_recipe() {
        let conn = test_conn();

        let row = create_recipe(&conn, "Local Beer", &sample_recipe(), None).unwrap();
        clear_dirty(&conn, &row.id).unwrap();

        // Simulate a remote merge with updated name
        let doc = RecipeDocument {
            id: row.id.clone(),
            name: "Merged Beer".to_string(),
            recipe: sample_recipe(),
            equipment: None,
            is_deleted: false,
        };
        let am_data = reconcile_to_automerge(&doc, None).unwrap();

        apply_remote_merge(&conn, &row.id, &am_data).unwrap();

        let fetched = get_recipe(&conn, &row.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Merged Beer");
        assert!(!fetched.is_dirty); // remote merge doesn't set dirty
    }

    #[test]
    fn apply_remote_merge_with_equipment() {
        let conn = test_conn();
        let equipment = brewdio_core::data::equipment()[0].clone();

        let doc = RecipeDocument {
            id: "remote-equip".to_string(),
            name: "Remote Equipped".to_string(),
            recipe: sample_recipe(),
            equipment: Some(equipment.clone()),
            is_deleted: false,
        };
        let am_data = reconcile_to_automerge(&doc, None).unwrap();

        apply_remote_merge(&conn, "remote-equip", &am_data).unwrap();

        let fetched = get_recipe(&conn, "remote-equip").unwrap().unwrap();
        assert!(fetched.equipment.is_some());
        let doc = fetched.to_document().unwrap();
        assert_eq!(doc.equipment.as_ref().unwrap().name, "Default Setup");
    }

    #[test]
    fn list_all_recipe_ids_includes_deleted() {
        let conn = test_conn();

        let row1 = create_recipe(&conn, "Beer 1", &sample_recipe(), None).unwrap();
        let row2 = create_recipe(&conn, "Beer 2", &sample_recipe(), None).unwrap();
        delete_recipe(&conn, &row2.id).unwrap();

        let ids = list_all_recipe_ids(&conn).unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&row1.id));
        assert!(ids.contains(&row2.id));
    }

    #[test]
    fn migration_version_tracking() {
        let conn = test_conn();

        // Verify user_version was set
        let version: i32 = conn
            .prepare("PRAGMA user_version")
            .unwrap()
            .query_row([], |row| row.get(0))
            .unwrap();
        assert!(version > 0, "user_version should be set after migrations");

        // Verify the equipment column exists by inserting a row with equipment
        let equipment = brewdio_core::data::equipment()[0].clone();
        let row = create_recipe(&conn, "Migration Test", &sample_recipe(), Some(&equipment)).unwrap();
        assert!(row.equipment.is_some());
    }
}
