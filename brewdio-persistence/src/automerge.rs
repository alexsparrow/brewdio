use automerge::{AutoCommit, ChangeHash, ObjType, ReadDoc, ScalarValue, Value};
use autosurgeon::{hydrate, reconcile};
use serde::{Deserialize, Serialize};

use crate::batch::BatchDocument;
use crate::recipe::RecipeDocument;

/// Get the current time in milliseconds since UNIX epoch, working on both native and WASM.
#[cfg(not(feature = "wasm"))]
pub(crate) fn current_time_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[cfg(feature = "wasm")]
pub(crate) fn current_time_millis() -> i64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// Generate a new ULID, working on both native and WASM.
pub(crate) fn new_ulid() -> String {
    ulid::Ulid::from_parts(current_time_millis() as u64, rand::random::<u128>()).to_string()
}

/// Create or update an Automerge document from a reconcilable value.
/// If `existing` bytes are provided, loads and reconciles into that doc (preserving history).
/// Otherwise creates a new doc.
pub fn reconcile_to_automerge<T: autosurgeon::Reconcile>(doc: &T, existing: Option<&[u8]>) -> Result<Vec<u8>, String> {
    let mut am_doc = match existing {
        Some(bytes) => AutoCommit::load(bytes).map_err(|e| format!("Failed to load Automerge doc: {}", e))?,
        None => AutoCommit::new(),
    };
    reconcile(&mut am_doc, doc).map_err(|e| format!("Failed to reconcile document: {}", e))?;
    let now = current_time_millis();
    am_doc.commit_with(automerge::transaction::CommitOptions {
        message: None,
        time: Some(now),
    });
    Ok(am_doc.save())
}

/// Hydrate a value from Automerge binary data.
pub fn hydrate_from_automerge<T: autosurgeon::Hydrate>(bytes: &[u8]) -> Result<T, autosurgeon::HydrateError> {
    let am_doc = AutoCommit::load(bytes).expect("Failed to load Automerge document");
    hydrate(&am_doc)
}

/// Extract a single string field by key from an Automerge document.
/// Returns None if the key is missing or not a string.
pub fn extract_string_field_from_automerge(bytes: &[u8], key: &str) -> Option<String> {
    let doc = AutoCommit::load(bytes).ok()?;
    doc.get(automerge::ROOT, key)
        .ok()
        .flatten()
        .and_then(|(v, _)| match v {
            Value::Scalar(s) => match s.as_ref() {
                ScalarValue::Str(s) => Some(s.to_string()),
                _ => None,
            },
            _ => None,
        })
}

/// Extract `name` and `is_deleted` directly from an Automerge document
/// without full autosurgeon hydration. Used as a fallback when hydration
/// fails (e.g. due to `#[serde(untagged)]` enum types in BeerJSON).
pub fn extract_fields_from_automerge(bytes: &[u8]) -> (String, bool) {
    let doc = AutoCommit::load(bytes).expect("Failed to load Automerge document");

    let name = doc
        .get(automerge::ROOT, "name")
        .ok()
        .flatten()
        .and_then(|(v, _)| match v {
            Value::Scalar(s) => match s.as_ref() {
                ScalarValue::Str(s) => Some(s.to_string()),
                _ => None,
            },
            _ => None,
        })
        .unwrap_or_default();

    let is_deleted = doc
        .get(automerge::ROOT, "is_deleted")
        .ok()
        .flatten()
        .and_then(|(v, _)| match v {
            Value::Scalar(s) => match s.as_ref() {
                ScalarValue::Boolean(b) => Some(*b),
                _ => None,
            },
            _ => None,
        })
        .unwrap_or(false);

    (name, is_deleted)
}

/// Dump the top-level structure of an Automerge document for debugging.
/// Recursively prints keys and value types up to `max_depth`.
pub fn dump_automerge_structure(bytes: &[u8], max_depth: usize) -> String {
    let doc = AutoCommit::load(bytes).expect("Failed to load Automerge document");
    let mut out = String::new();
    dump_obj(&doc, &automerge::ROOT, 0, max_depth, &mut out);
    out
}

