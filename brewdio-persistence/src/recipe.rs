use brewdio_core::beerjson_types::{EquipmentType, RecipeType};
use serde::{Deserialize, Serialize};

use crate::connection::DbError;
use crate::protocol::DocType;
use crate::traits::{SyncDocument, SyncRow};

/// Public typed recipe for external consumers.
/// JSON fields from the underlying `RecipeRow` are deserialized.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub recipe: RecipeType,
    pub equipment: Option<EquipmentType>,
    pub equipment_id: Option<String>,
}

/// Row representation for SQLite storage.
#[derive(Debug, Clone)]
pub struct RecipeRow {
    pub id: String,
    pub name: String,
    pub recipe: String,
    pub equipment: Option<String>,
    pub equipment_id: Option<String>,
    pub am_data: Vec<u8>,
    pub is_deleted: bool,
    pub is_dirty: bool,
}

fn default_none<T>() -> Option<T> {
    None
}

/// Automerge-reconciled document for CRDT sync.
#[derive(Clone, Debug, Serialize, Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate)]
pub struct RecipeDocument {
    pub id: String,
    pub name: String,
    pub recipe: RecipeType,
    #[autosurgeon(missing = "default_none")]
    pub equipment: Option<EquipmentType>,
    #[autosurgeon(missing = "default_none")]
    pub equipment_id: Option<String>,
    pub is_deleted: bool,
}

impl RecipeRow {
    /// Convert to a typed `Recipe` (deserializes JSON fields).
    pub fn to_recipe(&self) -> Result<Recipe, DbError> {
        let recipe: RecipeType = serde_json::from_str(&self.recipe)
            .map_err(|e| DbError(e.to_string()))?;
        let equipment: Option<EquipmentType> = self.equipment.as_ref()
            .map(|json| serde_json::from_str(json))
            .transpose()
            .map_err(|e| DbError(e.to_string()))?;
        Ok(Recipe {
            id: self.id.clone(),
            name: self.name.clone(),
            recipe,
            equipment,
            equipment_id: self.equipment_id.clone(),
        })
    }

    /// Deserialize the JSON `recipe` field into a full `RecipeDocument`.
    pub fn to_document(&self) -> Result<RecipeDocument, serde_json::Error> {
        let recipe: RecipeType = serde_json::from_str(&self.recipe)?;
        let equipment: Option<EquipmentType> = match &self.equipment {
            Some(json) => Some(serde_json::from_str(json)?),
            None => None,
        };
        Ok(RecipeDocument {
            id: self.id.clone(),
            name: self.name.clone(),
            recipe,
            equipment,
            equipment_id: self.equipment_id.clone(),
            is_deleted: self.is_deleted,
        })
    }
}

impl SyncDocument for RecipeDocument {
    fn id(&self) -> &str { &self.id }
    fn is_deleted(&self) -> bool { self.is_deleted }
    fn set_is_deleted(&mut self, deleted: bool) { self.is_deleted = deleted; }
}

impl SyncRow for RecipeRow {
    type Document = RecipeDocument;
    fn doc_type() -> DocType { DocType::Recipe }
    fn id(&self) -> &str { &self.id }
    fn am_data(&self) -> &[u8] { &self.am_data }
    fn into_document(&self) -> Result<RecipeDocument, DbError> {
        self.to_document().map_err(|e| DbError(e.to_string()))
    }
}

