//! Unit conversion utilities for BeerJSON types.
//! All conversions support the complete set of units defined in the BeerJSON format.

use crate::beerjson_types::*;

// ============================================================================
// VOLUME CONVERSIONS
// ============================================================================

/// Convert any VolumeType to milliliters (ml)
pub fn volume_to_milliliters(volume: &VolumeType) -> f64 {
    match volume.unit {
        VolumeUnitType::Ml => volume.value,
        VolumeUnitType::L => volume.value * 1000.0,
        VolumeUnitType::Tsp => volume.value * 4.92892,
        VolumeUnitType::Tbsp => volume.value * 14.7868,
        VolumeUnitType::Floz => volume.value * 29.5735,
        VolumeUnitType::Cup => volume.value * 236.588,
        VolumeUnitType::Pt => volume.value * 473.176,
        VolumeUnitType::Qt => volume.value * 946.353,
        VolumeUnitType::Gal => volume.value * 3785.41,
        VolumeUnitType::Bbl => volume.value * 117347.77,
        VolumeUnitType::Ifloz => volume.value * 28.4131,
        VolumeUnitType::Ipt => volume.value * 568.261,
        VolumeUnitType::Iqt => volume.value * 1136.52,
        VolumeUnitType::Igal => volume.value * 4546.09,
        VolumeUnitType::Ibbl => volume.value * 163659.24,
    }
}

/// Convert any VolumeType to liters (l)
pub fn volume_to_liters(volume: &VolumeType) -> f64 {
    volume_to_milliliters(volume) / 1000.0
}

/// Convert any VolumeType to US gallons (gal)
pub fn volume_to_gallons(volume: &VolumeType) -> f64 {
    volume_to_milliliters(volume) / 3785.41
}

/// Convert any VolumeType to Imperial gallons (igal)
pub fn volume_to_imperial_gallons(volume: &VolumeType) -> f64 {
    volume_to_milliliters(volume) / 4546.09
}

// ============================================================================
// MASS CONVERSIONS
// ============================================================================

/// Convert any MassType to grams (g)
pub fn mass_to_grams(mass: &MassType) -> f64 {
    match mass.unit {
        MassUnitType::Mg => mass.value / 1000.0,
        MassUnitType::G => mass.value,
        MassUnitType::Kg => mass.value * 1000.0,
        MassUnitType::Lb => mass.value * 453.592,
        MassUnitType::Oz => mass.value * 28.3495,
    }
}

/// Convert any MassType to kilograms (kg)
pub fn mass_to_kilograms(mass: &MassType) -> f64 {
    mass_to_grams(mass) / 1000.0
}

/// Convert any MassType to pounds (lb)
pub fn mass_to_pounds(mass: &MassType) -> f64 {
    mass_to_grams(mass) / 453.592
}

/// Convert any MassType to ounces (oz)
pub fn mass_to_ounces(mass: &MassType) -> f64 {
    mass_to_grams(mass) / 28.3495
}

// ============================================================================
// TIME CONVERSIONS
// ============================================================================

/// Convert any TimeType to seconds (sec).
///
/// Returns f64 despite TimeType.value being i64, because all downstream
/// calculations (brewing formulas, unit conversions) operate in f64.
pub fn time_to_seconds(time: &TimeType) -> f64 {
    let v = time.value as f64;
    match time.unit {
        TimeUnitType::Sec => v,
        TimeUnitType::Min => v * 60.0,
        TimeUnitType::Hr => v * 3600.0,
        TimeUnitType::Day => v * 86400.0,
        TimeUnitType::Week => v * 604800.0,
    }
}

/// Convert any TimeType to minutes (min)
pub fn time_to_minutes(time: &TimeType) -> f64 {
    time_to_seconds(time) / 60.0
}

/// Convert any TimeType to hours (hr)
pub fn time_to_hours(time: &TimeType) -> f64 {
    time_to_seconds(time) / 3600.0
}

/// Convert any TimeType to days (day)
pub fn time_to_days(time: &TimeType) -> f64 {
    time_to_seconds(time) / 86400.0
}

// ============================================================================
// TEMPERATURE CONVERSIONS
// ============================================================================

/// Convert any TemperatureType to Celsius (C)
pub fn temperature_to_celsius(temp: &TemperatureType) -> f64 {
    match temp.unit {
        TemperatureUnitType::C => temp.value,
        TemperatureUnitType::F => (temp.value - 32.0) * (5.0 / 9.0),
    }
}