fn dump_obj(doc: &AutoCommit, obj: &automerge::ObjId, depth: usize, max_depth: usize, out: &mut String) {
    if depth > max_depth {
        out.push_str(&format!("{}...\n", "  ".repeat(depth)));
        return;
    }
    let indent = "  ".repeat(depth);

    let obj_type = doc.object_type(obj);
    match obj_type {
        Ok(ObjType::Map) | Ok(ObjType::Table) => {
            let keys = doc.keys(obj);
            for key in keys {
                match doc.get(obj, &*key) {
                    Ok(Some((val, id))) => match val {
                        Value::Object(ot) => {
                            out.push_str(&format!("{}{}: {:?}\n", indent, key, ot));
                            dump_obj(doc, &id, depth + 1, max_depth, out);
                        }
                        Value::Scalar(s) => {
                            out.push_str(&format!("{}{}: {:?}\n", indent, key, s.as_ref()));
                        }
                    },
                    Ok(None) => {
                        out.push_str(&format!("{}{}: <none>\n", indent, key));
                    }
                    Err(e) => {
                        out.push_str(&format!("{}{}: <error: {}>\n", indent, key, e));
                    }
                }
            }
        }
        Ok(ObjType::List) | Ok(ObjType::Text) => {
            let len = doc.length(obj);
            for i in 0..len {
                match doc.get(obj, i as usize) {
                    Ok(Some((val, id))) => match val {
                        Value::Object(ot) => {
                            out.push_str(&format!("{}[{}]: {:?}\n", indent, i, ot));
                            dump_obj(doc, &id, depth + 1, max_depth, out);
                        }
                        Value::Scalar(s) => {
                            out.push_str(&format!("{}[{}]: {:?}\n", indent, i, s.as_ref()));
                        }
                    },
                    Ok(None) => {
                        out.push_str(&format!("{}[{}]: <none>\n", indent, i));
                    }
                    Err(e) => {
                        out.push_str(&format!("{}[{}]: <error: {}>\n", indent, i, e));
                    }
                }
            }
        }
        Err(_) => {
            out.push_str(&format!("{}(unknown object)\n", indent));
        }
    }
}

/// A single entry in the change history of an Automerge document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEntry {
    pub hash: String,
    pub timestamp: i64,
    pub message: Option<String>,
    pub num_ops: usize,
    pub seq: u64,
    pub deps: Vec<String>,
    pub actor_id: String,
}

/// A history entry combining a change with a human-readable diff summary.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub change: ChangeEntry,
    pub summary: String,
}

/// Read the full history of changes from Automerge binary data.
/// Returns changes in causal order (earliest first).
pub fn get_change_history(bytes: &[u8]) -> Vec<ChangeEntry> {
    let mut am_doc = AutoCommit::load(bytes).expect("Failed to load Automerge document");
    let changes = am_doc.get_changes(&[]);
    changes
        .into_iter()
        .map(|c| ChangeEntry {
            hash: c.hash().to_string(),
            timestamp: c.timestamp(),
            message: c.message().cloned(),
            num_ops: c.len(),
            seq: c.seq(),
            deps: c.deps().iter().map(ChangeHash::to_string).collect(),
            actor_id: c.actor_id().to_string(),
        })
        .collect()
}

/// Hydrate the document state at a specific point in history.
/// `hash` identifies the change up to which (inclusive) to reconstruct state.
pub fn hydrate_at_change<T: autosurgeon::Hydrate>(
    bytes: &[u8],
    hash: &str,
) -> Result<T, autosurgeon::HydrateError> {
    let mut am_doc = AutoCommit::load(bytes).expect("Failed to load Automerge document");

    let changes = am_doc.get_changes(&[]);
    let target_hash: ChangeHash = hash.parse().expect("Invalid change hash");

    let target_idx = changes
        .iter()
        .position(|c| c.hash() == target_hash)
        .expect("Change hash not found in document history");

    // Create a new doc with only changes up to (and including) the target
    let mut partial = AutoCommit::new();
    for change in &changes[..=target_idx] {
        partial
            .apply_changes(std::iter::once(change.clone()))
            .expect("Failed to apply change");
    }

    hydrate(&partial)
}

