import type {
  VolumeType,
  MassType,
  TimeType,
  TemperatureType,
  ColorType,
  PressureType,
  GravityType,
  ConcentrationType,
  SpecificVolumeType,
  AcidityType,
  CarbonationType,
  BitternessType,
  PercentType,
} from "@beerjson/beerjson";

/**
 * Unit conversion utilities for BeerJSON types.
 * All conversions support the complete set of units defined in the BeerJSON format.
 */

// ============================================================================
// VOLUME CONVERSIONS
// ============================================================================

/**
 * Convert any VolumeType to milliliters (ml)
 */
export function volumeToMilliliters(volume: VolumeType): number {
  switch (volume.unit) {
    case "ml":
      return volume.value;
    case "l":
      return volume.value * 1000;
    case "tsp":
      return volume.value * 4.92892;
    case "tbsp":
      return volume.value * 14.7868;
    case "floz":
      return volume.value * 29.5735;
    case "cup":
      return volume.value * 236.588;
    case "pt":
      return volume.value * 473.176;
    case "qt":
      return volume.value * 946.353;
    case "gal":
      return volume.value * 3785.41;
    case "bbl":
      return volume.value * 117347.77;
    case "ifloz":
      return volume.value * 28.4131;
    case "ipt":
      return volume.value * 568.261;
    case "iqt":
      return volume.value * 1136.52;
    case "igal":
      return volume.value * 4546.09;
    case "ibbl":
      return volume.value * 163659.24;
    default:
      throw Error(`Unrecognized volume unit: ${volume.unit}`);
  }
}

/**
 * Convert any VolumeType to liters (l)
 */
export function volumeToLiters(volume: VolumeType): number {
  return volumeToMilliliters(volume) / 1000;
}

/**
 * Convert any VolumeType to US gallons (gal)
 */
export function volumeToGallons(volume: VolumeType): number {
  return volumeToMilliliters(volume) / 3785.41;
}

/**
 * Convert any VolumeType to Imperial gallons (igal)
 */
export function volumeToImperialGallons(volume: VolumeType): number {
  return volumeToMilliliters(volume) / 4546.09;
}

// ============================================================================
// MASS CONVERSIONS
// ============================================================================

/**
 * Convert any MassType to grams (g)
 */
export function massToGrams(mass: MassType): number {
  switch (mass.unit) {
    case "mg":
      return mass.value / 1000;
    case "g":
      return mass.value;
    case "kg":
      return mass.value * 1000;
    case "lb":
      return mass.value * 453.592;
    case "oz":
      return mass.value * 28.3495;
    default:
      throw Error(`Unrecognized mass unit: ${mass.unit}`);
  }
}

/**
 * Convert any MassType to kilograms (kg)
 */
export function massToKilograms(mass: MassType): number {
  return massToGrams(mass) / 1000;
}

/**
 * Convert any MassType to pounds (lb)
 */
export function massToPounds(mass: MassType): number {
  return massToGrams(mass) / 453.592;
}

/**
 * Convert any MassType to ounces (oz)
 */
export function massToOunces(mass: MassType): number {
  return massToGrams(mass) / 28.3495;
}

// ============================================================================
// TIME CONVERSIONS
// ============================================================================

/**
 * Convert any TimeType to seconds (sec)
 */
export function timeToSeconds(time: TimeType): number {
  switch (time.unit) {
    case "sec":
      return time.value;
    case "min":
      return time.value * 60;
    case "hr":
      return time.value * 3600;
    case "day":
      return time.value * 86400;
    case "week":
      return time.value * 604800;
    default:
      throw Error(`Unrecognized time unit: ${time.unit}`);
  }
}

/**
 * Convert any TimeType to minutes (min)
 */
export function timeToMinutes(time: TimeType): number {
  return timeToSeconds(time) / 60;
}

/**
 * Convert any TimeType to hours (hr)
 */
export function timeToHours(time: TimeType): number {
  return timeToSeconds(time) / 3600;
}

/**
 * Convert any TimeType to days (day)
 */
export function timeToDays(time: TimeType): number {
  return timeToSeconds(time) / 86400;
}

// ============================================================================
// TEMPERATURE CONVERSIONS
// ============================================================================

/**
 * Convert any TemperatureType to Celsius (C)
 */
export function temperatureToCelsius(temp: TemperatureType): number {
  switch (temp.unit) {
    case "C":
      return temp.value;
    case "F":
      return (temp.value - 32) * (5 / 9);
    default:
      throw Error(`Unrecognized temperature unit: ${temp.unit}`);
  }
}

/**
 * Convert any TemperatureType to Fahrenheit (F)
 */
