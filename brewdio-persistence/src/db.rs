use brewdio_core::beerjson_types::RecipeType;

use crate::connection::{Connection, DbError, Value};
use crate::recipe::{hydrate_from_automerge, reconcile_to_automerge, RecipeDocument, RecipeRow};

fn row_from_query(row: &dyn crate::connection::Row) -> RecipeRow {
    RecipeRow {
        id: row.get_text(0),
        name: row.get_text(1),
        recipe: row.get_text(2),
        am_data: row.get_blob(3),
        is_deleted: row.get_bool(4),
        is_dirty: row.get_bool(5),
    }
}

/// Create a new recipe and return its row.
pub fn create_recipe(
    conn: &(impl Connection + ?Sized),
    name: &str,
    recipe: &RecipeType,
) -> Result<RecipeRow, DbError> {
    let id = ulid::Ulid::new().to_string();
    let recipe_json = serde_json::to_string(recipe).expect("Failed to serialize recipe");

    let doc = RecipeDocument {
        id: id.clone(),
        name: name.to_string(),
        recipe: recipe.clone(),
        is_deleted: false,
    };
    let am_data = reconcile_to_automerge(&doc, None);

    conn.execute(
        "INSERT INTO recipe (id, name, recipe, am_data, is_deleted, is_dirty) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        &[Value::Text(&id), Value::Text(name), Value::Text(&recipe_json), Value::Blob(&am_data), Value::Bool(false), Value::Bool(true)],
    )?;

    Ok(RecipeRow {
        id,
        name: name.to_string(),
        recipe: recipe_json,
        am_data,
        is_deleted: false,
        is_dirty: true,
    })
}

/// Get a recipe by ID.
pub fn get_recipe(conn: &(impl Connection + ?Sized), id: &str) -> Result<Option<RecipeRow>, DbError> {
    conn.query_one(
        "SELECT id, name, recipe, am_data, is_deleted, is_dirty FROM recipe WHERE id = ?1",
        &[Value::Text(id)],
        row_from_query,
    )
}

/// List all non-deleted recipes.
pub fn list_recipes(conn: &(impl Connection + ?Sized)) -> Result<Vec<RecipeRow>, DbError> {
    conn.query_map(
        "SELECT id, name, recipe, am_data, is_deleted, is_dirty FROM recipe WHERE is_deleted = FALSE",
        &[],
        row_from_query,
    )
}

/// Update a recipe's name and content, re-reconciling into the existing Automerge document.
pub fn update_recipe(
    conn: &(impl Connection + ?Sized),
    id: &str,
    name: &str,
    recipe: &RecipeType,
) -> Result<(), DbError> {
    let recipe_json = serde_json::to_string(recipe).expect("Failed to serialize recipe");

    let existing = get_recipe(conn, id)?;
    let existing_am = existing.as_ref().map(|r| r.am_data.as_slice());

    let doc = RecipeDocument {
        id: id.to_string(),
        name: name.to_string(),
        recipe: recipe.clone(),
        is_deleted: false,
    };
    let am_data = reconcile_to_automerge(&doc, existing_am);

    conn.execute(
        "UPDATE recipe SET name = ?1, recipe = ?2, am_data = ?3, is_dirty = TRUE WHERE id = ?4",
        &[Value::Text(name), Value::Text(&recipe_json), Value::Blob(&am_data), Value::Text(id)],
    )?;

    Ok(())
}

