//! WASM bindings for the core brewing calculations.
//! Types are automatically marshalled via tsify/wasm-bindgen.

use wasm_bindgen::prelude::*;
use core::beerjson_types::*;
use core::water::WaterCalculatorInput;
use core::carbonation::CarbonationInput;
use persistence::connection_wasm::WasmConnection;
use persistence::db;
use persistence::batch;
use persistence::settings;
use persistence::sync::SyncSession as InnerSyncSession;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($arg:tt)*) => { log(&format!($($arg)*)) }
}

/// Serialize to JsValue using JSON-compatible mode (produces plain objects, not Maps).
fn to_js<T: serde::Serialize>(value: &T) -> Result<JsValue, serde_wasm_bindgen::Error> {
    value.serialize(&serde_wasm_bindgen::Serializer::json_compatible())
}

// ============================================================================
// ABV — primitive args/return
// ============================================================================

#[wasm_bindgen]
pub fn calculate_abv(og: f64, fg: f64) -> f64 {
    core::abv::calculate_abv(og, fg)
}

// ============================================================================
// OG / FG / IBU / Color — Strongly Typed API
// ============================================================================

#[wasm_bindgen]
pub fn calculate_og(
    fermentables: Vec<FermentableAdditionType>,
    batch_size: VolumeType,
    efficiency: PercentType,
) -> f64 {
    core::og::calculate_og(&fermentables, &batch_size, &efficiency)
}

#[wasm_bindgen]
pub fn calculate_fg(og: f64, cultures: Vec<CultureAdditionType>) -> f64 {
    core::fg::calculate_fg(og, &cultures)
}

#[wasm_bindgen]
pub fn calculate_ibu(hops: Vec<HopAdditionType>, batch_size: VolumeType, og: f64) -> f64 {
    core::ibu::calculate_ibu(&hops, &batch_size, og)
}

#[wasm_bindgen]
pub fn calculate_color(fermentables: Vec<FermentableAdditionType>, batch_size: VolumeType) -> f64 {
    core::color::calculate_color(&fermentables, &batch_size)
}

// ============================================================================
// Olfarve — color rendering
// ============================================================================

#[wasm_bindgen]
pub fn srm_to_srgb(srm: f64, path_cm: Option<f64>) -> JsValue {
    let rgb = core::olfarve::srm_to_srgb(srm, path_cm);
    to_js(&rgb).unwrap()
}

#[wasm_bindgen]
pub fn ebc_to_srgb(ebc: f64, path_cm: Option<f64>) -> JsValue {
    let rgb = core::olfarve::ebc_to_srgb(ebc, path_cm);
    to_js(&rgb).unwrap()
}

#[wasm_bindgen]
pub fn rgb_to_hex(r: f64, g: f64, b: f64) -> String {
    core::olfarve::rgb_to_hex(&[r, g, b])
}

// ============================================================================
// Complex Calculators
// ============================================================================

#[wasm_bindgen]
pub fn calculate_water(input: WaterCalculatorInput) -> JsValue {
    let result = core::water::calculate_water(&input);
    to_js(&result).unwrap()
}

#[wasm_bindgen]
pub fn calculate_carbonation(input: CarbonationInput) -> JsValue {
    let result = core::carbonation::calculate_carbonation(&input);
    to_js(&result).unwrap()
}

// ============================================================================
// Unit conversions
// ============================================================================

#[wasm_bindgen]
pub fn volume_to_milliliters(volume: VolumeType) -> f64 {
    core::units::volume_to_milliliters(&volume)
}

#[wasm_bindgen]
pub fn volume_to_liters(volume: VolumeType) -> f64 {
    core::units::volume_to_liters(&volume)
}

#[wasm_bindgen]
pub fn volume_to_gallons(volume: VolumeType) -> f64 {
    core::units::volume_to_gallons(&volume)
}

#[wasm_bindgen]
pub fn mass_to_grams(mass: MassType) -> f64 {
    core::units::mass_to_grams(&mass)
}

#[wasm_bindgen]
pub fn mass_to_kilograms(mass: MassType) -> f64 {
    core::units::mass_to_kilograms(&mass)
}

#[wasm_bindgen]
pub fn mass_to_pounds(mass: MassType) -> f64 {
    core::units::mass_to_pounds(&mass)
}

