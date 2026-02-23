#![cfg(feature = "native")]

use brewdio_core::beerjson_types::RecipeType;
use rusqlite::{params, Connection};

use crate::recipe::{hydrate_from_automerge, reconcile_to_automerge, RecipeDocument, RecipeRow};

const MIGRATION_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS recipe (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    recipe TEXT NOT NULL,
    am_data BLOB NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    is_dirty BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE IF NOT EXISTS sync_state (
    recipe_id TEXT NOT NULL,
    peer_id TEXT NOT NULL DEFAULT 'server',
    state BLOB NOT NULL,
    PRIMARY KEY (recipe_id, peer_id),
    FOREIGN KEY (recipe_id) REFERENCES recipe(id)
);
"#;

/// Initialize the database, run migrations, and return a connection.
pub fn init_db(path: &str) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(path)?;
    conn.execute_batch(MIGRATION_SQL)?;
    Ok(conn)
}

fn row_from_rusqlite(row: &rusqlite::Row) -> Result<RecipeRow, rusqlite::Error> {
    Ok(RecipeRow {
        id: row.get("id")?,
        name: row.get("name")?,
        recipe: row.get("recipe")?,
        am_data: row.get("am_data")?,
        is_deleted: row.get("is_deleted")?,
        is_dirty: row.get("is_dirty")?,
    })
}

/// Create a new recipe and return its row.
pub fn create_recipe(
    conn: &Connection,
    name: &str,
    recipe: &RecipeType,
) -> Result<RecipeRow, rusqlite::Error> {
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
        params![id, name, recipe_json, am_data, false, true],
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
pub fn get_recipe(conn: &Connection, id: &str) -> Result<Option<RecipeRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, recipe, am_data, is_deleted, is_dirty FROM recipe WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], row_from_rusqlite)?;
    match rows.next() {
        Some(Ok(row)) => Ok(Some(row)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

/// List all non-deleted recipes.
pub fn list_recipes(conn: &Connection) -> Result<Vec<RecipeRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, recipe, am_data, is_deleted, is_dirty FROM recipe WHERE is_deleted = FALSE",
    )?;
    let rows = stmt.query_map([], row_from_rusqlite)?;
    rows.collect()
}

/// Update a recipe's name and content, re-reconciling into the existing Automerge document.
pub fn update_recipe(
    conn: &Connection,
    id: &str,
    name: &str,
    recipe: &RecipeType,
) -> Result<(), rusqlite::Error> {
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
        params![name, recipe_json, am_data, id],
    )?;

    Ok(())
}

/// Soft-delete a recipe by setting `is_deleted = true`.
pub fn delete_recipe(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    let existing = get_recipe(conn, id)?;
    if let Some(row) = existing {
        let mut doc = row.to_document().expect("Failed to deserialize recipe");
        doc.is_deleted = true;
        let am_data = reconcile_to_automerge(&doc, Some(&row.am_data));

        conn.execute(
            "UPDATE recipe SET is_deleted = TRUE, is_dirty = TRUE, am_data = ?1 WHERE id = ?2",
            params![am_data, id],
        )?;
    }

    Ok(())
}

// --- Sync-related functions ---

/// List all recipes that have local changes not yet synced.
pub fn list_dirty_recipes(conn: &Connection) -> Result<Vec<RecipeRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, name, recipe, am_data, is_deleted, is_dirty FROM recipe WHERE is_dirty = TRUE",
    )?;
    let rows = stmt.query_map([], row_from_rusqlite)?;
    rows.collect()
}

/// Clear the dirty flag for a recipe (after successful sync).
pub fn clear_dirty(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE recipe SET is_dirty = FALSE WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

/// List all recipe IDs (including deleted).
pub fn list_all_recipe_ids(conn: &Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id FROM recipe")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect()
}

/// Get the stored Automerge sync state for a (recipe, peer) pair.
pub fn get_sync_state(
    conn: &Connection,
    recipe_id: &str,
    peer_id: &str,
) -> Result<Option<Vec<u8>>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT state FROM sync_state WHERE recipe_id = ?1 AND peer_id = ?2",
    )?;
    let mut rows = stmt.query_map(params![recipe_id, peer_id], |row| row.get(0))?;
    match rows.next() {
        Some(Ok(state)) => Ok(Some(state)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

/// Save (upsert) Automerge sync state for a (recipe, peer) pair.
pub fn save_sync_state(
    conn: &Connection,
    recipe_id: &str,
    peer_id: &str,
    state: &[u8],
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO sync_state (recipe_id, peer_id, state) VALUES (?1, ?2, ?3)
         ON CONFLICT (recipe_id, peer_id) DO UPDATE SET state = excluded.state",
        params![recipe_id, peer_id, state],
    )?;
    Ok(())
}

/// Apply a remotely-merged Automerge document. Updates the JSON and name columns
/// from the merged AM state. Does NOT set `is_dirty` since the change came from the server.
/// If the recipe doesn't exist locally, inserts it with `is_dirty = FALSE`.
pub fn apply_remote_merge(
    conn: &Connection,
    recipe_id: &str,
    am_data: &[u8],
) -> Result<(), rusqlite::Error> {
    let doc = hydrate_from_automerge(am_data).expect("Failed to hydrate merged AM doc");
    let recipe_json = serde_json::to_string(&doc.recipe).expect("Failed to serialize recipe");

    let existing = get_recipe(conn, recipe_id)?;
    if existing.is_some() {
        conn.execute(
            "UPDATE recipe SET name = ?1, recipe = ?2, am_data = ?3, is_deleted = ?4 WHERE id = ?5",
            params![doc.name, recipe_json, am_data, doc.is_deleted, recipe_id],
        )?;
    } else {
        conn.execute(
            "INSERT INTO recipe (id, name, recipe, am_data, is_deleted, is_dirty) VALUES (?1, ?2, ?3, ?4, ?5, FALSE)",
            params![recipe_id, doc.name, recipe_json, am_data, doc.is_deleted],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use brewdio_core::beerjson_types::RecipeType;

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
        let conn = init_db(":memory:").unwrap();

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
        let conn = init_db(":memory:").unwrap();

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
        let conn = init_db(":memory:").unwrap();

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
        let conn = init_db(":memory:").unwrap();

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
        let conn = init_db(":memory:").unwrap();

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
        let conn = init_db(":memory:").unwrap();

        let row1 = create_recipe(&conn, "Beer 1", &sample_recipe()).unwrap();
        let row2 = create_recipe(&conn, "Beer 2", &sample_recipe()).unwrap();
        delete_recipe(&conn, &row2.id).unwrap();

        let ids = list_all_recipe_ids(&conn).unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&row1.id));
        assert!(ids.contains(&row2.id));
    }
}
