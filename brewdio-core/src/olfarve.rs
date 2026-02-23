//! Ol farve: sRGB color rendering of SRM/EBC beer color ratings.
//!
//! Attempt at a spectroscopic model of beer color perception using CIE 1931
//! colour-matching functions, CIE standard illuminant D65, and the Beer–Lambert
//! law.  Output is mapped to the sRGB gamut.
//!
//! Based on the TypeScript/JavaScript implementation by Thomas Ascher,
//! which is in turn based on:
//!
//!   A. J. de Lange, "Color," in *Brewing Materials and Processes*,
//!   Elsevier, 2016, pp. 199–249.
//!
//! sRGB transformations follow:
//!
//!   C. Poynton, *Digital Video and HD: Algorithms and Interfaces*, 2nd ed.
//!   Morgan Kaufmann, 2014.
//!
//! Original code copyright 2022 Thomas Ascher <thomas.ascher@gmx.at>
//! SPDX-License-Identifier: MIT

use std::sync::LazyLock;

// ============================================================================
// CIE DATA TABLE
// ============================================================================

/// Each row: [wavelength_nm, x_bar, y_bar, z_bar, D65_illuminant]
///
/// CIE 1931 colour-matching functions (2 degree observer, 5 nm intervals)
/// https://cie.co.at/datatable/cie-1931-colour-matching-functions-2-degree-observer-5nm
///
/// CIE standard illuminant D65
/// https://cie.co.at/datatable/cie-standard-illuminant-d65
const CIE_DATA: [[f64; 5]; 81] = [
    [380.0, 0.001368, 0.000039, 0.006450, 49.9755],
    [385.0, 0.002236, 0.000064, 0.010550, 52.3118],
    [390.0, 0.004243, 0.000120, 0.020050, 54.6482],
    [395.0, 0.007650, 0.000217, 0.036210, 68.7015],
    [400.0, 0.014310, 0.000396, 0.067850, 82.7549],
    [405.0, 0.023190, 0.000640, 0.110200, 87.1204],
    [410.0, 0.043510, 0.001210, 0.207400, 91.486],
    [415.0, 0.077630, 0.002180, 0.371300, 92.4589],
    [420.0, 0.134380, 0.004000, 0.645600, 93.4318],
    [425.0, 0.214770, 0.007300, 1.039050, 90.057],
    [430.0, 0.283900, 0.011600, 1.385600, 86.6823],
    [435.0, 0.328500, 0.016840, 1.622960, 95.7736],
    [440.0, 0.348280, 0.023000, 1.747060, 104.865],
    [445.0, 0.348060, 0.029800, 1.782600, 110.936],
    [450.0, 0.336200, 0.038000, 1.772110, 117.008],
    [455.0, 0.318700, 0.048000, 1.744100, 117.41],
    [460.0, 0.290800, 0.060000, 1.669200, 117.812],
    [465.0, 0.251100, 0.073900, 1.528100, 116.336],
    [470.0, 0.195360, 0.090980, 1.287640, 114.861],
    [475.0, 0.142100, 0.112600, 1.041900, 115.392],
    [480.0, 0.095640, 0.139020, 0.812950, 115.923],
    [485.0, 0.057950, 0.169300, 0.616200, 112.367],
    [490.0, 0.032010, 0.208020, 0.465180, 108.811],
    [495.0, 0.014700, 0.258600, 0.353300, 109.082],
    [500.0, 0.004900, 0.323000, 0.272000, 109.354],
    [505.0, 0.002400, 0.407300, 0.212300, 108.578],
    [510.0, 0.009300, 0.503000, 0.158200, 107.802],
    [515.0, 0.029100, 0.608200, 0.111700, 106.296],
    [520.0, 0.063270, 0.710000, 0.078250, 104.79],
    [525.0, 0.109600, 0.793200, 0.057250, 106.239],
    [530.0, 0.165500, 0.862000, 0.042160, 107.689],
    [535.0, 0.225750, 0.914850, 0.029840, 106.047],
    [540.0, 0.290400, 0.954000, 0.020300, 104.405],
    [545.0, 0.359700, 0.980300, 0.013400, 104.225],
    [550.0, 0.433450, 0.994950, 0.008750, 104.046],
    [555.0, 0.512050, 1.000000, 0.005750, 102.023],
    [560.0, 0.594500, 0.995000, 0.003900, 100.0],
    [565.0, 0.678400, 0.978600, 0.002750, 98.1671],
    [570.0, 0.762100, 0.952000, 0.002100, 96.3342],
    [575.0, 0.842500, 0.915400, 0.001800, 96.0611],
    [580.0, 0.916300, 0.870000, 0.001650, 95.788],
    [585.0, 0.978600, 0.816300, 0.001400, 92.2368],
    [590.0, 1.026300, 0.757000, 0.001100, 88.6856],
    [595.0, 1.056700, 0.694900, 0.001000, 89.3459],
    [600.0, 1.062200, 0.631000, 0.000800, 90.0062],
    [605.0, 1.045600, 0.566800, 0.000600, 89.8026],
    [610.0, 1.002600, 0.503000, 0.000340, 89.5991],
    [615.0, 0.938400, 0.441200, 0.000240, 88.6489],
    [620.0, 0.854450, 0.381000, 0.000190, 87.69871],
    [625.0, 0.751400, 0.321000, 0.000100, 85.4936],
    [630.0, 0.642400, 0.265000, 0.000050, 83.2886],
    [635.0, 0.541900, 0.217000, 0.000030, 83.4939],
    [640.0, 0.447900, 0.175000, 0.000020, 83.6992],
    [645.0, 0.360800, 0.138200, 0.000010, 81.863],
    [650.0, 0.283500, 0.107000, 0.000000, 80.0268],
    [655.0, 0.218700, 0.081600, 0.000000, 80.1207],
    [660.0, 0.164900, 0.061000, 0.000000, 80.2146],
    [665.0, 0.121200, 0.044580, 0.000000, 81.2462],
    [670.0, 0.087400, 0.032000, 0.000000, 82.2778],
    [675.0, 0.063600, 0.023200, 0.000000, 80.281],
    [680.0, 0.046770, 0.017000, 0.000000, 78.2842],
    [685.0, 0.032900, 0.011920, 0.000000, 74.0027],
    [690.0, 0.022700, 0.008210, 0.000000, 69.7213],
    [695.0, 0.015840, 0.005723, 0.000000, 70.6652],
    [700.0, 0.011359, 0.004102, 0.000000, 71.6091],
    [705.0, 0.008111, 0.002929, 0.000000, 72.979],
    [710.0, 0.005790, 0.002091, 0.000000, 74.349],
    [715.0, 0.004109, 0.001484, 0.000000, 67.9765],
    [720.0, 0.002899, 0.001047, 0.000000, 61.604],
    [725.0, 0.002049, 0.000740, 0.000000, 65.7448],
    [730.0, 0.001440, 0.000520, 0.000000, 69.8856],
    [735.0, 0.001000, 0.000361, 0.000000, 72.4863],
    [740.0, 0.000690, 0.000249, 0.000000, 75.087],
    [745.0, 0.000476, 0.000172, 0.000000, 69.3398],
    [750.0, 0.000332, 0.000120, 0.000000, 63.5927],
    [755.0, 0.000235, 0.000085, 0.000000, 55.0054],
    [760.0, 0.000166, 0.000060, 0.000000, 46.4182],
    [765.0, 0.000117, 0.000042, 0.000000, 56.6118],
    [770.0, 0.000083, 0.000030, 0.000000, 66.8054],
    [775.0, 0.000059, 0.000021, 0.000000, 65.0941],
    [780.0, 0.000042, 0.000015, 0.000000, 63.3828],
];

