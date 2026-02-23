use automerge::{AutoCommit, ChangeHash, ObjType, ReadDoc, ScalarValue, Value};
use autosurgeon::{hydrate, reconcile};
use serde::{Deserialize, Serialize};

/// Create or update an Automerge document from a reconcilable value.
/// If `existing` bytes are provided, loads and reconciles into that doc (preserving history).
/// Otherwise creates a new doc.
pub fn reconcile_to_automerge<T: autosurgeon::Reconcile>(doc: &T, existing: Option<&[u8]>) -> Vec<u8> {
    let mut am_doc = match existing {
        Some(bytes) => AutoCommit::load(bytes).expect("Failed to load Automerge doc"),
        None => AutoCommit::new(),
    };
    reconcile(&mut am_doc, doc).expect("Failed to reconcile document");
    am_doc.save()
}

/// Hydrate a value from Automerge binary data.
pub fn hydrate_from_automerge<T: autosurgeon::Hydrate>(bytes: &[u8]) -> Result<T, autosurgeon::HydrateError> {
    let am_doc = AutoCommit::load(bytes).expect("Failed to load Automerge document");
    hydrate(&am_doc)
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
