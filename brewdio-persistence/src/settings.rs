use serde::{Deserialize, Serialize};

use crate::connection::{Connection, DbError, Value};
use crate::automerge::reconcile_to_automerge;

pub struct SettingsRow {
    pub id: String,
    pub data: String,
    pub am_data: Vec<u8>,
    pub is_dirty: bool,
}

fn default_id() -> String {
    "default".to_string()
}

fn default_false() -> bool {
    false
}

fn default_empty_string() -> String {
    String::new()
}

fn serde_default_volume_unit() -> String {
    "gal".to_string()
}

fn serde_default_mass_unit() -> String {
    "lb".to_string()
}

fn serde_default_temperature_unit() -> String {
    "F".to_string()
}

fn serde_default_author() -> String {
    "Brewdio User".to_string()
}

fn missing_volume_unit() -> String {
    "gal".to_string()
}

fn missing_mass_unit() -> String {
    "lb".to_string()
}

fn missing_temperature_unit() -> String {
    "F".to_string()
}

fn missing_author() -> String {
    "Brewdio User".to_string()
}

/// Typed settings document with per-field CRDT semantics.
/// Each field is tracked independently by automerge, so concurrent edits
/// to different settings merge cleanly instead of conflicting.
#[derive(Clone, Debug, Serialize, Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDocument {
    #[serde(default = "default_id")]
    pub id: String,
    #[serde(default)]
    #[autosurgeon(missing = "default_false")]
    pub vim_mode: bool,
    #[serde(default = "serde_default_volume_unit")]
    #[autosurgeon(missing = "missing_volume_unit")]
    pub default_volume_unit: String,
    #[serde(default = "serde_default_mass_unit")]
    #[autosurgeon(missing = "missing_mass_unit")]
    pub default_mass_unit: String,
    #[serde(default = "serde_default_temperature_unit")]
    #[autosurgeon(missing = "missing_temperature_unit")]
    pub default_temperature_unit: String,
    #[serde(default)]
    #[autosurgeon(missing = "default_empty_string")]
    pub openai_api_key: String,
    #[serde(default = "serde_default_author")]
    #[autosurgeon(missing = "missing_author")]
    pub default_author: String,
    #[serde(default)]
    #[autosurgeon(missing = "default_empty_string")]
    pub ai_base_url: String,
    #[serde(default)]
    #[autosurgeon(missing = "default_empty_string")]
    pub ai_model: String,
}

impl Default for SettingsDocument {
    fn default() -> Self {
        Self {
            id: default_id(),
            vim_mode: false,
            default_volume_unit: serde_default_volume_unit(),
            default_mass_unit: serde_default_mass_unit(),
            default_temperature_unit: serde_default_temperature_unit(),
            openai_api_key: String::new(),
            default_author: serde_default_author(),
            ai_base_url: String::new(),
            ai_model: String::new(),
        }
    }
}

// --- Settings metadata registry ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingKind {
    Bool,
    VolumeUnit,
    MassUnit,
    TemperatureUnit,
    Secret,
    Text,
}

pub struct SettingDescriptor {
    pub key: &'static str,
    pub kind: SettingKind,
    pub description: &'static str,
}

pub const SETTINGS_DESCRIPTORS: &[SettingDescriptor] = &[
    SettingDescriptor { key: "vimMode", kind: SettingKind::Bool, description: "Enable vim-style keybindings (h/j/k/l navigation)" },
    SettingDescriptor { key: "defaultVolumeUnit", kind: SettingKind::VolumeUnit, description: "Default unit for volume measurements" },
    SettingDescriptor { key: "defaultMassUnit", kind: SettingKind::MassUnit, description: "Default unit for mass/weight measurements" },
    SettingDescriptor { key: "defaultTemperatureUnit", kind: SettingKind::TemperatureUnit, description: "Default unit for temperature readings" },
    SettingDescriptor { key: "openaiApiKey", kind: SettingKind::Secret, description: "API key for AI assistant" },
    SettingDescriptor { key: "defaultAuthor", kind: SettingKind::Text, description: "Default author name for new recipes" },
    SettingDescriptor { key: "aiBaseUrl", kind: SettingKind::Text, description: "AI API base URL (blank for OpenAI)" },
    SettingDescriptor { key: "aiModel", kind: SettingKind::Text, description: "AI model name (blank for gpt-4o-mini)" },
];

impl SettingsDocument {
    /// Get a setting value by its camelCase key name.
    pub fn get_value(&self, key: &str) -> String {
        match key {
            "vimMode" => self.vim_mode.to_string(),
            "defaultVolumeUnit" => self.default_volume_unit.clone(),
            "defaultMassUnit" => self.default_mass_unit.clone(),
            "defaultTemperatureUnit" => self.default_temperature_unit.clone(),
            "openaiApiKey" => self.openai_api_key.clone(),
            "defaultAuthor" => self.default_author.clone(),
            "aiBaseUrl" => self.ai_base_url.clone(),
            "aiModel" => self.ai_model.clone(),
            _ => String::new(),
        }
    }

