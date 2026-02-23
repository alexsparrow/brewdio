//! Original Gravity (OG) Calculation
//! Calculates the original gravity based on fermentables, batch size, and efficiency.

use crate::beerjson_types::*;
use crate::units::{mass_to_pounds, volume_to_gallons};

/// Convert yield to Points Per Gallon (PPG)
fn yield_to_ppg(yield_: &YieldType) -> f64 {
    if let Some(ref fine_grind) = yield_.fine_grind {
        (fine_grind.value / 100.0) * 46.21
    } else {
        0.0
    }
}

/// Calculate gravity units for a fermentable
fn gravity_unit(fermentable: &FermentableAdditionType) -> f64 {
    let yield_ppg = yield_to_ppg(&fermentable.yield_);

    let amount_lb = match &fermentable.amount {
        FermentableAdditionTypeAmount::MassType(mass) => mass_to_pounds(mass),
        FermentableAdditionTypeAmount::VolumeType(_) => return 0.0,
    };

    yield_ppg * amount_lb
}

/// Calculate the original gravity based on fermentables, batch size, and efficiency.
///
/// Formula:
/// - PPG = (fine_grind_yield / 100) × 46.21
/// - gravity_units = Σ(PPG × amount_lb)
/// - OG = 1 + (gravity_units × efficiency) / post_boil_volume / 1000
pub fn calculate_og(
    fermentables: &[FermentableAdditionType],
    batch_size: &VolumeType,
    efficiency: &PercentType,
) -> f64 {
    let efficiency_value = efficiency.value;

    // Calculate total gravity units
    let total_gravity: f64 = fermentables.iter().map(|f| gravity_unit(f)).sum();

    // Apply efficiency
    let total_gravity_with_efficiency = (total_gravity * efficiency_value) / 100.0;

    // Adjust for post-boil volume
    let post_boil_volume_in_gallons = volume_to_gallons(batch_size) + 0.53;

    // Calculate final OG
    let gravity_units = total_gravity_with_efficiency / post_boil_volume_in_gallons;
    1.0 + gravity_units / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a fermentable with yield data
    fn fermentable(amount_value: f64, amount_unit: MassUnitType, fine_grind: f64) -> FermentableAdditionType {
        FermentableAdditionType {
            name: "Test Grain".to_string(),
            type_: FermentableAdditionTypeType::Grain,
            amount: FermentableAdditionTypeAmount::MassType(MassType {
                value: amount_value,
                unit: amount_unit,
            }),
            yield_: YieldType {
                fine_grind: Some(PercentType {
                    value: fine_grind,
                    unit: PercentUnitType::X,
                }),
                coarse_grind: None,
                fine_coarse_difference: None,
                potential: None,
            },
            color: ColorType {
                value: 3.0,
                unit: ColorUnitType::Srm,
            },
            timing: None,
            grain_group: None,
            origin: None,
            producer: None,
            product_id: None,
        }
    }

    fn batch_size(value: f64, unit: VolumeUnitType) -> VolumeType {
        VolumeType { value, unit }
    }

    fn efficiency(value: f64) -> PercentType {
        PercentType {
            value,
            unit: PercentUnitType::X,
        }
    }

    mod basic_calculations {
        use super::*;

        #[test]
        fn calculates_og_for_a_typical_pale_ale_grain_bill() {
            // 10 lb of 2-row (80% fine grind yield), 5 gal, 72% efficiency
            // PPG = (80/100) * 46.21 = 36.968
            // Gravity units = 36.968 * 10 = 369.68
            // With efficiency: 369.68 * 72/100 = 266.17
            // Post-boil volume: 5 + 0.53 = 5.53 gal
            // GU / vol = 266.17 / 5.53 = 48.13
            // OG = 1 + 48.13/1000 = 1.048
            let result = calculate_og(
                &[fermentable(10.0, MassUnitType::Lb, 80.0)],
                &batch_size(5.0, VolumeUnitType::Gal),
                &efficiency(72.0),
            );
            assert!((result - 1.048).abs() < 0.005);
        }

        #[test]
        fn calculates_og_for_a_high_gravity_beer() {
            // 15 lb of grain, 5 gal, 75% efficiency
            let result = calculate_og(
                &[fermentable(15.0, MassUnitType::Lb, 80.0)],
                &batch_size(5.0, VolumeUnitType::Gal),
                &efficiency(75.0),
            );
            assert!(result > 1.065);
            assert!(result < 1.085);
        }

        #[test]
        fn calculates_og_for_a_session_beer() {
            // 6 lb of grain, 5 gal, 70% efficiency
            let result = calculate_og(
                &[fermentable(6.0, MassUnitType::Lb, 78.0)],
                &batch_size(5.0, VolumeUnitType::Gal),
                &efficiency(70.0),
            );
            assert!(result > 1.020);
            assert!(result < 1.040);
        }
    }

    mod multiple_fermentables {
        use super::*;

        #[test]
        fn sums_gravity_contributions_from_multiple_grains() {
            let base_malt = fermentable(8.0, MassUnitType::Lb, 80.0);
            let crystal_malt = fermentable(1.0, MassUnitType::Lb, 73.0);

            let bs = batch_size(5.0, VolumeUnitType::Gal);
            let eff = efficiency(72.0);

            let combined = calculate_og(&[base_malt.clone(), crystal_malt], &bs, &eff);
            let base_only = calculate_og(&[base_malt], &bs, &eff);

            assert!(combined > base_only);
        }

        #[test]
        fn gravity_contributions_are_additive_before_volume_division() {
            let grain1 = fermentable(5.0, MassUnitType::Lb, 80.0);
            let grain2 = fermentable(5.0, MassUnitType::Lb, 80.0);
            let grain_double = fermentable(10.0, MassUnitType::Lb, 80.0);

            let bs = batch_size(5.0, VolumeUnitType::Gal);
            let eff = efficiency(72.0);

            let two_grains = calculate_og(&[grain1, grain2], &bs, &eff);
            let one_double = calculate_og(&[grain_double], &bs, &eff);

            assert!((two_grains - one_double).abs() < 1e-5);
        }
    }

    mod efficiency_impact {
        use super::*;

        #[test]
        fn higher_efficiency_increases_og() {
            let grains = vec![fermentable(10.0, MassUnitType::Lb, 80.0)];
            let bs = batch_size(5.0, VolumeUnitType::Gal);

            let low = calculate_og(&grains, &bs, &efficiency(60.0));
            let high = calculate_og(&grains, &bs, &efficiency(80.0));

            assert!(high > low);
        }

        #[test]
        fn full_efficiency_gives_maximum_og() {
            let grains = vec![fermentable(10.0, MassUnitType::Lb, 80.0)];
            let bs = batch_size(5.0, VolumeUnitType::Gal);

            let full = calculate_og(&grains, &bs, &efficiency(100.0));
            let partial = calculate_og(&grains, &bs, &efficiency(72.0));

            assert!(full > partial);
        }
    }

    mod batch_size_impact {
        use super::*;

        #[test]
        fn larger_batch_size_reduces_og() {
            let grains = vec![fermentable(10.0, MassUnitType::Lb, 80.0)];
            let eff = efficiency(72.0);

            let small = calculate_og(&grains, &batch_size(5.0, VolumeUnitType::Gal), &eff);
            let large = calculate_og(&grains, &batch_size(10.0, VolumeUnitType::Gal), &eff);

            assert!(small > large);
        }

        #[test]
        fn accepts_volume_in_liters() {
            let grains = vec![fermentable(10.0, MassUnitType::Lb, 80.0)];
            let eff = efficiency(72.0);

            let in_gallons = calculate_og(&grains, &batch_size(5.0, VolumeUnitType::Gal), &eff);
            let in_liters = calculate_og(&grains, &batch_size(18.9271, VolumeUnitType::L), &eff);

            assert!((in_gallons - in_liters).abs() < 0.005);
        }
    }

    mod amount_units {
        use super::*;

        #[test]
        fn accepts_weight_in_kilograms() {
            let eff = efficiency(72.0);
            let bs = batch_size(5.0, VolumeUnitType::Gal);

            let in_lb = calculate_og(&[fermentable(10.0, MassUnitType::Lb, 80.0)], &bs, &eff);
            let in_kg = calculate_og(&[fermentable(4.53592, MassUnitType::Kg, 80.0)], &bs, &eff);

            assert!((in_kg - in_lb).abs() < 0.001);
        }

        #[test]
        fn accepts_weight_in_grams() {
            let eff = efficiency(72.0);
            let bs = batch_size(5.0, VolumeUnitType::Gal);

            let in_lb = calculate_og(&[fermentable(1.0, MassUnitType::Lb, 80.0)], &bs, &eff);
            let in_g = calculate_og(&[fermentable(453.592, MassUnitType::G, 80.0)], &bs, &eff);

            assert!((in_g - in_lb).abs() < 0.001);
        }
    }

    mod post_boil_volume_adjustment {
        use super::*;

        #[test]
        fn includes_053_gal_post_boil_volume_offset() {
            // The formula adds 0.53 gal to batch size
            let grains = vec![fermentable(10.0, MassUnitType::Lb, 80.0)];
            let result = calculate_og(&grains, &batch_size(5.0, VolumeUnitType::Gal), &efficiency(72.0));

            // With offset: 266.17 / 5.53 / 1000 + 1 = 1.04813
            assert!((result - 1.048).abs() < 0.005);
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn returns_1_for_empty_fermentable_list() {
            let result = calculate_og(
                &[],
                &batch_size(5.0, VolumeUnitType::Gal),
                &efficiency(72.0),
            );
            assert!((result - 1.0).abs() < 1e-10);
        }

        #[test]
        fn fermentable_with_no_fine_grind_contributes_zero() {
            let no_yield = FermentableAdditionType {
                name: "Mystery Grain".to_string(),
                type_: FermentableAdditionTypeType::Grain,
                amount: FermentableAdditionTypeAmount::MassType(MassType {
                    value: 10.0,
                    unit: MassUnitType::Lb,
                }),
                yield_: YieldType {
                    fine_grind: None,
                    coarse_grind: None,
                    fine_coarse_difference: None,
                    potential: None,
                },
                color: ColorType {
                    value: 3.0,
                    unit: ColorUnitType::Srm,
                },
                timing: None,
                grain_group: None,
                origin: None,
                producer: None,
                product_id: None,
            };

            let result = calculate_og(
                &[no_yield],
                &batch_size(5.0, VolumeUnitType::Gal),
                &efficiency(72.0),
            );
            assert!((result - 1.0).abs() < 1e-10);
        }
    }
}
