use serde::Serialize;
use tsify::Tsify;
use wasm_bindgen::prelude::*;
use brewdio_core::beerjson_types::RecipeType;
use brewdio_persistence::connection_wasm::WasmConnection;
use brewdio_persistence::db;
use brewdio_persistence::batch;
use brewdio_persistence::settings;

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct RecipeDocumentJs {
    pub id: String,
    pub name: String,
    pub recipe: RecipeType,
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
/// Must be called once before creating a RecipeDb with `RecipeDb.open(path)`.
#[wasm_bindgen(js_name = "initPersistentStorage")]
pub async fn init_persistent_storage() -> Result<(), JsError> {
    brewdio_persistence::connection_wasm::install_persistent_vfs()
        .await
        .map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen]
pub struct RecipeDb {
    conn: WasmConnection,
    on_recipes_change: Option<js_sys::Function>,
    on_batches_change: Option<js_sys::Function>,
    on_settings_change: Option<js_sys::Function>,
}

#[wasm_bindgen]
impl RecipeDb {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<RecipeDb, JsError> {
        let conn = WasmConnection::open_memory().map_err(|e| JsError::new(&e.to_string()))?;
        Ok(RecipeDb { conn, on_recipes_change: None, on_batches_change: None, on_settings_change: None })
    }

    pub fn open(path: &str) -> Result<RecipeDb, JsError> {
        let conn = WasmConnection::open(path).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(RecipeDb { conn, on_recipes_change: None, on_batches_change: None, on_settings_change: None })
    }

    #[wasm_bindgen(js_name = "onRecipesChange")]
    pub fn on_recipes_change(&mut self, callback: js_sys::Function) {
        self.on_recipes_change = Some(callback);
    }

    fn notify(&self) {
        if let Some(ref cb) = self.on_recipes_change {
            let _ = cb.call0(&JsValue::NULL);
        }
    }

    fn notify_batches(&self) {
        if let Some(ref cb) = self.on_batches_change {
            let _ = cb.call0(&JsValue::NULL);
        }
    }

    fn notify_settings(&self) {
        if let Some(ref cb) = self.on_settings_change {
            let _ = cb.call0(&JsValue::NULL);
        }
    }

    #[wasm_bindgen(js_name = "onBatchesChange")]
    pub fn on_batches_change(&mut self, callback: js_sys::Function) {
        self.on_batches_change = Some(callback);
    }

    #[wasm_bindgen(js_name = "onSettingsChange")]
    pub fn on_settings_change(&mut self, callback: js_sys::Function) {
        self.on_settings_change = Some(callback);
    }

    #[wasm_bindgen(js_name = "listRecipes")]
    pub fn list_recipes(&self) -> Result<JsValue, JsError> {
        let rows = db::list_recipes(&self.conn)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let docs: Vec<serde_json::Value> = rows
            .into_iter()
            .filter_map(|r| {
                let recipe: RecipeType = serde_json::from_str(&r.recipe).ok()?;
                Some(serde_json::json!({
                    "id": r.id,
                    "name": r.name,
                    "recipe": recipe,
                }))
            })
            .collect();
        crate::to_js(&docs).map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen(js_name = "getRecipe")]
    pub fn get_recipe(&self, id: &str) -> Result<JsValue, JsError> {
        let row = db::get_recipe(&self.conn, id)
            .map_err(|e| JsError::new(&e.to_string()))?;
        match row {
            Some(r) if !r.is_deleted => {
                let recipe: RecipeType = serde_json::from_str(&r.recipe)
                    .map_err(|e| JsError::new(&e.to_string()))?;
                let doc = serde_json::json!({
                    "id": r.id,
                    "name": r.name,
                    "recipe": recipe,
                });
                crate::to_js(&doc).map_err(|e| JsError::new(&e.to_string()))
            }
            _ => Ok(JsValue::NULL),
        }
    }

    #[wasm_bindgen(js_name = "createRecipe")]
    pub fn create_recipe(&self, name: &str, recipe: RecipeType) -> Result<String, JsError> {
        let row = db::create_recipe(&self.conn, name, &recipe)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify();
        Ok(row.id)
    }

    #[wasm_bindgen(js_name = "updateRecipe")]
    pub fn update_recipe(&self, id: &str, name: &str, recipe: RecipeType) -> Result<(), JsError> {
        db::update_recipe(&self.conn, id, name, &recipe)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify();
        Ok(())
    }

    #[wasm_bindgen(js_name = "deleteRecipe")]
    pub fn delete_recipe(&self, id: &str) -> Result<(), JsError> {
        db::delete_recipe(&self.conn, id)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify();
        Ok(())
    }

    #[wasm_bindgen(js_name = "createBatch")]
    pub fn create_batch(&self, name: &str, recipe_id: &str, data: JsValue) -> Result<String, JsError> {
        let data: serde_json::Value = serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let data_json = serde_json::to_string(&data).unwrap();
        let row = batch::create_batch(&self.conn, name, recipe_id, &data_json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify_batches();
        Ok(row.id)
    }

    #[wasm_bindgen(js_name = "getBatch")]
    pub fn get_batch(&self, id: &str) -> Result<JsValue, JsError> {
        let row = batch::get_batch(&self.conn, id)
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
        let rows = batch::list_batches(&self.conn)
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
        batch::update_batch(&self.conn, id, name, &data_json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify_batches();
        Ok(())
    }

    #[wasm_bindgen(js_name = "deleteBatch")]
    pub fn delete_batch(&self, id: &str) -> Result<(), JsError> {
        batch::delete_batch(&self.conn, id)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify_batches();
        Ok(())
    }

    #[wasm_bindgen(js_name = "getSettings")]
    pub fn get_settings(&self) -> Result<JsValue, JsError> {
        let row = settings::get_settings(&self.conn)
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
        settings::save_settings(&self.conn, &data_json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify_settings();
        Ok(())
    }
}