#[wasm_bindgen]
pub fn mass_to_ounces(mass: MassType) -> f64 {
    core::units::mass_to_ounces(&mass)
}

#[wasm_bindgen]
pub fn time_to_minutes(time: TimeType) -> f64 {
    core::units::time_to_minutes(&time)
}

#[wasm_bindgen]
pub fn time_to_hours(time: TimeType) -> f64 {
    core::units::time_to_hours(&time)
}

#[wasm_bindgen]
pub fn temperature_to_celsius(temp: TemperatureType) -> f64 {
    core::units::temperature_to_celsius(&temp)
}

#[wasm_bindgen]
pub fn temperature_to_fahrenheit(temp: TemperatureType) -> f64 {
    core::units::temperature_to_fahrenheit(&temp)
}

#[wasm_bindgen]
pub fn color_to_srm(color: ColorType) -> f64 {
    core::units::color_to_srm(&color)
}

#[wasm_bindgen]
pub fn srm_to_ebc(srm: f64) -> f64 {
    core::units::srm_to_ebc(srm)
}

#[wasm_bindgen]
pub fn ebc_to_srm(ebc: f64) -> f64 {
    core::units::ebc_to_srm(ebc)
}

#[wasm_bindgen]
pub fn lovibond_to_srm(lovibond: f64) -> f64 {
    core::units::lovibond_to_srm(lovibond)
}

#[wasm_bindgen]
pub fn srm_to_lovibond(srm: f64) -> f64 {
    core::units::srm_to_lovibond(srm)
}

#[wasm_bindgen]
pub fn gravity_to_sg(gravity: GravityType) -> f64 {
    core::units::gravity_to_sg(&gravity)
}

#[wasm_bindgen]
pub fn gravity_to_plato(gravity: GravityType) -> f64 {
    core::units::gravity_to_plato(&gravity)
}

#[wasm_bindgen]
pub fn pressure_to_psi(pressure: PressureType) -> f64 {
    core::units::pressure_to_psi(&pressure)
}

#[wasm_bindgen]
pub fn pressure_to_bar(pressure: PressureType) -> f64 {
    core::units::pressure_to_bar(&pressure)
}

#[wasm_bindgen]
pub fn pressure_to_kilopascals(pressure: PressureType) -> f64 {
    core::units::pressure_to_kilopascals(&pressure)
}

#[wasm_bindgen]
pub fn carbonation_to_volumes(carbonation: CarbonationType) -> f64 {
    core::units::carbonation_to_volumes(&carbonation)
}

#[wasm_bindgen]
pub fn bitterness_to_ibu(bitterness: BitternessType) -> f64 {
    core::units::bitterness_to_ibu(&bitterness)
}

#[wasm_bindgen]
pub fn percent_to_decimal(percent: PercentType) -> f64 {
    core::units::percent_to_decimal(&percent)
}

#[wasm_bindgen]
pub fn specific_volume_to_gallons_per_kilogram(specific_volume: SpecificVolumeType) -> f64 {
    core::units::specific_volume_to_gal_per_kg(&specific_volume)
}

#[wasm_bindgen]
pub fn specific_volume_to_l_per_kg(specific_volume: SpecificVolumeType) -> f64 {
    core::units::specific_volume_to_l_per_kg(&specific_volume)
}

// ============================================================================
// Data accessors
// ============================================================================

#[wasm_bindgen]
pub fn get_fermentables() -> Vec<FermentableType> {
    core::data::fermentables()
}

#[wasm_bindgen]
pub fn get_hops() -> Vec<HopVarietyBase> {
    core::data::hops()
}

#[wasm_bindgen]
pub fn get_cultures() -> Vec<CultureInformation> {
    core::data::cultures()
}

#[wasm_bindgen]
pub fn get_styles() -> Vec<StyleType> {
    core::data::styles()
}

#[wasm_bindgen]
pub fn get_equipment() -> Vec<EquipmentType> {
    core::data::equipment()
}

#[wasm_bindgen]
pub fn get_mash_profiles() -> Vec<MashProcedureType> {
    core::data::mash_profiles()
}

// ============================================================================
// Persistence / Sync exports
// ============================================================================

#[wasm_bindgen]
pub struct SyncSession {
    inner: InnerSyncSession,
}