impl RecipeDocument {
    /// Serialize back into a `RecipeRow`, pairing with existing Automerge binary data.
    pub fn to_row(&self, am_data: Vec<u8>) -> Result<RecipeRow, serde_json::Error> {
        let recipe_json = serde_json::to_string(&self.recipe)?;
        let equipment_json = match &self.equipment {
            Some(eq) => Some(serde_json::to_string(eq)?),
            None => None,
        };
        Ok(RecipeRow {
            id: self.id.clone(),
            name: self.name.clone(),
            recipe: recipe_json,
            equipment: equipment_json,
            equipment_id: self.equipment_id.clone(),
            am_data,
            is_deleted: self.is_deleted,
            is_dirty: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use automerge::AutoCommit;
    use crate::automerge::{hydrate_from_automerge, reconcile_to_automerge};

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
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };

        let bytes = reconcile_to_automerge(&doc, None).unwrap();
        assert!(!bytes.is_empty());

        let hydrated: RecipeDocument = hydrate_from_automerge(&bytes).unwrap();
        assert_eq!(hydrated.id, "test-id");
        assert_eq!(hydrated.name, "Test IPA");
        assert_eq!(hydrated.is_deleted, false);
        assert!(hydrated.equipment.is_none());

        // Verify the recipe survived the roundtrip
        let original_json = serde_json::to_value(&doc.recipe).unwrap();
        let hydrated_json = serde_json::to_value(&hydrated.recipe).unwrap();
        assert_eq!(original_json, hydrated_json);
    }

    #[test]
    fn automerge_roundtrip_with_equipment() {
        let recipe: RecipeType = serde_json::from_str(sample_recipe_json()).unwrap();
        let profile = brewdio_core::data::equipment()[0].clone();
        let doc = RecipeDocument {
            id: "equip-test".to_string(),
            name: "Equipped IPA".to_string(),
            recipe,
            equipment: Some(profile.equipment.clone()),
            equipment_id: Some(profile.id.clone()),
            is_deleted: false,
        };

        let bytes = reconcile_to_automerge(&doc, None).unwrap();
        let hydrated: RecipeDocument = hydrate_from_automerge(&bytes).unwrap();
        assert!(hydrated.equipment.is_some());
        assert_eq!(hydrated.equipment.unwrap().name, profile.equipment.name);
        assert_eq!(hydrated.equipment_id.as_deref(), Some("default-setup"));
    }

    #[test]
    fn automerge_history_preservation() {
        let recipe: RecipeType = serde_json::from_str(sample_recipe_json()).unwrap();
        let doc = RecipeDocument {
            id: "test-id".to_string(),
            name: "Test IPA".to_string(),
            recipe: recipe.clone(),
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };

        // First reconcile — creates doc
        let bytes_v1 = reconcile_to_automerge(&doc, None).unwrap();

        // Second reconcile — loads existing doc (preserving history)
        let doc_v2 = RecipeDocument {
            id: "test-id".to_string(),
            name: "Updated IPA".to_string(),
            recipe,
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };
        let bytes_v2 = reconcile_to_automerge(&doc_v2, Some(&bytes_v1)).unwrap();

        // Verify the updated doc has the new name
        let hydrated: RecipeDocument = hydrate_from_automerge(&bytes_v2).unwrap();
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
    fn automerge_roundtrip_with_style() {
        // This reproduces the Unexpected(Map) bug: RecipeStyleType is a newtype
        // wrapper around StyleBase. When used inside Option<T>, the autosurgeon
        // derive's hydrate didn't implement hydrate_map, causing failure.
        let recipe: RecipeType = serde_json::from_str(
            r#"{
                "name": "Belgian Ale",
                "type": "all grain",
                "author": "Tester",
                "batch_size": { "unit": "l", "value": 20.0 },
                "efficiency": { "brewhouse": { "unit": "%", "value": 72.0 } },
                "style": {
                    "name": "Belgian Specialty Ale",
                    "category": "Belgian and French Ale",
                    "category_number": 16,
                    "style_guide": "BJCP",
                    "type": "beer"
                },
                "ingredients": {
                    "fermentable_additions": [],
                    "hop_additions": []
                }
            }"#,
        )
        .unwrap();

        let doc = RecipeDocument {
            id: "style-test".to_string(),
            name: "Belgian Ale".to_string(),
            recipe,
            equipment: None,
            equipment_id: None,
            is_deleted: false,
        };

        let bytes = reconcile_to_automerge(&doc, None).unwrap();
        let hydrated: RecipeDocument = hydrate_from_automerge(&bytes).unwrap();
        assert_eq!(hydrated.name, "Belgian Ale");
        assert!(hydrated.recipe.style.is_some());
        assert_eq!(hydrated.recipe.style.unwrap().name, "Belgian Specialty Ale");
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
                equipment: None,
                equipment_id: None,
                is_deleted: false,
            },
            None,
        ).unwrap();

