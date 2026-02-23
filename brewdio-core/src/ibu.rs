//! IBU (International Bitterness Units) Calculation using Tinseth method
//! See: https://homebrewacademy.com/ibu-calculator/

use crate::beerjson_types::*;
use crate::units::{mass_to_ounces, time_to_minutes, volume_to_gallons};

/// Calculate milligrams per liter of added alpha acids
fn mgpl_added_alpha_acids(hop: &HopAdditionType, batch_size: &VolumeType) -> f64 {
    let alpha_acid = hop.alpha_acid.value;
    let amount_oz = match &hop.amount {
        HopAdditionTypeAmount::MassType(mass) => mass_to_ounces(mass),
        HopAdditionTypeAmount::VolumeType(_) => return 0.0,
    };
    ((alpha_acid / 100.0) * amount_oz * 7490.0) / volume_to_gallons(batch_size)
}

/// Calculate boil time factor for hop utilization
fn boil_time_factor(hop: &HopAdditionType) -> f64 {
    match &hop.timing.time {
        Some(time) => (1.0 - (-0.04 * time_to_minutes(time)).exp()) / 4.15,
        None => 0.0,
    }
}

/// Calculate IBU using the Tinseth method.
///
/// Filters hops to only include boil additions (excludes dry hop, fermentation, packaging).
/// For each boil hop:
///   IBU = bigness_factor × boil_time_factor × mgpl
///
/// where bigness_factor = 1.65 × 0.000125^(OG - 1)
pub fn calculate_ibu(
    hops: &[HopAdditionType],
    batch_size: &VolumeType,
    og: f64,
) -> f64 {
    // Filter to only include boil hop additions
    // Dry hops and other fermentation/packaging additions don't contribute to IBU
    let boil_hops: Vec<&HopAdditionType> = hops
        .iter()
        .filter(|hop| match &hop.timing.use_ {
            None => true, // default to boil
            Some(UseType::AddToBoil) => true,
            _ => false,
        })
        .collect();

    if boil_hops.is_empty() {
        return 0.0;
    }

    // Bigness factor based on original gravity
    let bigness_factor = 1.65 * 0.000125_f64.powf(og - 1.0);

    // Calculate IBU for each boil hop addition
    boil_hops
        .iter()
        .map(|hop| {
            let btf = boil_time_factor(hop);
            let mgpl = mgpl_added_alpha_acids(hop, batch_size);
            bigness_factor * btf * mgpl
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hop(
        alpha_acid: f64,
        amount_value: f64,
        amount_unit: MassUnitType,
        time_value: i64,
        time_unit: TimeUnitType,
        use_: Option<UseType>,
    ) -> HopAdditionType {
        HopAdditionType {
            name: "Test Hop".to_string(),
            alpha_acid: PercentType {
                value: alpha_acid,
                unit: PercentUnitType::X,
            },
            amount: HopAdditionTypeAmount::MassType(MassType {
                value: amount_value,
                unit: amount_unit,
            }),
            timing: TimingType {
                time: Some(TimeType {
                    value: time_value,
                    unit: time_unit,
                }),
                use_: use_,
                continuous: None,
                duration: None,
                p_h: None,
                specific_gravity: None,
                step: None,
            },
            beta_acid: None,
            form: None,
            origin: None,
            producer: None,
            product_id: None,
            year: None,
        }
    }

    fn boil_hop(alpha_acid: f64, oz: f64, minutes: i64) -> HopAdditionType {
        hop(alpha_acid, oz, MassUnitType::Oz, minutes, TimeUnitType::Min, Some(UseType::AddToBoil))
    }

    fn batch_size(value: f64, unit: VolumeUnitType) -> VolumeType {
        VolumeType { value, unit }
    }

    mod basic_single_hop_additions {
        use super::*;

        #[test]
        fn calculates_ibu_for_a_typical_60_minute_boil_addition() {
            // 1 oz of 10% AA hops, 60 min boil, 5 gal, OG 1.050
            let result = calculate_ibu(
                &[boil_hop(10.0, 1.0, 60)],
                &batch_size(5.0, VolumeUnitType::Gal),
                1.05,
            );
            // Expected ~34.6 IBU
            assert!((result - 34.6).abs() < 1.0);
        }

        #[test]
        fn calculates_ibu_for_a_15_minute_addition() {
            let result = calculate_ibu(
                &[boil_hop(5.0, 1.0, 15)],
                &batch_size(5.0, VolumeUnitType::Gal),
                1.05,
            );
            assert!(result > 0.0);
            assert!(result < 20.0);
        }

        #[test]
        fn calculates_ibu_for_a_high_alpha_hop() {
            let bs = batch_size(5.0, VolumeUnitType::Gal);
            let result = calculate_ibu(&[boil_hop(18.0, 1.0, 60)], &bs, 1.05);
            let base_result = calculate_ibu(&[boil_hop(10.0, 1.0, 60)], &bs, 1.05);
            // 18% AA should give roughly 1.8x the IBU of 10% AA
            assert!((result - base_result * 1.8).abs() < 1.0);
        }
    }

    mod multiple_hop_additions {
        use super::*;

        #[test]
        fn sums_ibu_from_multiple_hop_additions() {
            let bittering = boil_hop(12.0, 1.0, 60);
            let flavor = boil_hop(5.0, 0.5, 15);
            let bs = batch_size(5.0, VolumeUnitType::Gal);

            let combined = calculate_ibu(&[bittering.clone(), flavor.clone()], &bs, 1.05);
            let bittering_only = calculate_ibu(&[bittering], &bs, 1.05);
            let flavor_only = calculate_ibu(&[flavor], &bs, 1.05);

            assert!((combined - (bittering_only + flavor_only)).abs() < 1e-5);
        }
    }

    mod gravity_impact {
        use super::*;

        #[test]
        fn higher_og_reduces_ibu() {
            let hops = vec![boil_hop(10.0, 1.0, 60)];
            let bs = batch_size(5.0, VolumeUnitType::Gal);

            let low_gravity = calculate_ibu(&hops, &bs, 1.040);
            let high_gravity = calculate_ibu(&hops, &bs, 1.080);

            assert!(low_gravity > high_gravity);
        }

        #[test]
        fn og_1000_gives_maximum_bigness_factor() {
            let hops = vec![boil_hop(10.0, 1.0, 60)];
            let bs = batch_size(5.0, VolumeUnitType::Gal);

            let result = calculate_ibu(&hops, &bs, 1.0);
            assert!(result > calculate_ibu(&hops, &bs, 1.05));
        }
    }

    mod boil_time_impact {
        use super::*;

        #[test]
        fn longer_boil_time_increases_ibu() {
            let bs = batch_size(5.0, VolumeUnitType::Gal);
            let short = calculate_ibu(&[boil_hop(10.0, 1.0, 15)], &bs, 1.05);
            let long = calculate_ibu(&[boil_hop(10.0, 1.0, 60)], &bs, 1.05);
            assert!(long > short);
        }

        #[test]
        fn zero_minute_boil_gives_zero_ibu() {
            let result = calculate_ibu(
                &[boil_hop(10.0, 1.0, 0)],
                &batch_size(5.0, VolumeUnitType::Gal),
                1.05,
            );
            assert!(result.abs() < 1e-5);
        }

        #[test]
        fn accepts_time_in_hours() {
            let bs = batch_size(5.0, VolumeUnitType::Gal);
            let from_minutes = calculate_ibu(&[boil_hop(10.0, 1.0, 60)], &bs, 1.05);
            let from_hours = calculate_ibu(
                &[hop(10.0, 1.0, MassUnitType::Oz, 1, TimeUnitType::Hr, Some(UseType::AddToBoil))],
                &bs,
                1.05,
            );
            assert!((from_hours - from_minutes).abs() < 1e-5);
        }
    }

    mod batch_size_impact {
        use super::*;

        #[test]
        fn larger_batch_size_reduces_ibu() {
            let hops = vec![boil_hop(10.0, 1.0, 60)];

            let small = calculate_ibu(&hops, &batch_size(5.0, VolumeUnitType::Gal), 1.05);
            let large = calculate_ibu(&hops, &batch_size(10.0, VolumeUnitType::Gal), 1.05);

            assert!(small > large);
            // Doubling batch size should halve IBU (linear relationship)
            assert!((small - large * 2.0).abs() < 0.5);
        }

        #[test]
        fn accepts_volume_in_liters() {
            let hops = vec![boil_hop(10.0, 1.0, 60)];

            let in_gallons = calculate_ibu(&hops, &batch_size(5.0, VolumeUnitType::Gal), 1.05);
            let in_liters = calculate_ibu(&hops, &batch_size(18.9271, VolumeUnitType::L), 1.05);

            assert!((in_gallons - in_liters).abs() < 1.0);
        }
    }

    mod hop_amount_units {
        use super::*;

        #[test]
        fn accepts_grams() {
            let bs = batch_size(5.0, VolumeUnitType::Gal);
            let in_oz = calculate_ibu(&[boil_hop(10.0, 1.0, 60)], &bs, 1.05);
            let in_grams = calculate_ibu(
                &[hop(10.0, 28.3495, MassUnitType::G, 60, TimeUnitType::Min, Some(UseType::AddToBoil))],
                &bs,
                1.05,
            );
            assert!((in_grams - in_oz).abs() < 0.5);
        }
    }

    mod non_boil_additions_excluded {
        use super::*;

        #[test]
        fn dry_hop_additions_contribute_0_ibu() {
            let result = calculate_ibu(
                &[hop(10.0, 2.0, MassUnitType::Oz, 72, TimeUnitType::Hr, Some(UseType::AddToFermentation))],
                &batch_size(5.0, VolumeUnitType::Gal),
                1.05,
            );
            assert_eq!(result, 0.0);
        }

        #[test]
        fn packaging_additions_contribute_0_ibu() {
            let result = calculate_ibu(
                &[hop(10.0, 1.0, MassUnitType::Oz, 0, TimeUnitType::Min, Some(UseType::AddToPackage))],
                &batch_size(5.0, VolumeUnitType::Gal),
                1.05,
            );
            assert_eq!(result, 0.0);
        }

        #[test]
        fn only_boil_hops_are_counted_in_a_mixed_addition() {
            let boil = boil_hop(10.0, 1.0, 60);
            let dry = hop(10.0, 2.0, MassUnitType::Oz, 72, TimeUnitType::Hr, Some(UseType::AddToFermentation));
            let bs = batch_size(5.0, VolumeUnitType::Gal);

            let mixed = calculate_ibu(&[boil.clone(), dry], &bs, 1.05);
            let boil_only = calculate_ibu(&[boil], &bs, 1.05);

            assert!((mixed - boil_only).abs() < 1e-5);
        }

        #[test]
        fn hops_with_no_use_default_to_boil() {
            let result = calculate_ibu(
                &[hop(10.0, 1.0, MassUnitType::Oz, 60, TimeUnitType::Min, None)],
                &batch_size(5.0, VolumeUnitType::Gal),
                1.05,
            );
            assert!(result > 0.0);
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn returns_0_for_empty_hop_list() {
            assert_eq!(calculate_ibu(&[], &batch_size(5.0, VolumeUnitType::Gal), 1.05), 0.0);
        }

        #[test]
        fn hop_with_no_timing_time_returns_0_contribution() {
            let hop_no_timing = HopAdditionType {
                name: "Test".to_string(),
                alpha_acid: PercentType { value: 10.0, unit: PercentUnitType::X },
                amount: HopAdditionTypeAmount::MassType(MassType { value: 1.0, unit: MassUnitType::Oz }),
                timing: TimingType {
                    time: None,
                    use_: None,
                    continuous: None,
                    duration: None,
                    p_h: None,
                    specific_gravity: None,
                    step: None,
                },
                beta_acid: None,
                form: None,
                origin: None,
                producer: None,
                product_id: None,
                year: None,
            };

            let result = calculate_ibu(&[hop_no_timing], &batch_size(5.0, VolumeUnitType::Gal), 1.05);
            assert_eq!(result, 0.0);
        }

        #[test]
        fn hop_with_0_percent_alpha_acid_contributes_0_ibu() {
            let result = calculate_ibu(
                &[boil_hop(0.0, 1.0, 60)],
                &batch_size(5.0, VolumeUnitType::Gal),
                1.05,
            );
            assert_eq!(result, 0.0);
        }
    }
}