    /// Set a setting value by its camelCase key name.
    pub fn set_value(&mut self, key: &str, value: &str) {
        match key {
            "vimMode" => self.vim_mode = value == "true",
            "defaultVolumeUnit" => self.default_volume_unit = value.to_string(),
            "defaultMassUnit" => self.default_mass_unit = value.to_string(),
            "defaultTemperatureUnit" => self.default_temperature_unit = value.to_string(),
            "openaiApiKey" => self.openai_api_key = value.to_string(),
            "defaultAuthor" => self.default_author = value.to_string(),
            "aiBaseUrl" => self.ai_base_url = value.to_string(),
            "aiModel" => self.ai_model = value.to_string(),
            _ => {}
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

/// Get the typed settings document, migrating from old JSON format if needed.
pub fn get_settings_document(conn: &(impl Connection + ?Sized)) -> Result<SettingsDocument, DbError> {
    let row = get_settings(conn)?;
    match row {
        Some(r) => {
            let doc: SettingsDocument = serde_json::from_str(&r.data)
                .unwrap_or_default();
            Ok(SettingsDocument { id: "default".to_string(), ..doc })
        }
        None => Ok(SettingsDocument { id: "default".to_string(), ..Default::default() }),
    }
}

pub fn save_settings(conn: &(impl Connection + ?Sized), doc: &SettingsDocument) -> Result<(), DbError> {
    let existing = get_settings(conn)?;
    let existing_am = existing
        .as_ref()
        .map(|r| r.am_data.as_slice())
        .filter(|am| !am.is_empty());

    let data_json = serde_json::to_string(doc).unwrap_or_else(|_| "{}".to_string());
    let am_data = reconcile_to_automerge(doc, existing_am).map_err(|e| DbError(e))?;

    conn.execute(
        "INSERT INTO settings (id, data, am_data) VALUES ('default', ?1, ?2) ON CONFLICT (id) DO UPDATE SET data = excluded.data, am_data = excluded.am_data, is_dirty = TRUE",
        &[Value::Text(&data_json), Value::Blob(&am_data)],
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

        // get_settings_document returns defaults when no row exists
        let doc = get_settings_document(&conn).unwrap();
        assert_eq!(doc.vim_mode, false);
        assert_eq!(doc.default_volume_unit, "gal");

        // Save
        let mut doc = SettingsDocument { id: "default".to_string(), ..Default::default() };
        doc.vim_mode = false;
        save_settings(&conn, &doc).unwrap();
        let settings = get_settings(&conn).unwrap().unwrap();
        assert!(settings.data.contains("vimMode"));
        assert!(settings.is_dirty);
        assert!(!settings.am_data.is_empty(), "am_data should be populated on save");

        // Upsert
        doc.vim_mode = true;
        save_settings(&conn, &doc).unwrap();
        let loaded = get_settings_document(&conn).unwrap();
        assert_eq!(loaded.vim_mode, true);
        let settings = get_settings(&conn).unwrap().unwrap();
        assert!(!settings.am_data.is_empty(), "am_data should be populated on upsert");
    }

    #[test]
    fn settings_migration_from_old_json() {
        let conn = test_conn();

        // Simulate old-format data (plain JSON string in data column)
        conn.execute_batch(
            "INSERT INTO settings (id, data, am_data, is_dirty) VALUES ('default', '{\"vimMode\":true,\"defaultVolumeUnit\":\"l\",\"openaiApiKey\":\"sk-test\"}', X'', FALSE)"
        ).unwrap();

        // get_settings_document should parse old format correctly
        let doc = get_settings_document(&conn).unwrap();
        assert_eq!(doc.vim_mode, true);
        assert_eq!(doc.default_volume_unit, "l");
        assert_eq!(doc.openai_api_key, "sk-test");
        // Missing fields should get defaults
        assert_eq!(doc.default_mass_unit, "lb");
        assert_eq!(doc.default_temperature_unit, "F");
        assert_eq!(doc.default_author, "Brewdio User");
    }

    #[test]
    fn settings_get_set_value() {
        let mut doc = SettingsDocument { id: "default".to_string(), ..Default::default() };

        assert_eq!(doc.get_value("vimMode"), "false");
        doc.set_value("vimMode", "true");
        assert_eq!(doc.vim_mode, true);
        assert_eq!(doc.get_value("vimMode"), "true");

        doc.set_value("defaultVolumeUnit", "l");
        assert_eq!(doc.default_volume_unit, "l");
    }
}
