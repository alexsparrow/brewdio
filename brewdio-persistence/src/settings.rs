use serde::{Deserialize, Serialize};

use crate::connection::{Connection, DbError, Value};
use crate::automerge::reconcile_to_automerge;

pub struct SettingsRow {
    pub id: String,
    pub data: String,
    pub am_data: Vec<u8>,
    pub is_dirty: bool,
}

/// Automerge-reconciled document for CRDT sync.
/// Settings data is stored as a JSON string since the schema is user-defined
/// and doesn't have a fixed Rust type with autosurgeon derives.
#[derive(Clone, Debug, Serialize, Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate)]
pub struct SettingsDocument {
    pub id: String,
    pub data: String,
    pub is_dirty: bool,
}

impl SettingsRow {
    pub fn to_document(&self) -> SettingsDocument {
        SettingsDocument {
            id: self.id.clone(),
            data: self.data.clone(),
            is_dirty: self.is_dirty,
        }
    }
}

pub fn get_settings(conn: &(impl Connection + ?Sized)) -> Result<Option<SettingsRow>, DbError> {
    conn.query_one(
        "SELECT id, data, am_data, is_dirty FROM settings WHERE id = 'default'",
        &[],
        |row| SettingsRow {
            id: row.get_text(0),
            data: row.get_text(1),
            am_data: row.get_blob(2),
            is_dirty: row.get_bool(3),
        },
    )
}

pub fn save_settings(conn: &(impl Connection + ?Sized), data: &str) -> Result<(), DbError> {
    let existing = get_settings(conn)?;
    let existing_am = existing.as_ref().map(|r| r.am_data.as_slice());

    let doc = SettingsDocument {
        id: "default".to_string(),
        data: data.to_string(),
        is_dirty: true,
    };
    let am_data = reconcile_to_automerge(&doc, existing_am);

    conn.execute(
        "INSERT INTO settings (id, data, am_data) VALUES ('default', ?1, ?2) ON CONFLICT (id) DO UPDATE SET data = excluded.data, am_data = excluded.am_data, is_dirty = TRUE",
        &[Value::Text(data), Value::Blob(&am_data)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> rusqlite::Connection {
        crate::connection_native::open(":memory:").unwrap()
    }

    #[test]
    fn settings_crud() {
        let conn = test_conn();

        // Initially empty
        let settings = get_settings(&conn).unwrap();
        assert!(settings.is_none());

        // Save
        save_settings(&conn, r#"{"vimMode":false}"#).unwrap();
        let settings = get_settings(&conn).unwrap().unwrap();
        assert_eq!(settings.data, r#"{"vimMode":false}"#);
        assert!(settings.is_dirty);
        assert!(!settings.am_data.is_empty(), "am_data should be populated on save");

        // Upsert
        save_settings(&conn, r#"{"vimMode":true}"#).unwrap();
        let settings = get_settings(&conn).unwrap().unwrap();
        assert_eq!(settings.data, r#"{"vimMode":true}"#);
        assert!(!settings.am_data.is_empty(), "am_data should be populated on upsert");
    }
}
