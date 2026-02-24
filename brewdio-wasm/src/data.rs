use wasm_bindgen::prelude::*;
use brewdio_core::beerjson_types::*;

#[wasm_bindgen(js_name = "getFermentables")]
pub fn get_fermentables() -> Vec<FermentableType> {
    brewdio_core::data::fermentables()
}

#[wasm_bindgen(js_name = "getHops")]
pub fn get_hops() -> Vec<HopVarietyBase> {
    brewdio_core::data::hops()
}

#[wasm_bindgen(js_name = "getCultures")]
pub fn get_cultures() -> Vec<CultureInformation> {
    brewdio_core::data::cultures()
}

#[wasm_bindgen(js_name = "getStyles")]
pub fn get_styles() -> Vec<StyleType> {
    brewdio_core::data::styles()
}

#[wasm_bindgen(js_name = "getEquipment")]
pub fn get_equipment() -> Vec<EquipmentType> {
    brewdio_core::data::equipment()
}

#[wasm_bindgen(js_name = "getMashProfiles")]
pub fn get_mash_profiles() -> Vec<MashProcedureType> {
    brewdio_core::data::mash_profiles()
}

#[wasm_bindgen(js_name = "styleForRecipe")]
pub fn style_for_recipe(recipe: RecipeType) -> Option<StyleType> {
    brewdio_core::data::style_for_recipe(&recipe)
}