// ============================================================================
// SCALE FACTOR
// ============================================================================

/// Normalization factor K = 1 / Σ(D65_i × y_bar_i)
fn calc_scale_factor() -> f64 {
    let k: f64 = CIE_DATA.iter().map(|row| row[4] * row[2]).sum();
    1.0 / k
}

static K: LazyLock<f64> = LazyLock::new(calc_scale_factor);

// ============================================================================
// CONSTANTS
// ============================================================================

/// Default transmission path in cm.
/// Set to typical sample glass width as specified by the BJCP color guide.
/// https://www.bjcp.org/education-training/education-resources/color-guide
pub const DEFAULT_PATH_CM: f64 = 5.0;

// ============================================================================
// INTERNAL HELPERS
// ============================================================================

/// Weighted summation against a colour-matching function column.
/// `cmf` selects the column index: 1 = x_bar, 2 = y_bar, 3 = z_bar.
fn summate(transmittance: &[f64], cmf: usize) -> f64 {
    let sum: f64 = CIE_DATA
        .iter()
        .zip(transmittance.iter())
        .map(|(row, t)| row[4] * t * row[cmf])
        .sum();
    *K * sum
}

/// Apply sRGB gamma correction, clamped to [0, 1].
fn correct_gamma(t: f64) -> f64 {
    let v = if t <= 0.0031308 {
        t * 12.92
    } else {
        1.055 * t.powf(1.0 / 2.4) - 0.055
    };
    v.clamp(0.0, 1.0)
}

