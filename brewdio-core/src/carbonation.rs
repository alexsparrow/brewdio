//! Carbonation Calculator
//! Calculates CO2 for both priming sugar and force carbonation.
//!
//! - Residual CO2 estimated from temperature (Fix, 1999 formula)
//! - Priming sugar: corn sugar, table sugar, DME
//! - Forced carbonation: ASBC formula for CO2 pressure

use crate::beerjson_types::*;
use crate::units::{carbonation_to_volumes, temperature_to_celsius, temperature_to_fahrenheit, volume_to_liters};
use crate::brewing_model;

// ============================================================================
// TYPES
// ============================================================================

brewing_model! {
pub struct CarbonationInput {
    pub target_carbonation: CarbonationType,
    pub beer_temperature: TemperatureType,
    pub beer_volume: VolumeType,
    pub units: Option<CarbonationOutputOptions>,
}
}

brewing_model! {
pub struct CarbonationOutputOptions {
    pub mass_unit: MassUnitType,
    pub pressure_unit: PressureUnitType,
}
}

brewing_model!{
pub struct PrimingSugarResult {
    pub corn_sugar: MassType,
    pub table_sugar: MassType,
    pub dme: MassType,
}
}


brewing_model!{
pub struct ForcedCarbonationResult {
    pub pressure: PressureType,
}
}

brewing_model!{
pub struct CarbonationResult {
    pub residual_co2: CarbonationType,
    pub priming_sugar: PrimingSugarResult,
    pub forced_carbonation: ForcedCarbonationResult,
}
}

// ============================================================================
// RESIDUAL CO2
// ============================================================================

/// Estimate residual CO2 volumes dissolved in beer after fermentation,
/// based on beer temperature. Uses the simplified approximation from
/// "Principles of Brewing Science" (Fix, 1999).
///
/// Formula expects temperature in °F.
/// At typical ale fermentation temps (18–22 C / 64–72 F) this gives ~0.80–0.90 vols.
fn residual_co2_volumes(temp_f: f64) -> f64 {
    3.0378 - 0.050062 * temp_f + 0.00026555 * temp_f * temp_f
}

// ============================================================================
// PRIMING SUGAR
// ============================================================================

// Grams of CO2 per volume of CO2 per litre of beer
const CO2_GRAMS_PER_VOL_PER_LITRE: f64 = 1.96;

// Grams of sugar needed to produce 1 g of CO2
const CORN_SUGAR_FACTOR: f64 = 1.0 / 0.510; // ~1.96 — corn sugar (dextrose) is ~51% fermentable by weight for CO2
const TABLE_SUGAR_FACTOR: f64 = 1.0 / 0.538; // sucrose is slightly more efficient
const DME_FACTOR: f64 = 1.0 / 0.408; // dry malt extract

fn grams_to_mass(g: f64, unit: &MassUnitType) -> MassType {
    let value = match unit {
        MassUnitType::G => g,
        MassUnitType::Kg => g / 1000.0,
        MassUnitType::Oz => g / 28.3495,
        MassUnitType::Lb => g / 453.592,
        MassUnitType::Mg => g * 1000.0,
    };
    MassType {
        value,
        unit: unit.clone(),
    }
}

fn calc_priming_sugar(
    target_vols: f64,
    residual_vols: f64,
    beer_litres: f64,
    mass_unit: &MassUnitType,
) -> PrimingSugarResult {
    let needed_vols = (target_vols - residual_vols).max(0.0);
    let co2_grams = needed_vols * CO2_GRAMS_PER_VOL_PER_LITRE * beer_litres;

    PrimingSugarResult {
        corn_sugar: grams_to_mass(co2_grams * CORN_SUGAR_FACTOR, mass_unit),
        table_sugar: grams_to_mass(co2_grams * TABLE_SUGAR_FACTOR, mass_unit),
        dme: grams_to_mass(co2_grams * DME_FACTOR, mass_unit),
    }
}

// ============================================================================
// FORCED CARBONATION
// ============================================================================