        let row = RecipeRow {
            id: "row-id".to_string(),
            name: "Row Test".to_string(),
            recipe: serde_json::to_string(&recipe).unwrap(),
            equipment: None,
            equipment_id: None,
            am_data: am_data.clone(),
            is_deleted: false,
            is_dirty: true,
        };

        let doc = row.to_document().unwrap();
        assert_eq!(doc.id, "row-id");
        assert_eq!(doc.name, "Row Test");
        assert!(doc.equipment.is_none());

        let back = doc.to_row(am_data).unwrap();
        assert_eq!(back.id, "row-id");
        assert!(back.equipment.is_none());
    }

    #[test]
    fn row_document_conversion_with_equipment() {
        let recipe_json = sample_recipe_json();
        let recipe: RecipeType = serde_json::from_str(recipe_json).unwrap();
        let profile = brewdio_core::data::equipment()[0].clone();
        let am_data = reconcile_to_automerge(
            &RecipeDocument {
                id: "row-eq-id".to_string(),
                name: "Equipped Row".to_string(),
                recipe: recipe.clone(),
                equipment: Some(profile.equipment.clone()),
                equipment_id: Some(profile.id.clone()),
                is_deleted: false,
            },
            None,
        ).unwrap();

        let row = RecipeRow {
            id: "row-eq-id".to_string(),
            name: "Equipped Row".to_string(),
            recipe: serde_json::to_string(&recipe).unwrap(),
            equipment: Some(serde_json::to_string(&profile.equipment).unwrap()),
            equipment_id: Some(profile.id.clone()),
            am_data: am_data.clone(),
            is_deleted: false,
            is_dirty: true,
        };

        let doc = row.to_document().unwrap();
        assert!(doc.equipment.is_some());
        assert_eq!(doc.equipment.as_ref().unwrap().name, "Default Setup");
        assert_eq!(doc.equipment_id.as_deref(), Some("default-setup"));

        let back = doc.to_row(am_data).unwrap();
        assert!(back.equipment.is_some());
        assert_eq!(back.equipment_id.as_deref(), Some("default-setup"));
    }

    /// Legacy automerge documents (created before equipment_id was added)
    /// should still hydrate correctly with the missing fields defaulting to None.
    #[test]
    fn hydrate_legacy_document_without_equipment_fields() {
        // Simulate a legacy document that only has id, name, recipe, is_deleted
        // (no equipment or equipment_id keys in the automerge doc)
        #[derive(autosurgeon::Reconcile)]
        struct LegacyRecipeDocument {
            id: String,
            name: String,
            recipe: RecipeType,
            is_deleted: bool,
        }

        let recipe: RecipeType = serde_json::from_str(sample_recipe_json()).unwrap();
        let legacy = LegacyRecipeDocument {
            id: "legacy-id".to_string(),
            name: "Legacy IPA".to_string(),
            recipe,
            is_deleted: false,
        };

        // Create automerge bytes with the legacy schema (no equipment fields)
        let bytes = reconcile_to_automerge(&legacy, None).unwrap();

        // Hydrate with the current schema — should succeed thanks to #[autosurgeon(missing)]
        let hydrated: RecipeDocument = hydrate_from_automerge(&bytes).unwrap();
        assert_eq!(hydrated.id, "legacy-id");
        assert_eq!(hydrated.name, "Legacy IPA");
        assert!(hydrated.equipment.is_none());
        assert!(hydrated.equipment_id.is_none());
        assert!(!hydrated.is_deleted);
    }
}