/// Beer spectral distribution → linear sRGB.
///
/// `a430` is the absorbance at 430 nm (A₄₃₀ = SRM / 12.7 = EBC / 25).
/// `path_cm` is the optical path length in centimetres (glass width).
///
/// Implemented according to:
///   A. J. de Lange, "Color," in *Brewing Materials and Processes*,
///   Elsevier, 2016, pp. 199–249.
fn beer_sd_to_srgb(a430: f64, path_cm: f64) -> [f64; 3] {
    let transmittance: Vec<f64> = CIE_DATA
        .iter()
        .map(|row| {
            let wl = row[0];
            let exponent = -a430
                * path_cm
                * (0.02465 * (-(wl - 430.0) / 17.591).exp()
                    + 0.97535 * (-(wl - 430.0) / 82.122).exp());
            10.0_f64.powf(exponent)
        })
        .collect();

    let x = summate(&transmittance, 1);
    let y = summate(&transmittance, 2);
    let z = summate(&transmittance, 3);

    let r = correct_gamma(x * 3.240479 + y * -1.537150 + z * -0.498535);
    let g = correct_gamma(x * -0.969256 + y * 1.875992 + z * 0.041556);
    let b = correct_gamma(x * 0.055648 + y * -0.204043 + z * 1.057311);

    [r, g, b]
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Determine a colour in sRGB space (relative intensity 0–1) for a given SRM
/// rating and transmission path in cm (glass width).
///
/// Uses the default 5 cm path (BJCP sample glass) when `path_cm` is `None`.
pub fn srm_to_srgb(srm: f64, path_cm: Option<f64>) -> [f64; 3] {
    beer_sd_to_srgb(srm / 12.7, path_cm.unwrap_or(DEFAULT_PATH_CM))
}

/// Determine a colour in sRGB space (relative intensity 0–1) for a given EBC
/// rating and transmission path in cm (glass width).
///
/// Uses the default 5 cm path (BJCP sample glass) when `path_cm` is `None`.
pub fn ebc_to_srgb(ebc: f64, path_cm: Option<f64>) -> [f64; 3] {
    beer_sd_to_srgb(ebc / 25.0, path_cm.unwrap_or(DEFAULT_PATH_CM))
}

/// Convert a relative-intensity RGB triplet ([0–1, 0–1, 0–1]) into a hex
/// string like `"#f4a623"`.
pub fn rgb_to_hex(rgb: &[f64; 3]) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (rgb[0] * 255.0).round() as u8,
        (rgb[1] * 255.0).round() as u8,
        (rgb[2] * 255.0).round() as u8,
    )
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Convenience: compute hex from SRM with default path
    fn srm_hex(srm: f64) -> String {
        rgb_to_hex(&srm_to_srgb(srm, None))
    }

    // Convenience: compute hex from EBC with default path
    fn ebc_hex(ebc: f64) -> String {
        rgb_to_hex(&ebc_to_srgb(ebc, None))
    }

    // ------------------------------------------------------------------
    // Scale factor sanity
    // ------------------------------------------------------------------

    #[test]
    fn scale_factor_is_positive_and_small() {
        let k = *K;
        assert!(k > 0.0);
        assert!(k < 1.0);
    }

    // ------------------------------------------------------------------
    // RGB value ranges
    // ------------------------------------------------------------------

    #[test]
    fn rgb_components_are_clamped_0_to_1() {
        for srm in [0.0, 2.0, 10.0, 30.0, 60.0, 100.0] {
            let rgb = srm_to_srgb(srm, None);
            for &c in &rgb {
                assert!(c >= 0.0 && c <= 1.0, "SRM {srm}: component {c} out of range");
            }
        }
    }

    // ------------------------------------------------------------------
    // Color progression: darker beer → lower RGB values
    // ------------------------------------------------------------------

    #[test]
    fn higher_srm_gives_darker_color() {
        let light = srm_to_srgb(2.0, None);
        let medium = srm_to_srgb(15.0, None);
        let dark = srm_to_srgb(40.0, None);

        // All channels should decrease (or stay the same) as SRM rises
        for i in 0..3 {
            assert!(
                light[i] >= medium[i],
                "channel {i}: light ({}) should be >= medium ({})",
                light[i], medium[i],
            );
            assert!(
                medium[i] >= dark[i],
                "channel {i}: medium ({}) should be >= dark ({})",
                medium[i], dark[i],
            );
        }
    }

    // ------------------------------------------------------------------
    // SRM 0 should be close to "white" / full transmittance
    // ------------------------------------------------------------------

    #[test]
    fn srm_zero_is_near_white() {
        let rgb = srm_to_srgb(0.0, None);
        // Very pale → all channels should be close to 1.0
        for &c in &rgb {
            assert!(c > 0.95, "SRM 0 channel {c} should be near 1.0");
        }
    }

    // ------------------------------------------------------------------
    // Very high SRM → near black
    // ------------------------------------------------------------------

    #[test]
    fn very_high_srm_is_near_black() {
        let rgb = srm_to_srgb(100.0, None);
        for &c in &rgb {
            assert!(c < 0.15, "SRM 100 channel {c} should be near 0.0");
        }
    }

    // ------------------------------------------------------------------
    // SRM / EBC equivalence: SRM = EBC / 1.97
    // ------------------------------------------------------------------

    #[test]
    fn srm_and_ebc_agree() {
        // SRM/12.7 vs EBC/25.0 — the 1.97 conversion factor is approximate,
        // so we allow a small tolerance in the final sRGB values.
        let srm = 10.0;
        let ebc = srm * 1.97;
        let rgb_srm = srm_to_srgb(srm, None);
        let rgb_ebc = ebc_to_srgb(ebc, None);
        for i in 0..3 {
            assert!(
                (rgb_srm[i] - rgb_ebc[i]).abs() < 0.005,
                "channel {i}: SRM ({}) != EBC ({})",
                rgb_srm[i], rgb_ebc[i],
            );
        }
    }

    // ------------------------------------------------------------------
    // Path length affects result
    // ------------------------------------------------------------------

    #[test]
    fn shorter_path_gives_lighter_color() {
        let thin = srm_to_srgb(10.0, Some(1.0)); // 1 cm glass
        let thick = srm_to_srgb(10.0, Some(10.0)); // 10 cm glass
        // Thinner glass → more light → brighter
        for i in 0..3 {
            assert!(
                thin[i] >= thick[i],
                "channel {i}: thin ({}) should be >= thick ({})",
                thin[i], thick[i],
            );
        }
    }

    // ------------------------------------------------------------------
    // Hex conversion
    // ------------------------------------------------------------------

    #[test]
    fn hex_format_is_correct() {
        let hex = rgb_to_hex(&[1.0, 0.0, 0.5]);
        assert_eq!(hex, "#ff0080");
    }

    #[test]
    fn hex_for_black() {
        assert_eq!(rgb_to_hex(&[0.0, 0.0, 0.0]), "#000000");
    }

    #[test]
    fn hex_for_white() {
        assert_eq!(rgb_to_hex(&[1.0, 1.0, 1.0]), "#ffffff");
    }

    // ------------------------------------------------------------------
    // Typical beer styles — spot-check expected colour families
    // ------------------------------------------------------------------

    #[test]
    fn pale_lager_is_straw_gold() {
        // SRM 2–4 → straw/gold hue, high red/green, lower blue
        let rgb = srm_to_srgb(3.0, None);
        assert!(rgb[0] > 0.8, "red channel should be high for pale lager");
        assert!(rgb[1] > 0.6, "green channel should be moderate-high for pale lager");
        // blue will be lower than green
        assert!(rgb[2] < rgb[1], "blue should be less than green for gold color");
    }

    #[test]
    fn amber_ale_is_amber() {
        // SRM 10–15 → amber/copper
        let rgb = srm_to_srgb(12.0, None);
        assert!(rgb[0] > 0.5, "red channel should be prominent");
        assert!(rgb[1] < rgb[0], "green < red for amber");
        assert!(rgb[2] < rgb[1], "blue < green for amber");
    }

    #[test]
    fn stout_is_very_dark() {
        // SRM 35+ → very dark brown/black
        let rgb = srm_to_srgb(40.0, None);
        assert!(rgb[0] < 0.3, "red should be low for stout");
        assert!(rgb[1] < 0.15, "green should be very low for stout");
        assert!(rgb[2] < 0.1, "blue should be near zero for stout");
    }

    // ------------------------------------------------------------------
    // EBC hex output
    // ------------------------------------------------------------------

    #[test]
    fn ebc_hex_produces_valid_string() {
        let hex = ebc_hex(20.0);
        assert!(hex.starts_with('#'));
        assert_eq!(hex.len(), 7);
        // All chars after # should be valid hex digits
        assert!(hex[1..].chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ------------------------------------------------------------------
    // SRM hex output
    // ------------------------------------------------------------------

    #[test]
    fn srm_hex_produces_valid_string() {
        let hex = srm_hex(5.0);
        assert!(hex.starts_with('#'));
        assert_eq!(hex.len(), 7);
        assert!(hex[1..].chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ------------------------------------------------------------------
    // Gamma correction edge cases
    // ------------------------------------------------------------------

    #[test]
    fn gamma_correction_clamps_to_zero() {
        assert_eq!(correct_gamma(-1.0), 0.0);
    }

    #[test]
    fn gamma_correction_clamps_to_one() {
        assert_eq!(correct_gamma(100.0), 1.0);
    }

    #[test]
    fn gamma_correction_linear_region() {
        // Below 0.0031308 the transform is linear: t * 12.92
        let t = 0.001;
        assert!((correct_gamma(t) - t * 12.92).abs() < 1e-10);
    }

    #[test]
    fn gamma_correction_power_region() {
        // Above 0.0031308 the transform is: 1.055 * t^(1/2.4) - 0.055
        let t: f64 = 0.5;
        let expected = 1.055 * t.powf(1.0 / 2.4) - 0.055;
        assert!((correct_gamma(t) - expected).abs() < 1e-10);
    }

    // ------------------------------------------------------------------
    // Consistency: same SRM → same output
    // ------------------------------------------------------------------

    #[test]
    fn deterministic_output() {
        let a = srm_to_srgb(8.0, None);
        let b = srm_to_srgb(8.0, None);
        assert_eq!(a, b);
    }
}