export function temperatureToFahrenheit(temp: TemperatureType): number {
  switch (temp.unit) {
    case "C":
      return temp.value * (9 / 5) + 32;
    case "F":
      return temp.value;
    default:
      throw Error(`Unrecognized temperature unit: ${temp.unit}`);
  }
}

// ============================================================================
// COLOR CONVERSIONS
// ============================================================================

/**
 * Convert SRM to EBC
 */
export function srmToEbc(srm: number): number {
  return 1.97 * srm;
}

/**
 * Convert EBC to SRM
 */
export function ebcToSrm(ebc: number): number {
  return ebc / 1.97;
}

/**
 * Convert Lovibond to SRM
 */
export function lovibondToSrm(lovibond: number): number {
  return 1.3546 * lovibond - 0.76;
}

/**
 * Convert SRM to Lovibond
 */
export function srmToLovibond(srm: number): number {
  return (srm + 0.76) / 1.3546;
}

/**
 * Convert any ColorType to SRM
 */
export function colorToSrm(color: ColorType): number {
  switch (color.unit) {
    case "SRM":
      return color.value;
    case "EBC":
      return ebcToSrm(color.value);
    case "Lovi":
      return lovibondToSrm(color.value);
    default:
      throw Error(`Unrecognized color unit: ${color.unit}`);
  }
}

/**
 * Convert any ColorType to EBC
 */
export function colorToEbc(color: ColorType): number {
  return srmToEbc(colorToSrm(color));
}

/**
 * Convert any ColorType to Lovibond
 */
export function colorToLovibond(color: ColorType): number {
  return srmToLovibond(colorToSrm(color));
}

// ============================================================================
// PRESSURE CONVERSIONS
// ============================================================================

/**
 * Convert any PressureType to kilopascals (kPa)
 */
export function pressureToKilopascals(pressure: PressureType): number {
  switch (pressure.unit) {
    case "kPa":
      return pressure.value;
    case "psi":
      return pressure.value * 6.89476;
    case "bar":
      return pressure.value * 100;
    default:
      throw Error(`Unrecognized pressure unit: ${pressure.unit}`);
  }
}

/**
 * Convert any PressureType to PSI
 */
export function pressureToPsi(pressure: PressureType): number {
  return pressureToKilopascals(pressure) / 6.89476;
}

/**
 * Convert any PressureType to bar
 */
export function pressureToBar(pressure: PressureType): number {
  return pressureToKilopascals(pressure) / 100;
}

// ============================================================================
// GRAVITY CONVERSIONS
// ============================================================================

/**
 * Convert any GravityType to specific gravity (sg)
 */
export function gravityToSG(gravity: GravityType): number {
  switch (gravity.unit) {
    case "sg":
      return gravity.value;
    case "plato":
      // Plato to SG: SG = 1 + (Plato / (258.6 - ((Plato / 258.2) * 227.1)))
      return (
        1 +
        gravity.value / (258.6 - (gravity.value / 258.2) * 227.1)
      );
    case "brix":
      // Brix to SG (using same formula as Plato, as they're very similar)
      return (
        1 +
        gravity.value / (258.6 - (gravity.value / 258.2) * 227.1)
      );
    default:
      throw Error(`Unrecognized gravity unit: ${gravity.unit}`);
  }
}

/**
 * Convert any GravityType to Plato
 */
export function gravityToPlato(gravity: GravityType): number {
  switch (gravity.unit) {
    case "sg":
      // SG to Plato: Plato = -616.868 + 1111.14 * SG - 630.272 * SG^2 + 135.997 * SG^3
      const sg = gravity.value;
      return (
        -616.868 +
        1111.14 * sg -
        630.272 * Math.pow(sg, 2) +
        135.997 * Math.pow(sg, 3)
      );
    case "plato":
      return gravity.value;
    case "brix":
      return gravity.value;
    default:
      throw Error(`Unrecognized gravity unit: ${gravity.unit}`);
  }
}

/**
 * Convert any GravityType to Brix
 */
export function gravityToBrix(gravity: GravityType): number {
  return gravityToPlato(gravity);
}

// ============================================================================
// CONCENTRATION CONVERSIONS
// ============================================================================

/**
 * Convert any ConcentrationType to mg/l (milligrams per liter)
 */
export function concentrationToMgPerL(concentration: ConcentrationType): number {
  switch (concentration.unit) {
    case "mg/l":
      return concentration.value;
    case "ppm":
      return concentration.value; // ppm is equivalent to mg/l for aqueous solutions
    case "ppb":
      return concentration.value / 1000;
    default:
      throw Error(`Unrecognized concentration unit: ${concentration.unit}`);
  }
}