/// Calculate the CO2 pressure (PSI) required to force-carbonate beer
/// to a target carbonation level at a given temperature.
///
/// Uses the formula from the American Society of Brewing Chemists:
///   P = -16.6999 - 0.0101059 * T + 0.00116512 * T^2
///       + 0.173354 * T * V + 4.24267 * V - 0.0684226 * V^2
///
/// where T = temperature in °F, V = target volumes CO2, P = gauge PSI.
fn forced_carb_psi(target_vols: f64, temp_c: f64) -> f64 {
    let temp_f = temp_c * (9.0 / 5.0) + 32.0;
    let psi = -16.6999 - 0.0101059 * temp_f + 0.00116512 * temp_f * temp_f
        + 0.173354 * temp_f * target_vols
        + 4.24267 * target_vols
        - 0.0684226 * target_vols * target_vols;
    psi.max(0.0)
}

fn psi_to_pressure(psi: f64, unit: &PressureUnitType) -> PressureType {
    let value = match unit {
        PressureUnitType::Psi => psi,
        PressureUnitType::KPa => psi * 6.89476,
        PressureUnitType::Bar => psi * 0.0689476,
    };
    PressureType {
        value,
        unit: unit.clone(),
    }
}

// ============================================================================
// MAIN CALCULATION
// ============================================================================