/// Soft-delete a recipe by setting `is_deleted = true`.
pub fn delete_recipe(conn: &(impl Connection + ?Sized), id: &str) -> Result<(), DbError> {
    let existing = get_recipe(conn, id)?;
    if let Some(row) = existing {
        let mut doc = row.to_document().expect("Failed to deserialize recipe");
        doc.is_deleted = true;
        let am_data = reconcile_to_automerge(&doc, Some(&row.am_data));

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
        "SELECT id, name, recipe, am_data, is_deleted, is_dirty FROM recipe WHERE is_dirty = TRUE",
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

/// Apply a remotely-merged Automerge document. Updates the JSON and name columns
/// from the merged AM state. Does NOT set `is_dirty` since the change came from the server.
/// If the recipe doesn't exist locally, inserts it with `is_dirty = FALSE`.
pub fn apply_remote_merge(
    conn: &(impl Connection + ?Sized),
    recipe_id: &str,
    am_data: &[u8],
) -> Result<(), DbError> {
    let doc = hydrate_from_automerge(am_data).expect("Failed to hydrate merged AM doc");
    let recipe_json = serde_json::to_string(&doc.recipe).expect("Failed to serialize recipe");

    let existing = get_recipe(conn, recipe_id)?;
    if existing.is_some() {
        conn.execute(
            "UPDATE recipe SET name = ?1, recipe = ?2, am_data = ?3, is_deleted = ?4 WHERE id = ?5",
            &[Value::Text(&doc.name), Value::Text(&recipe_json), Value::Blob(am_data), Value::Bool(doc.is_deleted), Value::Text(recipe_id)],
        )?;
    } else {
        conn.execute(
            "INSERT INTO recipe (id, name, recipe, am_data, is_deleted, is_dirty) VALUES (?1, ?2, ?3, ?4, ?5, FALSE)",
            &[Value::Text(recipe_id), Value::Text(&doc.name), Value::Text(&recipe_json), Value::Blob(am_data), Value::Bool(doc.is_deleted)],
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
        let row = create_recipe(&conn, "Test IPA", &sample_recipe()).unwrap();
        assert_eq!(row.name, "Test IPA");
        assert!(!row.id.is_empty());
        assert!(row.is_dirty);

        // Get
        let fetched = get_recipe(&conn, &row.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Test IPA");
        assert!(fetched.is_dirty);

        // List
        let recipes = list_recipes(&conn).unwrap();
        assert_eq!(recipes.len(), 1);

        // Update
        let mut updated_recipe = sample_recipe();
        updated_recipe.name = "Updated IPA".to_string();
        update_recipe(&conn, &row.id, "Updated IPA", &updated_recipe).unwrap();
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
    fn dirty_tracking() {
        let conn = test_conn();

        // Create sets dirty
        let row = create_recipe(&conn, "Dirty Test", &sample_recipe()).unwrap();
        let dirty = list_dirty_recipes(&conn).unwrap();
        assert_eq!(dirty.len(), 1);

        // Clear dirty
        clear_dirty(&conn, &row.id).unwrap();
        let dirty = list_dirty_recipes(&conn).unwrap();
        assert_eq!(dirty.len(), 0);

        // Update sets dirty again
        update_recipe(&conn, &row.id, "Still Dirty", &sample_recipe()).unwrap();
        let dirty = list_dirty_recipes(&conn).unwrap();
        assert_eq!(dirty.len(), 1);
    }

    #[test]
    fn sync_state_crud() {
        let conn = test_conn();

        // Create a recipe first (foreign key)
        let row = create_recipe(&conn, "Sync Test", &sample_recipe()).unwrap();

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
            is_deleted: false,
        };
        let am_data = reconcile_to_automerge(&doc, None);

        apply_remote_merge(&conn, "remote-id", &am_data).unwrap();

        let fetched = get_recipe(&conn, "remote-id").unwrap().unwrap();
        assert_eq!(fetched.name, "Remote Beer");
        assert!(!fetched.is_dirty); // came from server, not dirty
    }

    #[test]
    fn apply_remote_merge_existing_recipe() {
        let conn = test_conn();

        let row = create_recipe(&conn, "Local Beer", &sample_recipe()).unwrap();
        clear_dirty(&conn, &row.id).unwrap();

        // Simulate a remote merge with updated name
        let doc = RecipeDocument {
            id: row.id.clone(),
            name: "Merged Beer".to_string(),
            recipe: sample_recipe(),
            is_deleted: false,
        };
        let am_data = reconcile_to_automerge(&doc, None);

        apply_remote_merge(&conn, &row.id, &am_data).unwrap();

        let fetched = get_recipe(&conn, &row.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Merged Beer");
        assert!(!fetched.is_dirty); // remote merge doesn't set dirty
    }

    #[test]
    fn list_all_recipe_ids_includes_deleted() {
        let conn = test_conn();

        let row1 = create_recipe(&conn, "Beer 1", &sample_recipe()).unwrap();
        let row2 = create_recipe(&conn, "Beer 2", &sample_recipe()).unwrap();
        delete_recipe(&conn, &row2.id).unwrap();

        let ids = list_all_recipe_ids(&conn).unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&row1.id));
        assert!(ids.contains(&row2.id));
    }
}