/**
 * Convert any ConcentrationType to ppm (parts per million)
 */
export function concentrationToPpm(concentration: ConcentrationType): number {
  return concentrationToMgPerL(concentration);
}

/**
 * Convert any ConcentrationType to ppb (parts per billion)
 */
export function concentrationToPpb(concentration: ConcentrationType): number {
  return concentrationToMgPerL(concentration) * 1000;
}

// ============================================================================
// SPECIFIC VOLUME CONVERSIONS
// ============================================================================

/**
 * Convert any SpecificVolumeType to l/kg (liters per kilogram)
 */
export function specificVolumeToLPerKg(
  specificVolume: SpecificVolumeType
): number {
  switch (specificVolume.unit) {
    case "l/kg":
      return specificVolume.value;
    case "l/g":
      return specificVolume.value * 1000;
    case "qt/lb":
      return specificVolume.value * 2.08635;
    case "gal/lb":
      return specificVolume.value * 8.34540;
    case "gal/oz":
      return specificVolume.value * 133.526;
    case "floz/oz":
      return specificVolume.value * 1.04318;
    case "m^3/kg":
      return specificVolume.value * 1000;
    case "ft^3/lb":
      return specificVolume.value * 62.4279;
    default:
      throw Error(`Unrecognized specific volume unit: ${specificVolume.unit}`);
  }
}

/**
 * Convert any SpecificVolumeType to qt/lb (quarts per pound)
 */
export function specificVolumeToQtPerLb(
  specificVolume: SpecificVolumeType
): number {
  return specificVolumeToLPerKg(specificVolume) / 2.08635;
}

/**
 * Convert any SpecificVolumeType to gal/lb (gallons per pound)
 */
export function specificVolumeToGalPerLb(
  specificVolume: SpecificVolumeType
): number {
  return specificVolumeToLPerKg(specificVolume) / 8.34540;
}

/**
 * Convert any SpecificVolumeType to gal/kg (gallons per kilogram)
 */
export function specificVolumeToGallonsPerKilogram(
  specificVolume: SpecificVolumeType
): number {
  return specificVolumeToLPerKg(specificVolume) * 0.264172;
}

// ============================================================================
// ACIDITY CONVERSIONS
// ============================================================================

/**
 * Convert any AcidityType to pH value
 */
export function acidityToPh(acidity: AcidityType): number {
  switch (acidity.unit) {
    case "pH":
      return acidity.value;
    default:
      throw Error(`Unrecognized acidity unit: ${acidity.unit}`);
  }
}

// ============================================================================
// CARBONATION CONVERSIONS
// ============================================================================

/**
 * Convert any CarbonationType to volumes of CO2
 */
export function carbonationToVolumes(carbonation: CarbonationType): number {
  switch (carbonation.unit) {
    case "vols":
      return carbonation.value;
    case "g/l":
      // g/l to volumes: 1 volume = 1.96 g/l at STP
      return carbonation.value / 1.96;
    default:
      throw Error(`Unrecognized carbonation unit: ${carbonation.unit}`);
  }
}

/**
 * Convert any CarbonationType to g/l (grams per liter)
 */
export function carbonationToGramsPerLiter(
  carbonation: CarbonationType
): number {
  switch (carbonation.unit) {
    case "vols":
      return carbonation.value * 1.96;
    case "g/l":
      return carbonation.value;
    default:
      throw Error(`Unrecognized carbonation unit: ${carbonation.unit}`);
  }
}

// ============================================================================
// BITTERNESS CONVERSIONS
// ============================================================================

/**
 * Convert any BitternessType to IBUs
 */
export function bitternessToIBU(bitterness: BitternessType): number {
  switch (bitterness.unit) {
    case "IBUs":
      return bitterness.value;
    default:
      throw Error(`Unrecognized bitterness unit: ${bitterness.unit}`);
  }
}

// ============================================================================
// PERCENT CONVERSIONS
// ============================================================================

/**
 * Convert any PercentType to decimal (0-1 range)
 */
export function percentToDecimal(percent: PercentType): number {
  switch (percent.unit) {
    case "%":
      return percent.value / 100;
    default:
      throw Error(`Unrecognized percent unit: ${percent.unit}`);
  }
}

/**
 * Convert any PercentType to percentage value (0-100 range)
 */
export function percentToValue(percent: PercentType): number {
  switch (percent.unit) {
    case "%":
      return percent.value;
    default:
      throw Error(`Unrecognized percent unit: ${percent.unit}`);
  }
}
