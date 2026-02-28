use std::sync::Arc;
use serde::Serialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;
use brewdio_core::beerjson_types::{EquipmentType, RecipeType};
use brewdio_persistence::connection_wasm::WasmConnection;
use brewdio_persistence::db;
use brewdio_persistence::batch;
use brewdio_persistence::equipment_profile;
use brewdio_persistence::settings;

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct RecipeDocumentJs {
    pub id: String,
    pub name: String,
    pub recipe: RecipeType,
    pub equipment: Option<EquipmentType>,
    #[serde(rename = "equipmentId")]
    pub equipment_id: Option<String>,
}

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct BatchDocumentJs {
    pub id: String,
    pub name: String,
    #[serde(rename = "recipeId")]
    pub recipe_id: String,
    pub data: serde_json::Value,
}

/// Install the IndexedDB-backed persistent VFS.
/// Must be called once before creating an AppDb with `AppDb.open(path)`.
#[wasm_bindgen(js_name = "initPersistentStorage")]
pub async fn init_persistent_storage() -> Result<(), JsError> {
    brewdio_persistence::connection_wasm::install_persistent_vfs()
        .await
        .map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "AppDb")]
pub struct AppDb {
    conn: Arc<WasmConnection>,
    pub(crate) on_recipes_change: Option<js_sys::Function>,
    pub(crate) on_batches_change: Option<js_sys::Function>,
    pub(crate) on_settings_change: Option<js_sys::Function>,
    pub(crate) on_equipment_change: Option<js_sys::Function>,
}

#[wasm_bindgen(js_class = "AppDb")]
impl AppDb {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<AppDb, JsError> {
        let conn = WasmConnection::open_memory().map_err(|e| JsError::new(&e.to_string()))?;
        Ok(AppDb { conn: Arc::new(conn), on_recipes_change: None, on_batches_change: None, on_settings_change: None, on_equipment_change: None })
    }

    pub fn open(path: &str) -> Result<AppDb, JsError> {
        let conn = WasmConnection::open(path).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(AppDb { conn: Arc::new(conn), on_recipes_change: None, on_batches_change: None, on_settings_change: None, on_equipment_change: None })
    }

    pub(crate) fn conn_arc(&self) -> &Arc<WasmConnection> {
        &self.conn
    }

    #[wasm_bindgen(js_name = "onRecipesChange")]
    pub fn on_recipes_change(&mut self, callback: js_sys::Function) {
        self.on_recipes_change = Some(callback);
    }

    #[wasm_bindgen(js_name = "onBatchesChange")]
    pub fn on_batches_change(&mut self, callback: js_sys::Function) {
        self.on_batches_change = Some(callback);
    }

    #[wasm_bindgen(js_name = "onSettingsChange")]
    pub fn on_settings_change(&mut self, callback: js_sys::Function) {
        self.on_settings_change = Some(callback);
    }

    #[wasm_bindgen(js_name = "onEquipmentChange")]
    pub fn on_equipment_change(&mut self, callback: js_sys::Function) {
        self.on_equipment_change = Some(callback);
    }

    pub(crate) fn notify(&self) {
        if let Some(ref cb) = self.on_recipes_change {
            let _ = cb.call0(&JsValue::NULL);
        }
    }

    pub(crate) fn notify_batches(&self) {
        if let Some(ref cb) = self.on_batches_change {
            let _ = cb.call0(&JsValue::NULL);
        }
    }

    pub(crate) fn notify_settings(&self) {
        if let Some(ref cb) = self.on_settings_change {
            let _ = cb.call0(&JsValue::NULL);
        }
    }

