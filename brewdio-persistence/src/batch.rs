use brewdio_core::beerjson_types::{EquipmentType, RecipeType};
use serde::{Deserialize, Serialize};

use crate::connection::{Connection, DbError, Value};
use crate::automerge::{current_time_millis, new_ulid, reconcile_to_automerge};
use crate::protocol::DocType;
use crate::traits::{SyncDocument, SyncRow};

/// Public typed batch for external consumers.
/// JSON data field is deserialized from the underlying `BatchRow`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct Batch {
    pub id: String,
    pub name: String,
    pub recipe_id: String,
    pub data: BatchData,
}

#[derive(Debug, Clone, Serialize, Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate)]
#[serde(rename_all = "camelCase")]
pub struct BatchData {
    pub equipment_id: String,
    pub recipe: RecipeType,
    pub equipment: EquipmentType,
    pub brew_date: u64,
    pub notes: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

pub struct BatchRow {
    pub id: String,
    pub name: String,
    pub recipe_id: String,
    pub data: String,
    pub am_data: Vec<u8>,
    pub is_deleted: bool,
    pub is_dirty: bool,
}

/// Automerge-reconciled document for CRDT sync.
#[derive(Clone, Debug, Serialize, Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate)]
pub struct BatchDocument {
    pub id: String,
    pub name: String,
    pub recipe_id: String,
    pub data: BatchData,
    pub is_deleted: bool,
}

impl BatchRow {
    /// Convert to a typed `Batch` (deserializes JSON data field).
    pub fn to_batch(&self) -> Result<Batch, DbError> {
        let data: BatchData = serde_json::from_str(&self.data)
            .map_err(|e| DbError(e.to_string()))?;
        Ok(Batch {
            id: self.id.clone(),
            name: self.name.clone(),
            recipe_id: self.recipe_id.clone(),
            data,
        })
    }

    pub fn to_data(&self) -> Result<BatchData, serde_json::Error> {
        serde_json::from_str(&self.data)
    }