/// Build history entries for a recipe document with field-level diffs.
/// Uses incremental change application (O(n)) instead of replaying from scratch per entry.
pub fn get_recipe_history(bytes: &[u8]) -> Vec<HistoryEntry> {
    let mut am_doc = AutoCommit::load(bytes).expect("Failed to load Automerge document");
    let changes = am_doc.get_changes(&[]);

    let mut entries = Vec::with_capacity(changes.len());
    let mut partial = AutoCommit::new();
    let mut prev_doc: Option<RecipeDocument> = None;

    for (i, change) in changes.iter().enumerate() {
        let ce = ChangeEntry {
            hash: change.hash().to_string(),
            timestamp: change.timestamp(),
            message: change.message().cloned(),
            num_ops: change.len(),
            seq: change.seq(),
            deps: change.deps().iter().map(ChangeHash::to_string).collect(),
            actor_id: change.actor_id().to_string(),
        };

        partial
            .apply_changes(std::iter::once(change.clone()))
            .expect("Failed to apply change");

        let summary = if i == 0 {
            let curr: Option<RecipeDocument> = hydrate(&partial).ok();
            prev_doc = curr;
            "Created".to_string()
        } else {
            let curr: Option<RecipeDocument> = hydrate(&partial).ok();
            let s = match (&prev_doc, &curr) {
                (Some(prev), Some(curr)) => diff_recipe(prev, curr),
                _ => format!("{} operation(s)", ce.num_ops),
            };
            prev_doc = curr;
            s
        };

        entries.push(HistoryEntry { change: ce, summary });
    }
    entries
}

/// Build history entries for a batch document with field-level diffs.
/// Uses incremental change application (O(n)) instead of replaying from scratch per entry.
pub fn get_batch_history(bytes: &[u8]) -> Vec<HistoryEntry> {
    let mut am_doc = AutoCommit::load(bytes).expect("Failed to load Automerge document");
    let changes = am_doc.get_changes(&[]);

    let mut entries = Vec::with_capacity(changes.len());
    let mut partial = AutoCommit::new();
    let mut prev_doc: Option<BatchDocument> = None;

    for (i, change) in changes.iter().enumerate() {
        let ce = ChangeEntry {
            hash: change.hash().to_string(),
            timestamp: change.timestamp(),
            message: change.message().cloned(),
            num_ops: change.len(),
            seq: change.seq(),
            deps: change.deps().iter().map(ChangeHash::to_string).collect(),
            actor_id: change.actor_id().to_string(),
        };

        partial
            .apply_changes(std::iter::once(change.clone()))
            .expect("Failed to apply change");

        let summary = if i == 0 {
            let curr: Option<BatchDocument> = hydrate(&partial).ok();
            prev_doc = curr;
            "Created".to_string()
        } else {
            let curr: Option<BatchDocument> = hydrate(&partial).ok();
            let s = match (&prev_doc, &curr) {
                (Some(prev), Some(curr)) => diff_batch(prev, curr),
                _ => format!("{} operation(s)", ce.num_ops),
            };
            prev_doc = curr;
            s
        };

        entries.push(HistoryEntry { change: ce, summary });
    }
    entries
}

