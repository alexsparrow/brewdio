use crate::connection::{Connection, DbError, Value};

pub struct SettingsRow {
    pub id: String,
    pub data: String,
    pub am_data: Vec<u8>,
    pub is_dirty: bool,
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
    conn.execute(
        "INSERT INTO settings (id, data) VALUES ('default', ?1) ON CONFLICT (id) DO UPDATE SET data = excluded.data, is_dirty = TRUE",
        &[Value::Text(data)],
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

        // Upsert
        save_settings(&conn, r#"{"vimMode":true}"#).unwrap();
        let settings = get_settings(&conn).unwrap().unwrap();
        assert_eq!(settings.data, r#"{"vimMode":true}"#);
    }
}