    pub fn to_document(&self) -> Result<BatchDocument, serde_json::Error> {
        let data: BatchData = serde_json::from_str(&self.data)?;
        Ok(BatchDocument {
            id: self.id.clone(),
            name: self.name.clone(),
            recipe_id: self.recipe_id.clone(),
            data,
            is_deleted: self.is_deleted,
        })
    }
}

impl SyncDocument for BatchDocument {
    fn id(&self) -> &str { &self.id }
    fn is_deleted(&self) -> bool { self.is_deleted }
    fn set_is_deleted(&mut self, deleted: bool) { self.is_deleted = deleted; }
}

impl SyncRow for BatchRow {
    type Document = BatchDocument;
    fn doc_type() -> DocType { DocType::Batch }
    fn id(&self) -> &str { &self.id }
    fn am_data(&self) -> &[u8] { &self.am_data }
    fn into_document(&self) -> Result<BatchDocument, DbError> {
        self.to_document().map_err(|e| DbError(e.to_string()))
    }
}

impl BatchDocument {
    pub fn to_row(&self, am_data: Vec<u8>) -> Result<BatchRow, serde_json::Error> {
        let data_json = serde_json::to_string(&self.data)?;
        Ok(BatchRow {
            id: self.id.clone(),
            name: self.name.clone(),
            recipe_id: self.recipe_id.clone(),
            data: data_json,
            am_data,
            is_deleted: self.is_deleted,
            is_dirty: true,
        })
    }
}

fn row_from_query(row: &dyn crate::connection::Row) -> BatchRow {
    BatchRow {
        id: row.get_text(0),
        name: row.get_text(1),
        recipe_id: row.get_text(2),
        data: row.get_text(3),
        am_data: row.get_blob(4),
        is_deleted: row.get_bool(5),
        is_dirty: row.get_bool(6),
    }
}

pub fn create_batch(
    conn: &(impl Connection + ?Sized),
    name: &str,
    recipe_id: &str,
    data: &str,
) -> Result<BatchRow, DbError> {
    let id = new_ulid();
    let parsed_data: BatchData =
        serde_json::from_str(data).map_err(|e| DbError(e.to_string()))?;
    let doc = BatchDocument {
        id: id.clone(),
        name: name.to_string(),
        recipe_id: recipe_id.to_string(),
        data: parsed_data,
        is_deleted: false,
    };
    let am_data = reconcile_to_automerge(&doc, None).map_err(|e| DbError(e))?;

    conn.execute(
        "INSERT INTO batch (id, name, recipe_id, data, am_data) VALUES (?1, ?2, ?3, ?4, ?5)",
        &[
            Value::Text(&id),
            Value::Text(name),
            Value::Text(recipe_id),
            Value::Text(data),
            Value::Blob(&am_data),
        ],
    )?;
    Ok(BatchRow {
        id,
        name: name.to_string(),
        recipe_id: recipe_id.to_string(),
        data: data.to_string(),
        am_data,
        is_deleted: false,
        is_dirty: true,
    })
}

pub fn get_batch_row(
    conn: &(impl Connection + ?Sized),
    id: &str,
) -> Result<Option<BatchRow>, DbError> {
    conn.query_one(
        "SELECT id, name, recipe_id, data, am_data, is_deleted, is_dirty FROM batch WHERE id = ?1",
        &[Value::Text(id)],
        row_from_query,
    )
}

pub fn list_batch_rows(conn: &(impl Connection + ?Sized)) -> Result<Vec<BatchRow>, DbError> {
    conn.query_map(
        "SELECT id, name, recipe_id, data, am_data, is_deleted, is_dirty FROM batch WHERE is_deleted = FALSE",
        &[],
        row_from_query,
    )
}

/// List all non-deleted batches as typed `Batch` structs.
pub fn list_batches(conn: &(impl Connection + ?Sized)) -> Result<Vec<Batch>, DbError> {
    list_batch_rows(conn)?
        .into_iter()
        .map(|r| r.to_batch())
        .collect()
}

/// Get a batch by ID as a typed `Batch` struct.
pub fn get_batch(
    conn: &(impl Connection + ?Sized),
    id: &str,
) -> Result<Option<Batch>, DbError> {
    get_batch_row(conn, id)?
        .filter(|r| !r.is_deleted)
        .map(|r| r.to_batch())
        .transpose()
}

pub fn update_batch(
    conn: &(impl Connection + ?Sized),
    id: &str,
    name: &str,
    data: &str,
) -> Result<(), DbError> {
    let existing = get_batch_row(conn, id)?;
    let existing_am = existing.as_ref().map(|r| r.am_data.as_slice());

    let parsed_data: BatchData =
        serde_json::from_str(data).map_err(|e| DbError(e.to_string()))?;
    let doc = BatchDocument {
        id: id.to_string(),
        name: name.to_string(),
        recipe_id: existing.as_ref().map(|r| r.recipe_id.clone()).unwrap_or_default(),
        data: parsed_data,
        is_deleted: false,
    };
    let am_data = reconcile_to_automerge(&doc, existing_am).map_err(|e| DbError(e))?;

    conn.execute(
        "UPDATE batch SET name = ?1, data = ?2, am_data = ?3, is_dirty = TRUE WHERE id = ?4",
        &[Value::Text(name), Value::Text(data), Value::Blob(&am_data), Value::Text(id)],
    )
}

pub fn delete_batch(conn: &(impl Connection + ?Sized), id: &str) -> Result<(), DbError> {
    let existing = get_batch_row(conn, id)?;
    crate::traits::set_deleted(conn, existing, id, true)
}

pub fn create_batch_from_recipe(
    conn: &(impl Connection + ?Sized),
    name: &str,
    recipe_id: &str,
    recipe: &RecipeType,
    equipment: &EquipmentType,
    equipment_profile_id: &str,
) -> Result<BatchRow, DbError> {
    let now = current_time_millis() as u64;
    let data = BatchData {
        equipment_id: equipment_profile_id.to_string(),
        recipe: recipe.clone(),
        equipment: equipment.clone(),
        brew_date: now,
        notes: None,
        created_at: now,
        updated_at: now,
    };
    let json = serde_json::to_string(&data).map_err(|e| DbError(e.to_string()))?;
    create_batch(conn, name, recipe_id, &json)
}

pub fn count_batches_for_recipe(
    conn: &(impl Connection + ?Sized),
    recipe_id: &str,
) -> Result<usize, DbError> {
    let ids = conn.query_map(
        "SELECT id FROM batch WHERE recipe_id = ?1",
        &[Value::Text(recipe_id)],
        |row| row.get_text(0),
    )?;
    Ok(ids.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> rusqlite::Connection {
        crate::connection_native::open(":memory:").unwrap()
    }

    fn sample_batch_data_json() -> String {
        serde_json::to_string(&BatchData {
            equipment_id: "kettle-1".to_string(),
            recipe: serde_json::from_str(r#"{
                "name": "Test IPA", "type": "all grain", "author": "",
                "batch_size": { "unit": "l", "value": 20.0 },
                "efficiency": { "brewhouse": { "unit": "%", "value": 72.0 } },
                "ingredients": { "fermentable_additions": [], "hop_additions": [] }
            }"#).unwrap(),
            equipment: brewdio_core::data::equipment()[0].equipment.clone(),
            brew_date: 1000,
            notes: Some("test".to_string()),
            created_at: 1000,
            updated_at: 1000,
        }).unwrap()
    }

    #[test]
    fn batch_crud() {
        let conn = test_conn();
        let data = sample_batch_data_json();

        // Create
        let row = create_batch(&conn, "Batch 1", "recipe-123", &data).unwrap();
        assert_eq!(row.name, "Batch 1");
        assert!(!row.is_deleted);
        assert!(row.is_dirty);
        assert!(!row.am_data.is_empty(), "am_data should be populated on create");

        // Get
        let fetched = get_batch_row(&conn, &row.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Batch 1");
        assert_eq!(fetched.recipe_id, "recipe-123");
        assert!(fetched.is_dirty);
        assert!(!fetched.am_data.is_empty());

        // List
        let batches = list_batch_rows(&conn).unwrap();
        assert_eq!(batches.len(), 1);

        // Update
        update_batch(&conn, &row.id, "Updated Batch", &data).unwrap();
        let fetched = get_batch_row(&conn, &row.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Updated Batch");
        assert!(!fetched.am_data.is_empty(), "am_data should be populated on update");

        // Soft-delete
        delete_batch(&conn, &row.id).unwrap();
        let batches = list_batch_rows(&conn).unwrap();
        assert_eq!(batches.len(), 0);

        // Still exists
        let fetched = get_batch_row(&conn, &row.id).unwrap().unwrap();
        assert!(fetched.is_deleted);
        assert!(fetched.is_dirty);
        assert!(!fetched.am_data.is_empty(), "am_data should be populated on delete");
    }

    #[test]
    fn create_batch_from_recipe_roundtrip() {
        let conn = test_conn();
        let profile = &brewdio_core::data::equipment()[0];
        let recipe = brewdio_core::beerjson_types::RecipeType {
            name: "Test IPA".to_string(),
            author: String::new(),
            type_: brewdio_core::beerjson_types::RecipeTypeType::AllGrain,
            batch_size: brewdio_core::beerjson_types::VolumeType {
                unit: brewdio_core::beerjson_types::VolumeUnitType::L,
                value: 20.0,
            },
            efficiency: brewdio_core::beerjson_types::EfficiencyType {
                brewhouse: brewdio_core::beerjson_types::PercentType {
                    unit: brewdio_core::beerjson_types::PercentUnitType::X,
                    value: 72.0,
                },
                conversion: None,
                lauter: None,
                mash: None,
            },
            ingredients: brewdio_core::beerjson_types::IngredientsType {
                fermentable_additions: Vec::new(),
                hop_additions: Vec::new(),
                culture_additions: Vec::new(),
                miscellaneous_additions: Vec::new(),
                water_additions: Vec::new(),
            },
            alcohol_by_volume: None,
            apparent_attenuation: None,
            beer_p_h: None,
            boil: None,
            calories_per_pint: None,
            carbonation: None,
            coauthor: None,
            color_estimate: None,
            created: None,
            fermentation: None,
            final_gravity: None,
            ibu_estimate: None,
            mash: None,
            notes: None,
            original_gravity: None,
            packaging: None,
            style: None,
            taste: None,
        };

        let row = create_batch_from_recipe(&conn, "Batch #1", "recipe-1", &recipe, &profile.equipment, &profile.id).unwrap();
        assert_eq!(row.name, "Batch #1");

        let data = row.to_data().unwrap();
        assert_eq!(data.recipe.name, "Test IPA");
        assert_eq!(data.equipment_id, profile.id);
        assert!(data.brew_date > 0);
    }
}
