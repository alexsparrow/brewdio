use wasm_bindgen::prelude::*;
use brewdio_core::beerjson_types::*;

#[wasm_bindgen(js_name = "volumeToMilliliters")]
pub fn volume_to_milliliters(volume: VolumeType) -> f64 {
    brewdio_core::units::volume_to_milliliters(&volume)
}

#[wasm_bindgen(js_name = "volumeToLiters")]
pub fn volume_to_liters(volume: VolumeType) -> f64 {
    brewdio_core::units::volume_to_liters(&volume)
}

#[wasm_bindgen(js_name = "volumeToGallons")]
pub fn volume_to_gallons(volume: VolumeType) -> f64 {
    brewdio_core::units::volume_to_gallons(&volume)
}

#[wasm_bindgen(js_name = "massToGrams")]
pub fn mass_to_grams(mass: MassType) -> f64 {
    brewdio_core::units::mass_to_grams(&mass)
}

#[wasm_bindgen(js_name = "massToKilograms")]
pub fn mass_to_kilograms(mass: MassType) -> f64 {
    brewdio_core::units::mass_to_kilograms(&mass)
}

#[wasm_bindgen(js_name = "massToPounds")]
pub fn mass_to_pounds(mass: MassType) -> f64 {
    brewdio_core::units::mass_to_pounds(&mass)
}

#[wasm_bindgen(js_name = "massToOunces")]
pub fn mass_to_ounces(mass: MassType) -> f64 {
    brewdio_core::units::mass_to_ounces(&mass)
}

#[wasm_bindgen(js_name = "timeToMinutes")]
pub fn time_to_minutes(time: TimeType) -> f64 {
    brewdio_core::units::time_to_minutes(&time)
}

#[wasm_bindgen(js_name = "timeToHours")]
pub fn time_to_hours(time: TimeType) -> f64 {
    brewdio_core::units::time_to_hours(&time)
}

#[wasm_bindgen(js_name = "temperatureToCelsius")]
pub fn temperature_to_celsius(temp: TemperatureType) -> f64 {
    brewdio_core::units::temperature_to_celsius(&temp)
}

#[wasm_bindgen(js_name = "temperatureToFahrenheit")]
pub fn temperature_to_fahrenheit(temp: TemperatureType) -> f64 {
    brewdio_core::units::temperature_to_fahrenheit(&temp)
}

#[wasm_bindgen(js_name = "colorToSrm")]
pub fn color_to_srm(color: ColorType) -> f64 {
    brewdio_core::units::color_to_srm(&color)
}

#[wasm_bindgen(js_name = "srmToEbc")]
pub fn srm_to_ebc(srm: f64) -> f64 {
    brewdio_core::units::srm_to_ebc(srm)
}

#[wasm_bindgen(js_name = "ebcToSrm")]
pub fn ebc_to_srm(ebc: f64) -> f64 {
    brewdio_core::units::ebc_to_srm(ebc)
}

#[wasm_bindgen(js_name = "lovibondToSrm")]
pub fn lovibond_to_srm(lovibond: f64) -> f64 {
    brewdio_core::units::lovibond_to_srm(lovibond)
}

#[wasm_bindgen(js_name = "srmToLovibond")]
pub fn srm_to_lovibond(srm: f64) -> f64 {
    brewdio_core::units::srm_to_lovibond(srm)
}

#[wasm_bindgen(js_name = "gravityToSg")]
pub fn gravity_to_sg(gravity: GravityType) -> f64 {
    brewdio_core::units::gravity_to_sg(&gravity)
}

#[wasm_bindgen(js_name = "gravityToPlato")]
pub fn gravity_to_plato(gravity: GravityType) -> f64 {
    brewdio_core::units::gravity_to_plato(&gravity)
}

#[wasm_bindgen(js_name = "pressureToPsi")]
pub fn pressure_to_psi(pressure: PressureType) -> f64 {
    brewdio_core::units::pressure_to_psi(&pressure)
}

#[wasm_bindgen(js_name = "pressureToBar")]
pub fn pressure_to_bar(pressure: PressureType) -> f64 {
    brewdio_core::units::pressure_to_bar(&pressure)
}

#[wasm_bindgen(js_name = "pressureToKilopascals")]
pub fn pressure_to_kilopascals(pressure: PressureType) -> f64 {
    brewdio_core::units::pressure_to_kilopascals(&pressure)
}

#[wasm_bindgen(js_name = "carbonationToVolumes")]
pub fn carbonation_to_volumes(carbonation: CarbonationType) -> f64 {
    brewdio_core::units::carbonation_to_volumes(&carbonation)
}

#[wasm_bindgen(js_name = "bitternessToIbu")]
pub fn bitterness_to_ibu(bitterness: BitternessType) -> f64 {
    brewdio_core::units::bitterness_to_ibu(&bitterness)
}

#[wasm_bindgen(js_name = "percentToDecimal")]
pub fn percent_to_decimal(percent: PercentType) -> f64 {
    brewdio_core::units::percent_to_decimal(&percent)
}

#[wasm_bindgen(js_name = "specificVolumeToGallonsPerKilogram")]
pub fn specific_volume_to_gallons_per_kilogram(specific_volume: SpecificVolumeType) -> f64 {
    brewdio_core::units::specific_volume_to_gal_per_kg(&specific_volume)
}

#[wasm_bindgen(js_name = "specificVolumeToLPerKg")]
pub fn specific_volume_to_l_per_kg(specific_volume: SpecificVolumeType) -> f64 {
    brewdio_core::units::specific_volume_to_l_per_kg(&specific_volume)
}
