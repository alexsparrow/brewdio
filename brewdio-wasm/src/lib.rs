//! WASM bindings for the core brewing calculations.
//! Types are automatically marshalled via tsify/wasm-bindgen.

use wasm_bindgen::prelude::*;
use core::beerjson_types::*;
use core::water::WaterCalculatorInput;
use core::carbonation::CarbonationInput;
use persistence::sync::SyncSession as InnerSyncSession;

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
    serde_wasm_bindgen::to_value(&rgb).unwrap()
}

#[wasm_bindgen]
pub fn ebc_to_srgb(ebc: f64, path_cm: Option<f64>) -> JsValue {
    let rgb = core::olfarve::ebc_to_srgb(ebc, path_cm);
    serde_wasm_bindgen::to_value(&rgb).unwrap()
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
    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn calculate_carbonation(input: CarbonationInput) -> JsValue {
    let result = core::carbonation::calculate_carbonation(&input);
    serde_wasm_bindgen::to_value(&result).unwrap()
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