/// Convert any TemperatureType to Fahrenheit (F)
pub fn temperature_to_fahrenheit(temp: &TemperatureType) -> f64 {
    match temp.unit {
        TemperatureUnitType::C => temp.value * (9.0 / 5.0) + 32.0,
        TemperatureUnitType::F => temp.value,
    }
}

// ============================================================================
// COLOR CONVERSIONS
// ============================================================================

/// Convert SRM to EBC
pub fn srm_to_ebc(srm: f64) -> f64 {
    1.97 * srm
}

/// Convert EBC to SRM
pub fn ebc_to_srm(ebc: f64) -> f64 {
    ebc / 1.97
}

/// Convert Lovibond to SRM
pub fn lovibond_to_srm(lovibond: f64) -> f64 {
    1.3546 * lovibond - 0.76
}

/// Convert SRM to Lovibond
pub fn srm_to_lovibond(srm: f64) -> f64 {
    (srm + 0.76) / 1.3546
}

/// Convert any ColorType to SRM
pub fn color_to_srm(color: &ColorType) -> f64 {
    match color.unit {
        ColorUnitType::Srm => color.value,
        ColorUnitType::Ebc => ebc_to_srm(color.value),
        ColorUnitType::Lovi => lovibond_to_srm(color.value),
    }
}

/// Convert any ColorType to EBC
pub fn color_to_ebc(color: &ColorType) -> f64 {
    srm_to_ebc(color_to_srm(color))
}

/// Convert any ColorType to Lovibond
pub fn color_to_lovibond(color: &ColorType) -> f64 {
    srm_to_lovibond(color_to_srm(color))
}

// ============================================================================
// PRESSURE CONVERSIONS
// ============================================================================

/// Convert any PressureType to kilopascals (kPa)
pub fn pressure_to_kilopascals(pressure: &PressureType) -> f64 {
    match pressure.unit {
        PressureUnitType::KPa => pressure.value,
        PressureUnitType::Psi => pressure.value * 6.89476,
        PressureUnitType::Bar => pressure.value * 100.0,
    }
}

/// Convert any PressureType to PSI
pub fn pressure_to_psi(pressure: &PressureType) -> f64 {
    pressure_to_kilopascals(pressure) / 6.89476
}

/// Convert any PressureType to bar
pub fn pressure_to_bar(pressure: &PressureType) -> f64 {
    pressure_to_kilopascals(pressure) / 100.0
}

// ============================================================================
// GRAVITY CONVERSIONS
// ============================================================================

/// Convert any GravityType to specific gravity (sg)
pub fn gravity_to_sg(gravity: &GravityType) -> f64 {
    match gravity.unit {
        GravityUnitType::Sg => gravity.value,
        // Plato to SG: SG = 1 + (Plato / (258.6 - ((Plato / 258.2) * 227.1)))
        GravityUnitType::Plato => {
            1.0 + gravity.value / (258.6 - (gravity.value / 258.2) * 227.1)
        }
        // Brix to SG (using same formula as Plato, as they're very similar)
        GravityUnitType::Brix => {
            1.0 + gravity.value / (258.6 - (gravity.value / 258.2) * 227.1)
        }
    }
}

/// Convert any GravityType to Plato
pub fn gravity_to_plato(gravity: &GravityType) -> f64 {
    match gravity.unit {
        GravityUnitType::Sg => {
            // SG to Plato: Plato = -616.868 + 1111.14 * SG - 630.272 * SG^2 + 135.997 * SG^3
            let sg = gravity.value;
            -616.868 + 1111.14 * sg - 630.272 * sg.powi(2) + 135.997 * sg.powi(3)
        }
        GravityUnitType::Plato => gravity.value,
        GravityUnitType::Brix => gravity.value,
    }
}

/// Convert any GravityType to Brix
pub fn gravity_to_brix(gravity: &GravityType) -> f64 {
    gravity_to_plato(gravity)
}

// ============================================================================
// CONCENTRATION CONVERSIONS
// ============================================================================

