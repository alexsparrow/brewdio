use wasm_bindgen::prelude::*;
use brewdio_core::beerjson_types::*;
use brewdio_core::water::{WaterCalculatorInput, WaterCalculatorResult, WaterStage, StageCalculationResult};
use brewdio_core::carbonation::{CarbonationInput, CarbonationResult};

use crate::to_js;

#[wasm_bindgen(js_name = "calculateAbv")]
pub fn calculate_abv(og: f64, fg: f64) -> f64 {
    brewdio_core::abv::calculate_abv(og, fg)
}

#[wasm_bindgen(js_name = "calculateOg")]
pub fn calculate_og(
    fermentables: Vec<FermentableAdditionType>,
    batch_size: VolumeType,
    efficiency: PercentType,
) -> f64 {
    brewdio_core::og::calculate_og(&fermentables, &batch_size, &efficiency)
}

#[wasm_bindgen(js_name = "calculateFg")]
pub fn calculate_fg(og: f64, cultures: Vec<CultureAdditionType>) -> f64 {
    brewdio_core::fg::calculate_fg(og, &cultures)
}

#[wasm_bindgen(js_name = "calculateIbu")]
pub fn calculate_ibu(hops: Vec<HopAdditionType>, batch_size: VolumeType, og: f64) -> f64 {
    brewdio_core::ibu::calculate_ibu(&hops, &batch_size, og)
}

#[wasm_bindgen(js_name = "calculateColor")]
pub fn calculate_color(fermentables: Vec<FermentableAdditionType>, batch_size: VolumeType) -> f64 {
    brewdio_core::color::calculate_color(&fermentables, &batch_size)
}

#[wasm_bindgen(js_name = "calculateWater")]
pub fn calculate_water(input: WaterCalculatorInput) -> WaterCalculatorResult {
    brewdio_core::water::calculate_water(&input)
}

#[wasm_bindgen(js_name = "calculateWaterFromStages")]
pub fn calculate_water_from_stages(
    target_volume: f64,
    boil_time: f64,
    stages: Vec<WaterStage>,
) -> StageCalculationResult {
    brewdio_core::water::calculate_water_from_stages(target_volume, boil_time, &stages)
}

#[wasm_bindgen(js_name = "calculateCarbonation")]
pub fn calculate_carbonation(input: CarbonationInput) -> CarbonationResult {
    brewdio_core::carbonation::calculate_carbonation(&input)
}

#[wasm_bindgen(js_name = "srmToSrgb")]
pub fn srm_to_srgb(srm: f64, path_cm: Option<f64>) -> JsValue {
    let rgb = brewdio_core::olfarve::srm_to_srgb(srm, path_cm);
    to_js(&rgb).unwrap()
}

#[wasm_bindgen(js_name = "ebcToSrgb")]
pub fn ebc_to_srgb(ebc: f64, path_cm: Option<f64>) -> JsValue {
    let rgb = brewdio_core::olfarve::ebc_to_srgb(ebc, path_cm);
    to_js(&rgb).unwrap()
}

#[wasm_bindgen(js_name = "rgbToHex")]
pub fn rgb_to_hex(r: f64, g: f64, b: f64) -> String {
    brewdio_core::olfarve::rgb_to_hex(&[r, g, b])
}
