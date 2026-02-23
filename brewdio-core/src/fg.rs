//! Final Gravity (FG) Calculation
//! Calculates the final gravity based on OG and yeast attenuation.

use crate::beerjson_types::*;

/// Calculate the final gravity based on OG and yeast attenuation.
///
/// FG = OG - (OG - 1) × attenuation
///
/// Uses the first culture's attenuation, defaulting to 75% if none is provided.
pub fn calculate_fg(og: f64, cultures: &[CultureAdditionType]) -> f64 {
    // Get attenuation from first yeast, or use default (75%)
    let attenuation_value = cultures
        .first()
        .and_then(|c| c.attenuation.as_ref())
        .map(|a| a.value / 100.0)
        .unwrap_or(0.75);

    // FG = OG - (OG - 1) × attenuation
    og - (og - 1.0) * attenuation_value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn culture(attenuation: f64) -> CultureAdditionType {
        CultureAdditionType {
            name: "Test Yeast".to_string(),
            type_: CultureAdditionTypeType::Ale,
            form: CultureAdditionTypeForm::Liquid,
            amount: CultureAdditionTypeAmount::VolumeType(VolumeType {
                value: 1.0,
                unit: VolumeUnitType::L,
            }),
            attenuation: Some(PercentType {
                value: attenuation,
                unit: PercentUnitType::X,
            }),
            cell_count_billions: None,
            producer: None,
            product_id: None,
            times_cultured: None,
            timing: None,
        }
    }

    #[test]
    fn calculates_fg_from_og_and_attenuation() {
        // OG 1.050, 75% attenuation
        // FG = 1.050 - (1.050 - 1.0) * 0.75 = 1.050 - 0.0375 = 1.0125
        let result = calculate_fg(1.050, &[culture(75.0)]);
        assert!((result - 1.0125).abs() < 0.001);
    }

    #[test]
    fn defaults_to_75_percent_when_no_cultures() {
        let result = calculate_fg(1.050, &[]);
        let with_75 = calculate_fg(1.050, &[culture(75.0)]);
        assert!((result - with_75).abs() < 1e-10);
    }

    #[test]
    fn higher_attenuation_gives_lower_fg() {
        let low_atten = calculate_fg(1.060, &[culture(65.0)]);
        let high_atten = calculate_fg(1.060, &[culture(85.0)]);
        assert!(low_atten > high_atten);
    }

    #[test]
    fn uses_first_culture_attenuation() {
        let cultures = vec![culture(80.0), culture(70.0)];
        let result = calculate_fg(1.050, &cultures);
        let expected = calculate_fg(1.050, &[culture(80.0)]);
        assert!((result - expected).abs() < 1e-10);
    }
}
