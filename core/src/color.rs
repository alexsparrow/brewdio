//! Beer Color Calculation (Morey SRM)
//! Calculates the beer color based on fermentables and batch size.

use crate::beerjson_types::*;
use crate::units::{color_to_srm, mass_to_pounds, srm_to_lovibond, volume_to_gallons};

/// Calculate MCU (Malt Color Units) for a fermentable
fn mcu(fermentable: &FermentableAdditionType, batch_size: &VolumeType) -> f64 {
    let srm = color_to_srm(&fermentable.color);
    let lovibond = srm_to_lovibond(srm);
    let mass_in_pounds = match &fermentable.amount {
        FermentableAdditionTypeAmount::MassType(mass) => mass_to_pounds(mass),
        FermentableAdditionTypeAmount::VolumeType(_) => return 0.0,
    };
    (mass_in_pounds * lovibond) / volume_to_gallons(batch_size)
}

/// Convert MCU to SRM using Morey equation
fn mcu_to_srm(mcu: f64) -> f64 {
    1.4922 * mcu.powf(0.6859)
}

/// Calculate the beer color (SRM) based on fermentables and batch size.
///
/// Uses the Morey equation: SRM = 1.4922 × MCU^0.6859
/// where MCU = Σ(lbs × lovibond) / batch_gal
pub fn calculate_color(
    fermentables: &[FermentableAdditionType],
    batch_size: &VolumeType,
) -> f64 {
    let total_mcu: f64 = fermentables.iter().map(|f| mcu(f, batch_size)).sum();
    mcu_to_srm(total_mcu)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fermentable(
        amount_value: f64,
        amount_unit: MassUnitType,
        color_value: f64,
        color_unit: ColorUnitType,
    ) -> FermentableAdditionType {
        FermentableAdditionType {
            name: "Test Grain".to_string(),
            type_: FermentableAdditionTypeType::Grain,
            amount: FermentableAdditionTypeAmount::MassType(MassType {
                value: amount_value,
                unit: amount_unit,
            }),
            yield_: YieldType {
                fine_grind: Some(PercentType { value: 80.0, unit: PercentUnitType::X }),
                coarse_grind: None,
                fine_coarse_difference: None,
                potential: None,
            },
            color: ColorType {
                value: color_value,
                unit: color_unit,
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

    mod single_fermentable {
        use super::*;

        #[test]
        fn calculates_srm_for_a_typical_pale_malt() {
            // 10 lb of 2L grain in 5 gal
            let result = calculate_color(
                &[fermentable(10.0, MassUnitType::Lb, 2.0, ColorUnitType::Srm)],
                &batch_size(5.0, VolumeUnitType::Gal),
            );
            assert!(result > 3.0);
            assert!(result < 6.0);
        }

        #[test]
        fn calculates_srm_for_a_dark_specialty_malt() {
            // 1 lb of 500L roasted barley in 5 gal
            let result = calculate_color(
                &[fermentable(1.0, MassUnitType::Lb, 500.0, ColorUnitType::Lovi)],
                &batch_size(5.0, VolumeUnitType::Gal),
            );
            assert!(result > 30.0);
        }

        #[test]
        fn calculates_srm_for_crystal_malt_in_lovibond() {
            // 1 lb of 60L Crystal in 5 gal
            let result = calculate_color(
                &[fermentable(1.0, MassUnitType::Lb, 60.0, ColorUnitType::Lovi)],
                &batch_size(5.0, VolumeUnitType::Gal),
            );
            assert!(result > 5.0);
            assert!(result < 20.0);
        }
    }

    mod multiple_fermentables {
        use super::*;

        #[test]
        fn combines_mcu_from_multiple_grains_before_applying_morey() {
            let pale = fermentable(9.0, MassUnitType::Lb, 2.0, ColorUnitType::Srm);
            let crystal = fermentable(1.0, MassUnitType::Lb, 60.0, ColorUnitType::Lovi);
            let bs = batch_size(5.0, VolumeUnitType::Gal);

            let combined = calculate_color(&[pale.clone(), crystal], &bs);
            let pale_only = calculate_color(&[pale], &bs);
            assert!(combined > pale_only);
        }

        #[test]
        fn morey_equation_is_non_linear() {
            let grain1 = fermentable(5.0, MassUnitType::Lb, 3.0, ColorUnitType::Srm);
            let grain2 = fermentable(2.0, MassUnitType::Lb, 40.0, ColorUnitType::Lovi);
            let bs = batch_size(5.0, VolumeUnitType::Gal);

            let combined = calculate_color(&[grain1.clone(), grain2.clone()], &bs);
            let srm1 = calculate_color(&[grain1], &bs);
            let srm2 = calculate_color(&[grain2], &bs);

            // Due to Morey's power function, combined SRM != srm1 + srm2
            assert!((combined - (srm1 + srm2)).abs() > 0.5);
        }
    }

    mod batch_size_impact {
        use super::*;

        #[test]
        fn larger_batch_size_produces_lighter_color() {
            let grains = vec![fermentable(10.0, MassUnitType::Lb, 3.0, ColorUnitType::Srm)];

            let small = calculate_color(&grains, &batch_size(5.0, VolumeUnitType::Gal));
            let large = calculate_color(&grains, &batch_size(10.0, VolumeUnitType::Gal));

            assert!(small > large);
        }

        #[test]
        fn accepts_volume_in_liters() {
            let grains = vec![fermentable(10.0, MassUnitType::Lb, 3.0, ColorUnitType::Srm)];

            let in_gallons = calculate_color(&grains, &batch_size(5.0, VolumeUnitType::Gal));
            let in_liters = calculate_color(&grains, &batch_size(18.9271, VolumeUnitType::L));

            assert!((in_gallons - in_liters).abs() < 1.0);
        }
    }

    mod amount_units {
        use super::*;

        #[test]
        fn accepts_weight_in_kilograms() {
            let bs = batch_size(5.0, VolumeUnitType::Gal);
            let in_lb = calculate_color(
                &[fermentable(10.0, MassUnitType::Lb, 3.0, ColorUnitType::Srm)],
                &bs,
            );
            let in_kg = calculate_color(
                &[fermentable(4.53592, MassUnitType::Kg, 3.0, ColorUnitType::Srm)],
                &bs,
            );
            assert!((in_kg - in_lb).abs() < 0.5);
        }
    }

    mod color_unit_conversions {
        use super::*;

        #[test]
        fn accepts_ebc_color_values() {
            let bs = batch_size(5.0, VolumeUnitType::Gal);
            // 3.94 EBC ≈ 2 SRM
            let from_srm = calculate_color(
                &[fermentable(10.0, MassUnitType::Lb, 2.0, ColorUnitType::Srm)],
                &bs,
            );
            let from_ebc = calculate_color(
                &[fermentable(10.0, MassUnitType::Lb, 3.94, ColorUnitType::Ebc)],
                &bs,
            );
            assert!((from_ebc - from_srm).abs() < 0.5);
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn returns_0_for_empty_fermentable_list() {
            // Morey: 1.4922 * 0^0.6859 = 0
            let result = calculate_color(&[], &batch_size(5.0, VolumeUnitType::Gal));
            assert_eq!(result, 0.0);
        }
    }
}