    #[wasm_bindgen(js_name = "listRecipes")]
    pub fn list_recipes(&self) -> Result<JsValue, JsError> {
        let rows = db::list_recipes(&*self.conn)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let docs: Vec<serde_json::Value> = rows
            .into_iter()
            .filter_map(|r| {
                let recipe: RecipeType = serde_json::from_str(&r.recipe).ok()?;
                let equipment: Option<EquipmentType> = r.equipment.as_ref().and_then(|eq| serde_json::from_str(eq).ok());
                let mut obj = serde_json::json!({
                    "id": r.id,
                    "name": r.name,
                    "recipe": recipe,
                });
                if let Some(eq) = equipment {
                    obj["equipment"] = serde_json::to_value(eq).ok()?;
                }
                if let Some(eid) = r.equipment_id {
                    obj["equipmentId"] = serde_json::Value::String(eid);
                }
                Some(obj)
            })
            .collect();
        crate::to_js(&docs).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "getRecipe")]
    pub fn get_recipe(&self, id: &str) -> Result<JsValue, JsError> {
        let row = db::get_recipe(&*self.conn, id)
            .map_err(|e| JsError::new(&e.to_string()))?;
        match row {
            Some(r) if !r.is_deleted => {
                let recipe: RecipeType = serde_json::from_str(&r.recipe)
                    .map_err(|e| JsError::new(&e.to_string()))?;
                let equipment: Option<EquipmentType> = r.equipment.as_ref().and_then(|eq| serde_json::from_str(eq).ok());
                let mut doc = serde_json::json!({
                    "id": r.id,
                    "name": r.name,
                    "recipe": recipe,
                });
                if let Some(eq) = equipment {
                    if let Ok(val) = serde_json::to_value(eq) {
                        doc["equipment"] = val;
                    }
                }
                if let Some(eid) = r.equipment_id {
                    doc["equipmentId"] = serde_json::Value::String(eid);
                }
                crate::to_js(&doc).map_err(|e| JsError::new(&e.to_string()))
            }
            _ => Ok(JsValue::NULL),
        }
    }

    #[wasm_bindgen(js_name = "createRecipe")]
    pub fn create_recipe(&self, name: &str, recipe: RecipeType, equipment: Option<EquipmentType>, equipment_id: Option<String>) -> Result<String, JsError> {
        let row = db::create_recipe(&*self.conn, name, &recipe, equipment.as_ref(), equipment_id.as_deref())
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify();
        Ok(row.id)
    }

    #[wasm_bindgen(js_name = "updateRecipe")]
    pub fn update_recipe(&self, id: &str, name: &str, recipe: RecipeType, equipment: Option<EquipmentType>) -> Result<(), JsError> {
        db::update_recipe(&*self.conn, id, name, &recipe, equipment.as_ref())
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify();
        Ok(())
    }

    #[wasm_bindgen(js_name = "setRecipeEquipment")]
    pub fn set_recipe_equipment(&self, id: &str, equipment: Option<EquipmentType>, equipment_id: Option<String>) -> Result<(), JsError> {
        db::set_recipe_equipment(&*self.conn, id, equipment.as_ref(), equipment_id.as_deref())
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify();
        Ok(())
    }

    #[wasm_bindgen(js_name = "deleteRecipe")]
    pub fn delete_recipe(&self, id: &str) -> Result<(), JsError> {
        db::delete_recipe(&*self.conn, id)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify();
        Ok(())
    }

    #[wasm_bindgen(js_name = "createBatch")]
    pub fn create_batch(&self, name: &str, recipe_id: &str, data: JsValue) -> Result<String, JsError> {
        let data: serde_json::Value = serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let data_json = serde_json::to_string(&data).unwrap();
        let row = batch::create_batch(&*self.conn, name, recipe_id, &data_json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify_batches();
        Ok(row.id)
    }

    #[wasm_bindgen(js_name = "getBatch")]
    pub fn get_batch(&self, id: &str) -> Result<JsValue, JsError> {
        let row = batch::get_batch(&*self.conn, id)
            .map_err(|e| JsError::new(&e.to_string()))?;
        match row {
            Some(r) if !r.is_deleted => {
                let data: serde_json::Value = serde_json::from_str(&r.data)
                    .unwrap_or(serde_json::Value::Null);
                let doc = serde_json::json!({
                    "id": r.id,
                    "name": r.name,
                    "recipeId": r.recipe_id,
                    "data": data,
                });
                crate::to_js(&doc).map_err(|e| JsError::new(&e.to_string()))
            }
            _ => Ok(JsValue::NULL),
        }
    }

    #[wasm_bindgen(js_name = "listBatches")]
    pub fn list_batches(&self) -> Result<JsValue, JsError> {
        let rows = batch::list_batches(&*self.conn)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let docs: Vec<serde_json::Value> = rows
            .into_iter()
            .filter_map(|r| {
                let data: serde_json::Value = serde_json::from_str(&r.data).ok()?;
                Some(serde_json::json!({
                    "id": r.id,
                    "name": r.name,
                    "recipeId": r.recipe_id,
                    "data": data,
                }))
            })
            .collect();
        crate::to_js(&docs).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "updateBatch")]
    pub fn update_batch(&self, id: &str, name: &str, data: JsValue) -> Result<(), JsError> {
        let data: serde_json::Value = serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let data_json = serde_json::to_string(&data).unwrap();
        batch::update_batch(&*self.conn, id, name, &data_json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify_batches();
        Ok(())
    }

    #[wasm_bindgen(js_name = "deleteBatch")]
    pub fn delete_batch(&self, id: &str) -> Result<(), JsError> {
        batch::delete_batch(&*self.conn, id)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify_batches();
        Ok(())
    }

    #[wasm_bindgen(js_name = "getSettings")]
    pub fn get_settings(&self) -> Result<JsValue, JsError> {
        let row = settings::get_settings(&*self.conn)
            .map_err(|e| JsError::new(&e.to_string()))?;
        match row {
            Some(r) => {
                let data: serde_json::Value = serde_json::from_str(&r.data)
                    .unwrap_or(serde_json::Value::Null);
                crate::to_js(&data).map_err(|e| JsError::new(&e.to_string()))
            }
            None => Ok(JsValue::NULL),
        }
    }

    #[wasm_bindgen(js_name = "saveSettings")]
    pub fn save_settings(&self, data: JsValue) -> Result<(), JsError> {
        let data: serde_json::Value = serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let data_json = serde_json::to_string(&data).unwrap();
        settings::save_settings(&*self.conn, &data_json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify_settings();
        Ok(())
    }

    // --- Equipment Profile operations ---

    #[wasm_bindgen(js_name = "listEquipmentProfiles")]
    pub fn list_equipment_profiles(&self) -> Result<JsValue, JsError> {
        let rows = equipment_profile::list_equipment_profiles(&*self.conn)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let profiles: Vec<serde_json::Value> = rows
            .into_iter()
            .filter_map(|r| {
                let p = r.to_profile().ok()?;
                serde_json::to_value(&p).ok()
            })
            .collect();
        crate::to_js(&profiles).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "createEquipmentProfile")]
    pub fn create_equipment_profile(&self, profile: brewdio_core::equipment_profile::EquipmentProfile) -> Result<String, JsError> {
        let row = equipment_profile::create_equipment_profile(&*self.conn, &profile)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(row.id)
    }

    #[wasm_bindgen(js_name = "updateEquipmentProfile")]
    pub fn update_equipment_profile(&self, id: &str, profile: brewdio_core::equipment_profile::EquipmentProfile) -> Result<(), JsError> {
        equipment_profile::update_equipment_profile(&*self.conn, id, &profile)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "deleteEquipmentProfile")]
    pub fn delete_equipment_profile(&self, id: &str) -> Result<(), JsError> {
        equipment_profile::delete_equipment_profile(&*self.conn, id)
            .map_err(|e| JsError::new(&e.to_string()))
    }
}