#[wasm_bindgen]
impl SyncSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: InnerSyncSession::new(),
        }
    }

    /// Create a SyncSession from an existing Automerge document.
    pub fn from_bytes(am_bytes: &[u8]) -> Self {
        Self {
            inner: InnerSyncSession::from_doc_bytes(am_bytes),
        }
    }

    /// Create a SyncSession from existing doc and sync state bytes.
    pub fn from_doc_and_state(am_bytes: &[u8], state_bytes: &[u8]) -> Self {
        Self {
            inner: InnerSyncSession::from_doc_and_state(am_bytes, state_bytes),
        }
    }

    /// Reconcile a recipe document (as JSON string) into the Automerge doc.
    pub fn reconcile_json(&mut self, recipe_json: &str) {
        let doc: persistence::recipe::RecipeDocument =
            serde_json::from_str(recipe_json).expect("Failed to parse RecipeDocument JSON");
        self.inner.reconcile(&doc);
    }

    /// Hydrate the current Automerge doc to a RecipeDocument JSON string.
    pub fn hydrate_json(&self) -> String {
        let doc = self.inner.hydrate();
        serde_json::to_string(&doc).expect("Failed to serialize RecipeDocument")
    }

    /// Generate the next sync message to send to the peer.
    /// Returns `undefined` if already synced.
    pub fn generate_sync_message(&mut self) -> JsValue {
        match self.inner.generate_sync_message() {
            Some(bytes) => {
                let arr = js_sys::Uint8Array::from(bytes.as_slice());
                arr.into()
            }
            None => JsValue::UNDEFINED,
        }
    }

    /// Receive a sync message from the peer.
    /// Returns the next sync message as Uint8Array, or `undefined` if converged.
    pub fn receive_sync_message(&mut self, data: &[u8]) -> JsValue {
        match self.inner.receive_sync_message(data) {
            Ok(Some(bytes)) => {
                let arr = js_sys::Uint8Array::from(bytes.as_slice());
                arr.into()
            }
            Ok(None) => JsValue::UNDEFINED,
            Err(e) => {
                let err_msg = format!("Sync error: {}", e);
                JsValue::from_str(&err_msg)
            }
        }
    }

    /// Save the Automerge document to bytes.
    pub fn save_doc(&mut self) -> Vec<u8> {
        self.inner.save_doc()
    }

    /// Save the sync state to bytes for persistence.
    pub fn save_state(&self) -> Vec<u8> {
        self.inner.save_state()
    }

    /// Load sync state from bytes.
    pub fn load_state(&mut self, bytes: &[u8]) {
        self.inner.load_state(bytes);
    }

    /// Returns true if the sync has converged.
    pub fn is_synced(&mut self) -> bool {
        self.inner.is_synced()
    }
}

// ============================================================================
// Persistent VFS initialization
// ============================================================================

/// Install the IndexedDB-backed persistent VFS.
/// Must be called once before creating a RecipeDb with `RecipeDb.open(path)`.
/// After this call, databases opened by name will persist across page reloads.
#[wasm_bindgen]
pub async fn init_persistent_storage() -> Result<(), JsError> {
    persistence::connection_wasm::install_persistent_vfs()
        .await
        .map_err(|e| JsError::new(&e.to_string()))
}

// ============================================================================
// Recipe Database — persistent SQLite via WASM
// ============================================================================

#[wasm_bindgen]
pub struct RecipeDb {
    conn: WasmConnection,
    on_recipes_change: Option<js_sys::Function>,
    on_batches_change: Option<js_sys::Function>,
    on_settings_change: Option<js_sys::Function>,
}

#[wasm_bindgen]
impl RecipeDb {
    /// Open an in-memory database (data lost on page refresh).
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<RecipeDb, JsError> {
        let conn = WasmConnection::open_memory().map_err(|e| JsError::new(&e.to_string()))?;
        Ok(RecipeDb { conn, on_recipes_change: None, on_batches_change: None, on_settings_change: None })
    }

    /// Open a named database. If `init_persistent_storage()` was called first,
    /// the database persists to IndexedDB across page reloads.
    pub fn open(path: &str) -> Result<RecipeDb, JsError> {
        let conn = WasmConnection::open(path).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(RecipeDb { conn, on_recipes_change: None, on_batches_change: None, on_settings_change: None })
    }

