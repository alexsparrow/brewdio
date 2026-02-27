use brewdio_core::beerjson_types::{EfficiencyType, EquipmentType};
use brewdio_core::equipment_profile::EquipmentProfile;

use crate::automerge::{new_ulid, reconcile_to_automerge};
use crate::connection::{Connection, DbError, Value};

pub struct EquipmentProfileRow {
    pub id: String,
    pub name: String,
    pub data: String,
    pub efficiency: String,
    pub am_data: Vec<u8>,
    pub is_deleted: bool,
    pub is_dirty: bool,
}

/// Automerge-reconciled document for CRDT sync.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate)]
pub struct EquipmentProfileDocument {
    pub id: String,
    pub name: String,
    pub equipment: EquipmentType,
    pub efficiency: EfficiencyType,
    pub is_deleted: bool,
}

impl EquipmentProfileRow {
    pub fn to_profile(&self) -> Result<EquipmentProfile, serde_json::Error> {
        let equipment: EquipmentType = serde_json::from_str(&self.data)?;
        let efficiency: EfficiencyType = serde_json::from_str(&self.efficiency)?;
        Ok(EquipmentProfile {
            id: self.id.clone(),
            equipment,
            efficiency,
        })
    }

    pub fn to_document(&self) -> Result<EquipmentProfileDocument, serde_json::Error> {
        let equipment: EquipmentType = serde_json::from_str(&self.data)?;
        let efficiency: EfficiencyType = serde_json::from_str(&self.efficiency)?;
        Ok(EquipmentProfileDocument {
            id: self.id.clone(),
            name: self.name.clone(),
            equipment,
            efficiency,
            is_deleted: self.is_deleted,
        })
    }
}

impl EquipmentProfileDocument {
    pub fn to_row(&self, am_data: Vec<u8>) -> Result<EquipmentProfileRow, serde_json::Error> {
        let data = serde_json::to_string(&self.equipment)?;
        let efficiency = serde_json::to_string(&self.efficiency)?;
        Ok(EquipmentProfileRow {
            id: self.id.clone(),
            name: self.name.clone(),
            data,
            efficiency,
            am_data,
            is_deleted: self.is_deleted,
            is_dirty: true,
        })
    }
}

fn row_from_query(row: &dyn crate::connection::Row) -> EquipmentProfileRow {
    EquipmentProfileRow {
        id: row.get_text(0),
        name: row.get_text(1),
        data: row.get_text(2),
        efficiency: row.get_text(3),
        am_data: row.get_blob(4),
        is_deleted: row.get_bool(5),
        is_dirty: row.get_bool(6),
    }
}

pub fn create_equipment_profile(
    conn: &(impl Connection + ?Sized),
    profile: &EquipmentProfile,
) -> Result<EquipmentProfileRow, DbError> {
    let id = if profile.id.is_empty() {
        new_ulid()
    } else {
        profile.id.clone()
    };
    let data = serde_json::to_string(&profile.equipment).map_err(|e| DbError(e.to_string()))?;
    let efficiency =
        serde_json::to_string(&profile.efficiency).map_err(|e| DbError(e.to_string()))?;
    let name = profile.name().to_string();

    let doc = EquipmentProfileDocument {
        id: id.clone(),
        name: name.clone(),
        equipment: profile.equipment.clone(),
        efficiency: profile.efficiency.clone(),
        is_deleted: false,
    };
    let am_data = reconcile_to_automerge(&doc, None).map_err(|e| DbError(e))?;

    conn.execute(
        "INSERT INTO equipment_profile (id, name, data, efficiency, am_data) VALUES (?1, ?2, ?3, ?4, ?5)",
        &[
            Value::Text(&id),
            Value::Text(&name),
            Value::Text(&data),
            Value::Text(&efficiency),
            Value::Blob(&am_data),
        ],
    )?;

    Ok(EquipmentProfileRow {
        id,
        name,
        data,
        efficiency,
        am_data,
        is_deleted: false,
        is_dirty: true,
    })
}

pub fn get_equipment_profile(
    conn: &(impl Connection + ?Sized),
    id: &str,
) -> Result<Option<EquipmentProfileRow>, DbError> {
    conn.query_one(
        "SELECT id, name, data, efficiency, am_data, is_deleted, is_dirty FROM equipment_profile WHERE id = ?1",
        &[Value::Text(id)],
        row_from_query,
    )
}

pub fn list_equipment_profiles(
    conn: &(impl Connection + ?Sized),
) -> Result<Vec<EquipmentProfileRow>, DbError> {
    conn.query_map(
        "SELECT id, name, data, efficiency, am_data, is_deleted, is_dirty FROM equipment_profile WHERE is_deleted = FALSE",
        &[],
        row_from_query,
    )
}

pub fn update_equipment_profile(
    conn: &(impl Connection + ?Sized),
    id: &str,
    profile: &EquipmentProfile,
) -> Result<(), DbError> {
    let existing = get_equipment_profile(conn, id)?;
    let existing_am = existing.as_ref().map(|r| r.am_data.as_slice());

    let data = serde_json::to_string(&profile.equipment).map_err(|e| DbError(e.to_string()))?;
    let efficiency =
        serde_json::to_string(&profile.efficiency).map_err(|e| DbError(e.to_string()))?;
    let name = profile.name().to_string();

    let doc = EquipmentProfileDocument {
        id: id.to_string(),
        name: name.clone(),
        equipment: profile.equipment.clone(),
        efficiency: profile.efficiency.clone(),
        is_deleted: false,
    };
    let am_data = reconcile_to_automerge(&doc, existing_am).map_err(|e| DbError(e))?;

    conn.execute(
        "UPDATE equipment_profile SET name = ?1, data = ?2, efficiency = ?3, am_data = ?4, is_dirty = TRUE WHERE id = ?5",
        &[
            Value::Text(&name),
            Value::Text(&data),
            Value::Text(&efficiency),
            Value::Blob(&am_data),
            Value::Text(id),
        ],
    )
}