fn diff_recipe(prev: &RecipeDocument, curr: &RecipeDocument) -> String {
    let mut diffs = Vec::new();

    if prev.name != curr.name {
        diffs.push(format!("Renamed to \"{}\"", curr.name));
    }
    if prev.is_deleted != curr.is_deleted {
        if curr.is_deleted {
            diffs.push("Deleted".to_string());
        } else {
            diffs.push("Restored".to_string());
        }
    }

    let prev_bs = format!("{} {:?}", prev.recipe.batch_size.value, prev.recipe.batch_size.unit);
    let curr_bs = format!("{} {:?}", curr.recipe.batch_size.value, curr.recipe.batch_size.unit);
    if prev_bs != curr_bs {
        diffs.push("Changed batch size".to_string());
    }

    diff_ingredients(
        &prev.recipe.ingredients.fermentable_additions,
        &curr.recipe.ingredients.fermentable_additions,
        "fermentable",
        &mut diffs,
    );
    diff_ingredients(
        &prev.recipe.ingredients.hop_additions,
        &curr.recipe.ingredients.hop_additions,
        "hop",
        &mut diffs,
    );
    diff_ingredients(
        &prev.recipe.ingredients.culture_additions,
        &curr.recipe.ingredients.culture_additions,
        "culture",
        &mut diffs,
    );

    let prev_style = prev.recipe.style.as_ref().map(|s| s.0.name.as_str());
    let curr_style = curr.recipe.style.as_ref().map(|s| s.0.name.as_str());
    if prev_style != curr_style {
        match curr_style {
            Some(name) => diffs.push(format!("Set style to \"{}\"", name)),
            None => diffs.push("Removed style".to_string()),
        }
    }

    if prev.recipe.notes != curr.recipe.notes {
        diffs.push("Updated notes".to_string());
    }

    if diffs.is_empty() {
        "Updated".to_string()
    } else {
        diffs.join(", ")
    }
}

fn diff_batch(prev: &BatchDocument, curr: &BatchDocument) -> String {
    let mut diffs = Vec::new();

    if prev.name != curr.name {
        diffs.push(format!("Renamed to \"{}\"", curr.name));
    }
    if prev.is_deleted != curr.is_deleted {
        if curr.is_deleted {
            diffs.push("Deleted".to_string());
        } else {
            diffs.push("Restored".to_string());
        }
    }

    if prev.data.brew_date != curr.data.brew_date {
        diffs.push("Changed brew date".to_string());
    }
    if prev.data.notes != curr.data.notes {
        diffs.push("Updated notes".to_string());
    }

    diff_ingredients(
        &prev.data.recipe.ingredients.fermentable_additions,
        &curr.data.recipe.ingredients.fermentable_additions,
        "fermentable",
        &mut diffs,
    );
    diff_ingredients(
        &prev.data.recipe.ingredients.hop_additions,
        &curr.data.recipe.ingredients.hop_additions,
        "hop",
        &mut diffs,
    );
    diff_ingredients(
        &prev.data.recipe.ingredients.culture_additions,
        &curr.data.recipe.ingredients.culture_additions,
        "culture",
        &mut diffs,
    );

    if diffs.is_empty() {
        "Updated".to_string()
    } else {
        diffs.join(", ")
    }
}

