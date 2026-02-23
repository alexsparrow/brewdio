//! Alcohol By Volume (ABV) Calculation
//! Calculates ABV based on OG and FG.

/// Calculate ABV based on OG and FG.
///
/// ABV = (OG - FG) × 131.25
pub fn calculate_abv(og: f64, fg: f64) -> f64 {
    (og - fg) * 131.25
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_abv_for_typical_ale() {
        // OG 1.050, FG 1.010 → ABV = 0.040 * 131.25 = 5.25%
        let result = calculate_abv(1.050, 1.010);
        assert!((result - 5.25).abs() < 0.01);
    }

    #[test]
    fn zero_difference_gives_zero_abv() {
        let result = calculate_abv(1.050, 1.050);
        assert!((result).abs() < 1e-10);
    }

    #[test]
    fn higher_og_with_same_fg_gives_higher_abv() {
        let low = calculate_abv(1.040, 1.010);
        let high = calculate_abv(1.080, 1.010);
        assert!(high > low);
    }

    #[test]
    fn lower_fg_gives_higher_abv() {
        let low_fg = calculate_abv(1.060, 1.008);
        let high_fg = calculate_abv(1.060, 1.015);
        assert!(low_fg > high_fg);
    }
}