pub fn delete_equipment_profile(
    conn: &(impl Connection + ?Sized),
    id: &str,
) -> Result<(), DbError> {
    let existing = get_equipment_profile(conn, id)?;
    if let Some(row) = existing {
        let mut doc = row.to_document().map_err(|e| DbError(e.to_string()))?;
        doc.is_deleted = true;
        let am_data =
            reconcile_to_automerge(&doc, Some(&row.am_data)).map_err(|e| DbError(e))?;

        conn.execute(
            "UPDATE equipment_profile SET is_deleted = TRUE, is_dirty = TRUE, am_data = ?1 WHERE id = ?2",
            &[Value::Blob(&am_data), Value::Text(id)],
        )?;
    }
    Ok(())
}

// --- Sync-related functions ---

pub fn list_all_equipment_profile_ids(
    conn: &(impl Connection + ?Sized),
) -> Result<Vec<String>, DbError> {
    conn.query_map(
        "SELECT id FROM equipment_profile",
        &[],
        |row| row.get_text(0),
    )
}

pub fn list_dirty_equipment_profiles(
    conn: &(impl Connection + ?Sized),
) -> Result<Vec<EquipmentProfileRow>, DbError> {
    conn.query_map(
        "SELECT id, name, data, efficiency, am_data, is_deleted, is_dirty FROM equipment_profile WHERE is_dirty = TRUE",
        &[],
        row_from_query,
    )
}

pub fn clear_dirty_equipment_profile(
    conn: &(impl Connection + ?Sized),
    id: &str,
) -> Result<(), DbError> {
    conn.execute(
        "UPDATE equipment_profile SET is_dirty = FALSE WHERE id = ?1",
        &[Value::Text(id)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automerge::{hydrate_from_automerge, reconcile_to_automerge};

    fn test_conn() -> rusqlite::Connection {
        crate::connection_native::open(":memory:").unwrap()
    }

    fn sample_profile() -> EquipmentProfile {
        brewdio_core::data::equipment()[0].clone()
    }

    #[test]
    fn equipment_profile_crud() {
        let conn = test_conn();
        let profile = sample_profile();

        // Create
        let row = create_equipment_profile(&conn, &profile).unwrap();
        assert_eq!(row.name, "Default Setup");
        assert!(!row.is_deleted);
        assert!(row.is_dirty);
        assert!(!row.am_data.is_empty());

        // Get
        let fetched = get_equipment_profile(&conn, &row.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Default Setup");

        // List
        let profiles = list_equipment_profiles(&conn).unwrap();
        assert_eq!(profiles.len(), 1);

        // Update
        let mut updated = profile.clone();
        updated.equipment.name = "Updated Setup".to_string();
        updated.efficiency.brewhouse.value = 80.0;
        update_equipment_profile(&conn, &row.id, &updated).unwrap();
        let fetched = get_equipment_profile(&conn, &row.id).unwrap().unwrap();
        assert_eq!(fetched.name, "Updated Setup");
        let p = fetched.to_profile().unwrap();
        assert_eq!(p.efficiency.brewhouse.value, 80.0);

        // Soft-delete
        delete_equipment_profile(&conn, &row.id).unwrap();
        let profiles = list_equipment_profiles(&conn).unwrap();
        assert_eq!(profiles.len(), 0);

        // Still exists
        let fetched = get_equipment_profile(&conn, &row.id).unwrap().unwrap();
        assert!(fetched.is_deleted);
    }

    #[test]
    fn automerge_roundtrip() {
        let profile = sample_profile();
        let doc = EquipmentProfileDocument {
            id: "test-id".to_string(),
            name: "Test Setup".to_string(),
            equipment: profile.equipment.clone(),
            efficiency: profile.efficiency.clone(),
            is_deleted: false,
        };

        let bytes = reconcile_to_automerge(&doc, None).unwrap();
        assert!(!bytes.is_empty());

        let hydrated: EquipmentProfileDocument = hydrate_from_automerge(&bytes).unwrap();
        assert_eq!(hydrated.id, "test-id");
        assert_eq!(hydrated.name, "Test Setup");
        assert_eq!(hydrated.equipment.name, "Default Setup");
        assert_eq!(hydrated.efficiency.brewhouse.value, 72.0);
        assert!(!hydrated.is_deleted);
    }

    #[test]
    fn row_document_conversion() {
        let conn = test_conn();
        let profile = sample_profile();

        let row = create_equipment_profile(&conn, &profile).unwrap();
        let doc = row.to_document().unwrap();
        assert_eq!(doc.name, "Default Setup");
        assert_eq!(doc.equipment.name, "Default Setup");
        assert!(!doc.is_deleted);

        let back = doc.to_row(row.am_data.clone()).unwrap();
        assert_eq!(back.name, "Default Setup");
        assert!(!back.am_data.is_empty());
    }

    #[test]
    fn dirty_tracking() {
        let conn = test_conn();
        let profile = sample_profile();

        let row = create_equipment_profile(&conn, &profile).unwrap();
        let dirty = list_dirty_equipment_profiles(&conn).unwrap();
        assert_eq!(dirty.len(), 1);

        clear_dirty_equipment_profile(&conn, &row.id).unwrap();
        let dirty = list_dirty_equipment_profiles(&conn).unwrap();
        assert_eq!(dirty.len(), 0);
    }
}