    /// Register a callback to be invoked when recipe data changes.
    /// The callback receives no arguments. Use it to invalidate caches.
    pub fn on_recipes_change(&mut self, callback: js_sys::Function) {
        self.on_recipes_change = Some(callback);
    }

    fn notify(&self) {
        if let Some(ref cb) = self.on_recipes_change {
            let _ = cb.call0(&wasm_bindgen::JsValue::NULL);
        }
    }

    fn notify_batches(&self) {
        if let Some(ref cb) = self.on_batches_change {
            let _ = cb.call0(&wasm_bindgen::JsValue::NULL);
        }
    }

    fn notify_settings(&self) {
        if let Some(ref cb) = self.on_settings_change {
            let _ = cb.call0(&wasm_bindgen::JsValue::NULL);
        }
    }

    /// Register a callback for batch data changes.
    pub fn on_batches_change(&mut self, callback: js_sys::Function) {
        self.on_batches_change = Some(callback);
    }

    /// Register a callback for settings changes.
    pub fn on_settings_change(&mut self, callback: js_sys::Function) {
        self.on_settings_change = Some(callback);
    }

    /// List all non-deleted recipes. Returns an array of {id, name, recipe} objects.
    pub fn list_recipes(&self) -> Result<JsValue, JsError> {
        console_log!("[RecipeDb] list_recipes called");
        let rows = db::list_recipes(&self.conn).map_err(|e| {
            console_log!("[RecipeDb] list_recipes ERROR: {}", e);
            JsError::new(&e.to_string())
        })?;
        console_log!("[RecipeDb] list_recipes: db returned {} rows", rows.len());
        let docs: Vec<serde_json::Value> = rows
            .into_iter()
            .filter_map(|r| {
                let parse_result = serde_json::from_str::<RecipeType>(&r.recipe);
                match parse_result {
                    Ok(recipe) => Some(serde_json::json!({
                        "id": r.id,
                        "name": r.name,
                        "recipe": recipe,
                    })),
                    Err(e) => {
                        console_log!("[RecipeDb] list_recipes: failed to parse recipe JSON for id={}: {}", r.id, e);
                        console_log!("[RecipeDb]   recipe text (first 200 chars): {:.200}", r.recipe);
                        None
                    }
                }
            })
            .collect();
        console_log!("[RecipeDb] list_recipes: returning {} recipes", docs.len());
        to_js(&docs).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Get a single recipe by ID. Returns {id, name, recipe} or null.
    pub fn get_recipe(&self, id: &str) -> Result<JsValue, JsError> {
        console_log!("[RecipeDb] get_recipe called with id={}", id);
        let row = db::get_recipe(&self.conn, id).map_err(|e| {
            console_log!("[RecipeDb] get_recipe ERROR: {}", e);
            JsError::new(&e.to_string())
        })?;
        match row {
            Some(r) if !r.is_deleted => {
                console_log!("[RecipeDb] get_recipe: found recipe name={}, is_deleted={}, recipe_len={}, am_data_len={}", r.name, r.is_deleted, r.recipe.len(), r.am_data.len());
                let recipe: RecipeType =
                    serde_json::from_str(&r.recipe).map_err(|e| {
                        console_log!("[RecipeDb] get_recipe: failed to parse recipe JSON: {}", e);
                        console_log!("[RecipeDb]   recipe text (first 200 chars): {:.200}", r.recipe);
                        JsError::new(&e.to_string())
                    })?;
                let doc = serde_json::json!({
                    "id": r.id,
                    "name": r.name,
                    "recipe": recipe,
                });
                to_js(&doc).map_err(|e| JsError::new(&e.to_string()))
            }
            Some(r) => {
                console_log!("[RecipeDb] get_recipe: found recipe id={} but is_deleted={}", r.id, r.is_deleted);
                Ok(JsValue::NULL)
            }
            None => {
                console_log!("[RecipeDb] get_recipe: no recipe found for id={}", id);
                Ok(JsValue::NULL)
            }
        }
    }

    /// Create a new recipe. Takes the recipe name and a RecipeType as JsValue.
    /// Returns the created recipe's ID.
    pub fn create_recipe(&self, name: &str, recipe_js: JsValue) -> Result<String, JsError> {
        console_log!("[RecipeDb] create_recipe called with name={}", name);
        let recipe: RecipeType =
            serde_wasm_bindgen::from_value(recipe_js).map_err(|e| {
                console_log!("[RecipeDb] create_recipe: failed to deserialize RecipeType: {}", e);
                JsError::new(&e.to_string())
            })?;
        let row = db::create_recipe(&self.conn, name, &recipe).map_err(|e| {
            console_log!("[RecipeDb] create_recipe ERROR: {}", e);
            JsError::new(&e.to_string())
        })?;
        console_log!("[RecipeDb] create_recipe: created id={}, recipe_len={}, am_data_len={}", row.id, row.recipe.len(), row.am_data.len());
        self.notify();
        Ok(row.id)
    }

    /// Update an existing recipe. Takes the ID, name, and RecipeType as JsValue.
    pub fn update_recipe(&self, id: &str, name: &str, recipe_js: JsValue) -> Result<(), JsError> {
        console_log!("[RecipeDb] update_recipe called with id={}, name={}", id, name);
        let recipe: RecipeType =
            serde_wasm_bindgen::from_value(recipe_js).map_err(|e| {
                console_log!("[RecipeDb] update_recipe: failed to deserialize RecipeType: {}", e);
                JsError::new(&e.to_string())
            })?;
        db::update_recipe(&self.conn, id, name, &recipe).map_err(|e| {
            console_log!("[RecipeDb] update_recipe ERROR: {}", e);
            JsError::new(&e.to_string())
        })?;
        console_log!("[RecipeDb] update_recipe: success for id={}", id);
        self.notify();
        Ok(())
    }

    /// Soft-delete a recipe by ID.
    pub fn delete_recipe(&self, id: &str) -> Result<(), JsError> {
        console_log!("[RecipeDb] delete_recipe called with id={}", id);
        db::delete_recipe(&self.conn, id).map_err(|e| {
            console_log!("[RecipeDb] delete_recipe ERROR: {}", e);
            JsError::new(&e.to_string())
        })?;
        console_log!("[RecipeDb] delete_recipe: success for id={}", id);
        self.notify();
        Ok(())
    }

    // ========================================================================
    // Batch CRUD
    // ========================================================================

    /// Create a new batch. Returns the batch ID.
    pub fn create_batch(&self, name: &str, recipe_id: &str, data_js: JsValue) -> Result<String, JsError> {
        let data: serde_json::Value = serde_wasm_bindgen::from_value(data_js)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let data_json = serde_json::to_string(&data).unwrap();
        let row = batch::create_batch(&self.conn, name, recipe_id, &data_json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify_batches();
        Ok(row.id)
    }

    /// Get a single batch by ID. Returns the batch object or null.
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
                to_js(&doc).map_err(|e| JsError::new(&e.to_string()))
            }
            _ => Ok(JsValue::NULL),
        }
    }

    /// List all non-deleted batches.
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
        to_js(&docs).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Update a batch.
    pub fn update_batch(&self, id: &str, name: &str, data_js: JsValue) -> Result<(), JsError> {
        let data: serde_json::Value = serde_wasm_bindgen::from_value(data_js)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let data_json = serde_json::to_string(&data).unwrap();
        batch::update_batch(&self.conn, id, name, &data_json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify_batches();
        Ok(())
    }

    /// Soft-delete a batch.
    pub fn delete_batch(&self, id: &str) -> Result<(), JsError> {
        batch::delete_batch(&self.conn, id)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify_batches();
        Ok(())
    }

    // ========================================================================
    // Settings
    // ========================================================================

    /// Get settings. Returns the settings object or null.
    pub fn get_settings(&self) -> Result<JsValue, JsError> {
        let row = settings::get_settings(&self.conn)
            .map_err(|e| JsError::new(&e.to_string()))?;
        match row {
            Some(r) => {
                let data: serde_json::Value = serde_json::from_str(&r.data)
                    .unwrap_or(serde_json::Value::Null);
                to_js(&data).map_err(|e| JsError::new(&e.to_string()))
            }
            None => Ok(JsValue::NULL),
        }
    }

    /// Save settings (upsert).
    pub fn save_settings(&self, data_js: JsValue) -> Result<(), JsError> {
        let data: serde_json::Value = serde_wasm_bindgen::from_value(data_js)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let data_json = serde_json::to_string(&data).unwrap();
        settings::save_settings(&self.conn, &data_json)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.notify_settings();
        Ok(())
    }
}
