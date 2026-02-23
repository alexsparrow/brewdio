use automerge::AutoCommit;
use autosurgeon::{hydrate, reconcile};
use brewdio_core::beerjson_types::RecipeType;
use serde::{Deserialize, Serialize};

/// Row representation for SQLite storage.
#[derive(Debug, Clone)]
pub struct RecipeRow {
    pub id: String,
    pub name: String,
    pub recipe: String,
    pub am_data: Vec<u8>,
    pub is_deleted: bool,
    pub is_dirty: bool,
}

/// Automerge-reconciled document for CRDT sync.
#[derive(Clone, Debug, Serialize, Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate)]
pub struct RecipeDocument {
    pub id: String,
    pub name: String,
    pub recipe: RecipeType,
    pub is_deleted: bool,
}

impl RecipeRow {
    /// Deserialize the JSON `recipe` field into a full `RecipeDocument`.
    pub fn to_document(&self) -> Result<RecipeDocument, serde_json::Error> {
        let recipe: RecipeType = serde_json::from_str(&self.recipe)?;
        Ok(RecipeDocument {
            id: self.id.clone(),
            name: self.name.clone(),
            recipe,
            is_deleted: self.is_deleted,
        })
    }
}

impl RecipeDocument {
    /// Serialize back into a `RecipeRow`, pairing with existing Automerge binary data.
    pub fn to_row(&self, am_data: Vec<u8>) -> Result<RecipeRow, serde_json::Error> {
        let recipe_json = serde_json::to_string(&self.recipe)?;
        Ok(RecipeRow {
            id: self.id.clone(),
            name: self.name.clone(),
            recipe: recipe_json,
            am_data,
            is_deleted: self.is_deleted,
            is_dirty: true,
        })
    }
}

/// Create or update an Automerge document from a `RecipeDocument`.
/// If `existing` bytes are provided, loads and reconciles into that doc (preserving history).
/// Otherwise creates a new doc.
pub fn reconcile_to_automerge(doc: &RecipeDocument, existing: Option<&[u8]>) -> Vec<u8> {
    let mut am_doc = match existing {
        Some(bytes) => AutoCommit::load(bytes).expect("Failed to load Automerge doc"),
        None => AutoCommit::new(),
    };
    reconcile(&mut am_doc, doc).expect("Failed to reconcile document");
    am_doc.save()
}

/// Hydrate a `RecipeDocument` from Automerge binary data.
pub fn hydrate_from_automerge(bytes: &[u8]) -> Result<RecipeDocument, autosurgeon::HydrateError> {
    let am_doc = AutoCommit::load(bytes).expect("Failed to load Automerge document");
    hydrate(&am_doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_recipe_json() -> &'static str {
        r#"{
            "name": "Test IPA",
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
        }"#
    }

    #[test]
    fn automerge_roundtrip() {
        let recipe: RecipeType = serde_json::from_str(sample_recipe_json()).unwrap();
        let doc = RecipeDocument {
            id: "test-id".to_string(),
            name: "Test IPA".to_string(),
            recipe,
            is_deleted: false,
        };

        let bytes = reconcile_to_automerge(&doc, None);
        assert!(!bytes.is_empty());

        let hydrated = hydrate_from_automerge(&bytes).unwrap();
        assert_eq!(hydrated.id, "test-id");
        assert_eq!(hydrated.name, "Test IPA");
        assert_eq!(hydrated.is_deleted, false);

        // Verify the recipe survived the roundtrip
        let original_json = serde_json::to_value(&doc.recipe).unwrap();
        let hydrated_json = serde_json::to_value(&hydrated.recipe).unwrap();
        assert_eq!(original_json, hydrated_json);
    }

    #[test]
    fn automerge_history_preservation() {
        let recipe: RecipeType = serde_json::from_str(sample_recipe_json()).unwrap();
        let doc = RecipeDocument {
            id: "test-id".to_string(),
            name: "Test IPA".to_string(),
            recipe: recipe.clone(),
            is_deleted: false,
        };

        // First reconcile — creates doc
        let bytes_v1 = reconcile_to_automerge(&doc, None);

        // Second reconcile — loads existing doc (preserving history)
        let doc_v2 = RecipeDocument {
            id: "test-id".to_string(),
            name: "Updated IPA".to_string(),
            recipe,
            is_deleted: false,
        };
        let bytes_v2 = reconcile_to_automerge(&doc_v2, Some(&bytes_v1));

        // Verify the updated doc has the new name
        let hydrated = hydrate_from_automerge(&bytes_v2).unwrap();
        assert_eq!(hydrated.name, "Updated IPA");

        // Verify history is preserved: the v2 doc should have more than one change
        let mut am_doc = AutoCommit::load(&bytes_v2).unwrap();
        let change_count = am_doc.get_changes(&[]).len();
        assert!(
            change_count > 1,
            "Expected multiple changes for history preservation, got {}",
            change_count
        );
    }

    #[test]
    fn row_document_conversion() {
        let recipe_json = sample_recipe_json();
        let recipe: RecipeType = serde_json::from_str(recipe_json).unwrap();
        let am_data = reconcile_to_automerge(
            &RecipeDocument {
                id: "row-id".to_string(),
                name: "Row Test".to_string(),
                recipe: recipe.clone(),
                is_deleted: false,
            },
            None,
        );

        let row = RecipeRow {
            id: "row-id".to_string(),
            name: "Row Test".to_string(),
            recipe: serde_json::to_string(&recipe).unwrap(),
            am_data: am_data.clone(),
            is_deleted: false,
            is_dirty: true,
        };

        let doc = row.to_document().unwrap();
        assert_eq!(doc.id, "row-id");
        assert_eq!(doc.name, "Row Test");

        let back = doc.to_row(am_data).unwrap();
        assert_eq!(back.id, "row-id");
    }
}