fn diff_ingredients<T: Serialize>(prev: &[T], curr: &[T], kind: &str, diffs: &mut Vec<String>) {
    let prev_len = prev.len();
    let curr_len = curr.len();

    if curr_len > prev_len {
        let n = curr_len - prev_len;
        let plural = if n == 1 { "" } else { "s" };
        diffs.push(format!("Added {} {}{}", n, kind, plural));
    } else if curr_len < prev_len {
        let n = prev_len - curr_len;
        let plural = if n == 1 { "" } else { "s" };
        diffs.push(format!("Removed {} {}{}", n, kind, plural));
    }

    // Check overlapping items for modifications (compare via JSON)
    let check_len = prev_len.min(curr_len);
    let mut modified = 0;
    for i in 0..check_len {
        let prev_json = serde_json::to_string(&prev[i]).unwrap_or_default();
        let curr_json = serde_json::to_string(&curr[i]).unwrap_or_default();
        if prev_json != curr_json {
            modified += 1;
        }
    }
    if modified > 0 {
        let plural = if modified == 1 { "" } else { "s" };
        diffs.push(format!("Updated {} {}{}", modified, kind, plural));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_recipe, delete_recipe, get_recipe, update_recipe};
    use crate::recipe::RecipeDocument;
    use brewdio_core::beerjson_types::RecipeType;

    fn test_conn() -> rusqlite::Connection {
        crate::connection_native::open(":memory:").unwrap()
    }

    fn sample_recipe() -> RecipeType {
        serde_json::from_str(
            r#"{
                "name": "Test IPA",
                "type": "all grain",
                "author": "Tester",
                "batch_size": { "unit": "l", "value": 20.0 },
                "efficiency": { "brewhouse": { "unit": "%", "value": 72.0 } },
                "ingredients": {
                    "fermentable_additions": [],
                    "hop_additions": []
                }
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn recipe_history_tracks_create_and_updates() {
        let conn = test_conn();

        // Create a recipe
        let row = create_recipe(&conn, "Version 1", &sample_recipe()).unwrap();
        let fetched = get_recipe(&conn, &row.id).unwrap().unwrap();
        let history = get_change_history(&fetched.am_data);
        assert_eq!(history.len(), 1, "Create should produce 1 change");

        // First update
        let mut recipe_v2 = sample_recipe();
        recipe_v2.name = "Version 2".to_string();
        update_recipe(&conn, &row.id, "Version 2", &recipe_v2).unwrap();
        let fetched = get_recipe(&conn, &row.id).unwrap().unwrap();
        let history = get_change_history(&fetched.am_data);
        assert_eq!(history.len(), 2, "One update should produce 2 changes");

        // Second update
        let mut recipe_v3 = sample_recipe();
        recipe_v3.name = "Version 3".to_string();
        update_recipe(&conn, &row.id, "Version 3", &recipe_v3).unwrap();
        let fetched = get_recipe(&conn, &row.id).unwrap().unwrap();
        let history = get_change_history(&fetched.am_data);
        assert_eq!(history.len(), 3, "Two updates should produce 3 changes");

        // Changes should be causally ordered
        assert!(history[0].deps.is_empty());
        assert_eq!(history[1].deps, vec![history[0].hash.clone()]);
        assert_eq!(history[2].deps, vec![history[1].hash.clone()]);

        // Hydrate at each point in history
        let at_v1: RecipeDocument = hydrate_at_change(&fetched.am_data, &history[0].hash).unwrap();
        assert_eq!(at_v1.name, "Version 1");

        let at_v2: RecipeDocument = hydrate_at_change(&fetched.am_data, &history[1].hash).unwrap();
        assert_eq!(at_v2.name, "Version 2");

        let at_v3: RecipeDocument = hydrate_at_change(&fetched.am_data, &history[2].hash).unwrap();
        assert_eq!(at_v3.name, "Version 3");
    }

    #[test]
    fn recipe_history_tracks_delete() {
        let conn = test_conn();

        let row = create_recipe(&conn, "My Beer", &sample_recipe()).unwrap();

        // Update once
        update_recipe(&conn, &row.id, "My Beer v2", &sample_recipe()).unwrap();

        // Soft-delete
        delete_recipe(&conn, &row.id).unwrap();

        let fetched = get_recipe(&conn, &row.id).unwrap().unwrap();
        let history = get_change_history(&fetched.am_data);
        assert_eq!(history.len(), 3, "Create + update + delete = 3 changes");

        // At first change, not deleted
        let at_v1: RecipeDocument = hydrate_at_change(&fetched.am_data, &history[0].hash).unwrap();
        assert!(!at_v1.is_deleted);
        assert_eq!(at_v1.name, "My Beer");

        // At second change, updated but not deleted
        let at_v2: RecipeDocument = hydrate_at_change(&fetched.am_data, &history[1].hash).unwrap();
        assert!(!at_v2.is_deleted);
        assert_eq!(at_v2.name, "My Beer v2");

        // At third change, deleted
        let at_v3: RecipeDocument = hydrate_at_change(&fetched.am_data, &history[2].hash).unwrap();
        assert!(at_v3.is_deleted);
    }
}