pub fn calculate_carbonation(input: &CarbonationInput) -> CarbonationResult {
    let mass_unit = input
        .units
        .as_ref()
        .map(|u| u.mass_unit.clone())
        .unwrap_or(MassUnitType::G);
    let pressure_unit = input
        .units
        .as_ref()
        .map(|u| u.pressure_unit.clone())
        .unwrap_or(PressureUnitType::KPa);

    let target_vols = carbonation_to_volumes(&input.target_carbonation);
    let temp_c = temperature_to_celsius(&input.beer_temperature);
    let temp_f = temperature_to_fahrenheit(&input.beer_temperature);
    let litres = volume_to_liters(&input.beer_volume);

    let residual_vols = residual_co2_volumes(temp_f);

    CarbonationResult {
        residual_co2: CarbonationType {
            value: residual_vols,
            unit: CarbonationUnitType::Vols,
        },
        priming_sugar: calc_priming_sugar(target_vols, residual_vols, litres, &mass_unit),
        forced_carbonation: ForcedCarbonationResult {
            pressure: psi_to_pressure(forced_carb_psi(target_vols, temp_c), &pressure_unit),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // HELPERS
    // ========================================================================

    struct InputBuilder {
        target_carbonation: CarbonationType,
        beer_temperature: TemperatureType,
        beer_volume: VolumeType,
        units: Option<CarbonationOutputOptions>,
    }

    impl InputBuilder {
        fn typical() -> Self {
            Self {
                target_carbonation: CarbonationType { value: 2.5, unit: CarbonationUnitType::Vols },
                beer_temperature: TemperatureType { value: 20.0, unit: TemperatureUnitType::C },
                beer_volume: VolumeType { value: 19.0, unit: VolumeUnitType::L },
                units: None,
            }
        }

        fn calculate(&self) -> CarbonationResult {
            let input = CarbonationInput {
                target_carbonation: self.target_carbonation.clone(),
                beer_temperature: self.beer_temperature.clone(),
                beer_volume: self.beer_volume.clone(),
                units: self.units.as_ref().map(|u| CarbonationOutputOptions {
                    mass_unit: u.mass_unit.clone(),
                    pressure_unit: u.pressure_unit.clone(),
                }),
            };
            calculate_carbonation(&input)
        }
    }

    // ========================================================================
    // TESTS
    // ========================================================================

    mod residual_co2 {
        use super::*;

        #[test]
        fn estimates_residual_co2_at_typical_ale_temp() {
            let result = InputBuilder::typical().calculate();
            // At 20°C, residual CO2 ≈ 0.84 vols
            assert!(result.residual_co2.value > 0.7);
            assert!(result.residual_co2.value < 1.0);
            assert!(matches!(result.residual_co2.unit, CarbonationUnitType::Vols));
        }

        #[test]
        fn colder_beer_has_more_residual_co2() {
            let mut warm = InputBuilder::typical();
            warm.beer_temperature = TemperatureType { value: 22.0, unit: TemperatureUnitType::C };
            let warm_result = warm.calculate();

            let mut cold = InputBuilder::typical();
            cold.beer_temperature = TemperatureType { value: 10.0, unit: TemperatureUnitType::C };
            let cold_result = cold.calculate();

            assert!(cold_result.residual_co2.value > warm_result.residual_co2.value);
        }
    }

    mod priming_sugar {
        use super::*;

        #[test]
        fn calculates_corn_sugar_for_typical_19l_batch_at_25_vols() {
            let result = InputBuilder::typical().calculate();
            // ~2.5 - ~0.84 = ~1.66 vols needed
            // CO2 grams = 1.66 * 1.96 * 19 ≈ 61.8 g CO2
            // Corn sugar = 61.8 / 0.51 ≈ 121 g
            assert!(result.priming_sugar.corn_sugar.value > 100.0);
            assert!(result.priming_sugar.corn_sugar.value < 140.0);
            assert!(matches!(result.priming_sugar.corn_sugar.unit, MassUnitType::G));
        }

        #[test]
        fn table_sugar_is_less_than_corn_sugar() {
            let result = InputBuilder::typical().calculate();
            assert!(result.priming_sugar.table_sugar.value < result.priming_sugar.corn_sugar.value);
        }

        #[test]
        fn dme_is_more_than_corn_sugar() {
            let result = InputBuilder::typical().calculate();
            assert!(result.priming_sugar.dme.value > result.priming_sugar.corn_sugar.value);
        }

        #[test]
        fn larger_batch_needs_more_sugar() {
            let mut small = InputBuilder::typical();
            small.beer_volume = VolumeType { value: 10.0, unit: VolumeUnitType::L };
            let small_result = small.calculate();

            let mut large = InputBuilder::typical();
            large.beer_volume = VolumeType { value: 20.0, unit: VolumeUnitType::L };
            let large_result = large.calculate();

            assert!(large_result.priming_sugar.corn_sugar.value > small_result.priming_sugar.corn_sugar.value);
        }

        #[test]
        fn higher_target_carbonation_needs_more_sugar() {
            let mut low = InputBuilder::typical();
            low.target_carbonation = CarbonationType { value: 2.0, unit: CarbonationUnitType::Vols };
            let low_result = low.calculate();

            let mut high = InputBuilder::typical();
            high.target_carbonation = CarbonationType { value: 3.0, unit: CarbonationUnitType::Vols };
            let high_result = high.calculate();

            assert!(high_result.priming_sugar.corn_sugar.value > low_result.priming_sugar.corn_sugar.value);
        }

        #[test]
        fn warmer_beer_needs_more_sugar() {
            let mut cold = InputBuilder::typical();
            cold.beer_temperature = TemperatureType { value: 15.0, unit: TemperatureUnitType::C };
            let cold_result = cold.calculate();

            let mut warm = InputBuilder::typical();
            warm.beer_temperature = TemperatureType { value: 25.0, unit: TemperatureUnitType::C };
            let warm_result = warm.calculate();

            assert!(warm_result.priming_sugar.corn_sugar.value > cold_result.priming_sugar.corn_sugar.value);
        }

        #[test]
        fn sugar_is_zero_when_target_equals_residual_co2() {
            // At 0°C, residual CO2 ≈ 3.04 vols; target below that should give 0
            let mut builder = InputBuilder::typical();
            builder.target_carbonation = CarbonationType { value: 1.0, unit: CarbonationUnitType::Vols };
            builder.beer_temperature = TemperatureType { value: 0.0, unit: TemperatureUnitType::C };
            let result = builder.calculate();
            assert!(result.priming_sugar.corn_sugar.value.abs() < 1e-5);
        }
    }

    mod forced_carbonation {
        use super::*;

        #[test]
        fn calculates_pressure_for_typical_serving_temp() {
            // At ~2°C / 36°F and 2.5 vols, expect roughly 11–13 PSI
            let mut builder = InputBuilder::typical();
            builder.beer_temperature = TemperatureType { value: 2.0, unit: TemperatureUnitType::C };
            builder.units = Some(CarbonationOutputOptions {
                mass_unit: MassUnitType::G,
                pressure_unit: PressureUnitType::Psi,
            });
            let result = builder.calculate();
            assert!(result.forced_carbonation.pressure.value > 9.0);
            assert!(result.forced_carbonation.pressure.value < 15.0);
            assert!(matches!(result.forced_carbonation.pressure.unit, PressureUnitType::Psi));
        }

        #[test]
        fn higher_temperature_needs_more_pressure() {
            let psi_units = || Some(CarbonationOutputOptions {
                mass_unit: MassUnitType::G,
                pressure_unit: PressureUnitType::Psi,
            });

            let mut cold = InputBuilder::typical();
            cold.beer_temperature = TemperatureType { value: 2.0, unit: TemperatureUnitType::C };
            cold.units = psi_units();
            let cold_result = cold.calculate();

            let mut warm = InputBuilder::typical();
            warm.beer_temperature = TemperatureType { value: 10.0, unit: TemperatureUnitType::C };
            warm.units = psi_units();
            let warm_result = warm.calculate();

            assert!(warm_result.forced_carbonation.pressure.value > cold_result.forced_carbonation.pressure.value);
        }

        #[test]
        fn higher_target_carbonation_needs_more_pressure() {
            let psi_units = || Some(CarbonationOutputOptions {
                mass_unit: MassUnitType::G,
                pressure_unit: PressureUnitType::Psi,
            });

            let mut low = InputBuilder::typical();
            low.target_carbonation = CarbonationType { value: 2.0, unit: CarbonationUnitType::Vols };
            low.units = psi_units();
            let low_result = low.calculate();

            let mut high = InputBuilder::typical();
            high.target_carbonation = CarbonationType { value: 3.0, unit: CarbonationUnitType::Vols };
            high.units = psi_units();
            let high_result = high.calculate();

            assert!(high_result.forced_carbonation.pressure.value > low_result.forced_carbonation.pressure.value);
        }

        #[test]
        fn pressure_is_never_negative() {
            let mut builder = InputBuilder::typical();
            builder.target_carbonation = CarbonationType { value: 0.5, unit: CarbonationUnitType::Vols };
            builder.beer_temperature = TemperatureType { value: 0.0, unit: TemperatureUnitType::C };
            builder.units = Some(CarbonationOutputOptions {
                mass_unit: MassUnitType::G,
                pressure_unit: PressureUnitType::Psi,
            });
            let result = builder.calculate();
            assert!(result.forced_carbonation.pressure.value >= 0.0);
        }
    }

    mod output_units {
        use super::*;

        #[test]
        fn defaults_to_grams_and_kpa() {
            let result = InputBuilder::typical().calculate();
            assert!(matches!(result.priming_sugar.corn_sugar.unit, MassUnitType::G));
            assert!(matches!(result.forced_carbonation.pressure.unit, PressureUnitType::KPa));
        }

        #[test]
        fn returns_ounces_when_requested() {
            let mut builder = InputBuilder::typical();
            builder.units = Some(CarbonationOutputOptions {
                mass_unit: MassUnitType::Oz,
                pressure_unit: PressureUnitType::KPa,
            });
            let result = builder.calculate();
            assert!(matches!(result.priming_sugar.corn_sugar.unit, MassUnitType::Oz));
            // ~120g ≈ 4.2 oz
            assert!(result.priming_sugar.corn_sugar.value > 3.0);
            assert!(result.priming_sugar.corn_sugar.value < 6.0);
        }

        #[test]
        fn returns_bar_when_requested() {
            let mut builder = InputBuilder::typical();
            builder.beer_temperature = TemperatureType { value: 2.0, unit: TemperatureUnitType::C };
            builder.units = Some(CarbonationOutputOptions {
                mass_unit: MassUnitType::G,
                pressure_unit: PressureUnitType::Bar,
            });
            let result = builder.calculate();
            assert!(matches!(result.forced_carbonation.pressure.unit, PressureUnitType::Bar));
            // ~12 PSI ≈ 0.83 bar
            assert!(result.forced_carbonation.pressure.value > 0.5);
            assert!(result.forced_carbonation.pressure.value < 1.5);
        }

        #[test]
        fn psi_and_kpa_are_equivalent() {
            let mut psi_builder = InputBuilder::typical();
            psi_builder.beer_temperature = TemperatureType { value: 2.0, unit: TemperatureUnitType::C };
            psi_builder.units = Some(CarbonationOutputOptions {
                mass_unit: MassUnitType::G,
                pressure_unit: PressureUnitType::Psi,
            });
            let psi_result = psi_builder.calculate();

            let mut kpa_builder = InputBuilder::typical();
            kpa_builder.beer_temperature = TemperatureType { value: 2.0, unit: TemperatureUnitType::C };
            kpa_builder.units = Some(CarbonationOutputOptions {
                mass_unit: MassUnitType::G,
                pressure_unit: PressureUnitType::KPa,
            });
            let kpa_result = kpa_builder.calculate();

            assert!(
                (kpa_result.forced_carbonation.pressure.value
                    - psi_result.forced_carbonation.pressure.value * 6.89476)
                    .abs()
                    < 0.5
            );
        }

        #[test]
        fn grams_and_ounces_are_equivalent() {
            let grams_result = InputBuilder::typical().calculate();

            let mut oz_builder = InputBuilder::typical();
            oz_builder.units = Some(CarbonationOutputOptions {
                mass_unit: MassUnitType::Oz,
                pressure_unit: PressureUnitType::KPa,
            });
            let oz_result = oz_builder.calculate();

            assert!(
                (grams_result.priming_sugar.corn_sugar.value
                    - oz_result.priming_sugar.corn_sugar.value * 28.3495)
                    .abs()
                    < 1.0
            );
        }
    }

    mod input_unit_handling {
        use super::*;

        #[test]
        fn accepts_temperature_in_fahrenheit() {
            let in_c = InputBuilder::typical().calculate();

            let mut in_f = InputBuilder::typical();
            in_f.beer_temperature = TemperatureType { value: 68.0, unit: TemperatureUnitType::F };
            let in_f_result = in_f.calculate();

            assert!(
                (in_c.priming_sugar.corn_sugar.value - in_f_result.priming_sugar.corn_sugar.value)
                    .abs()
                    < 1.0
            );
        }

        #[test]
        fn accepts_volume_in_gallons() {
            let in_l = InputBuilder::typical().calculate();

            let mut in_gal = InputBuilder::typical();
            in_gal.beer_volume = VolumeType { value: 5.019, unit: VolumeUnitType::Gal };
            let in_gal_result = in_gal.calculate();

            assert!(
                (in_l.priming_sugar.corn_sugar.value - in_gal_result.priming_sugar.corn_sugar.value)
                    .abs()
                    < 1.0
            );
        }

        #[test]
        fn accepts_carbonation_in_g_per_l() {
            // 2.5 vols = 2.5 * 1.96 = 4.9 g/l
            let in_vols = InputBuilder::typical().calculate();

            let mut in_gl = InputBuilder::typical();
            in_gl.target_carbonation = CarbonationType { value: 4.9, unit: CarbonationUnitType::GL };
            let in_gl_result = in_gl.calculate();

            assert!(
                (in_vols.priming_sugar.corn_sugar.value
                    - in_gl_result.priming_sugar.corn_sugar.value)
                    .abs()
                    < 1.0
            );
        }
    }
}