/// Convert any ConcentrationType to mg/l (milligrams per liter)
pub fn concentration_to_mg_per_l(concentration: &ConcentrationType) -> f64 {
    match concentration.unit {
        ConcentrationUnitType::MgL => concentration.value,
        ConcentrationUnitType::Ppm => concentration.value, // ppm ≡ mg/l for aqueous solutions
        ConcentrationUnitType::Ppb => concentration.value / 1000.0,
    }
}

/// Convert any ConcentrationType to ppm (parts per million)
pub fn concentration_to_ppm(concentration: &ConcentrationType) -> f64 {
    concentration_to_mg_per_l(concentration)
}

/// Convert any ConcentrationType to ppb (parts per billion)
pub fn concentration_to_ppb(concentration: &ConcentrationType) -> f64 {
    concentration_to_mg_per_l(concentration) * 1000.0
}

// ============================================================================
// SPECIFIC VOLUME CONVERSIONS
// ============================================================================

/// Convert any SpecificVolumeType to l/kg (liters per kilogram)
pub fn specific_volume_to_l_per_kg(specific_volume: &SpecificVolumeType) -> f64 {
    match specific_volume.unit {
        SpecificVolumeUnitType::LKg => specific_volume.value,
        SpecificVolumeUnitType::LG => specific_volume.value * 1000.0,
        SpecificVolumeUnitType::QtLb => specific_volume.value * 2.08635,
        SpecificVolumeUnitType::GalLb => specific_volume.value * 8.34540,
        SpecificVolumeUnitType::GalOz => specific_volume.value * 133.526,
        SpecificVolumeUnitType::FlozOz => specific_volume.value * 1.04318,
        SpecificVolumeUnitType::M3Kg => specific_volume.value * 1000.0,
        SpecificVolumeUnitType::Ft3Lb => specific_volume.value * 62.4279,
    }
}

/// Convert any SpecificVolumeType to qt/lb (quarts per pound)
pub fn specific_volume_to_qt_per_lb(specific_volume: &SpecificVolumeType) -> f64 {
    specific_volume_to_l_per_kg(specific_volume) / 2.08635
}

/// Convert any SpecificVolumeType to gal/lb (gallons per pound)
pub fn specific_volume_to_gal_per_lb(specific_volume: &SpecificVolumeType) -> f64 {
    specific_volume_to_l_per_kg(specific_volume) / 8.34540
}

/// Convert any SpecificVolumeType to gal/kg (gallons per kilogram)
pub fn specific_volume_to_gal_per_kg(specific_volume: &SpecificVolumeType) -> f64 {
    specific_volume_to_l_per_kg(specific_volume) * 0.264172
}

// ============================================================================
// ACIDITY CONVERSIONS
// ============================================================================

/// Convert any AcidityType to pH value
pub fn acidity_to_ph(acidity: &AcidityType) -> f64 {
    match acidity.unit {
        AcidityUnitType::PH => acidity.value,
    }
}

// ============================================================================
// CARBONATION CONVERSIONS
// ============================================================================

/// Convert any CarbonationType to volumes of CO2
pub fn carbonation_to_volumes(carbonation: &CarbonationType) -> f64 {
    match carbonation.unit {
        CarbonationUnitType::Vols => carbonation.value,
        // g/l to volumes: 1 volume = 1.96 g/l at STP
        CarbonationUnitType::GL => carbonation.value / 1.96,
    }
}

/// Convert any CarbonationType to g/l (grams per liter)
pub fn carbonation_to_grams_per_liter(carbonation: &CarbonationType) -> f64 {
    match carbonation.unit {
        CarbonationUnitType::Vols => carbonation.value * 1.96,
        CarbonationUnitType::GL => carbonation.value,
    }
}

// ============================================================================
// BITTERNESS CONVERSIONS
// ============================================================================

/// Convert any BitternessType to IBUs
pub fn bitterness_to_ibu(bitterness: &BitternessType) -> f64 {
    match bitterness.unit {
        BitternessUnitType::IbUs => bitterness.value,
    }
}

// ============================================================================
// PERCENT CONVERSIONS
// ============================================================================

/// Convert any PercentType to decimal (0-1 range)
pub fn percent_to_decimal(percent: &PercentType) -> f64 {
    match percent.unit {
        PercentUnitType::X => percent.value / 100.0,
    }
}

/// Convert any PercentType to percentage value (0-100 range)
pub fn percent_to_value(percent: &PercentType) -> f64 {
    match percent.unit {
        PercentUnitType::X => percent.value,
    }
}
