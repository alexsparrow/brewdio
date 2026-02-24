//! Water Calculator
//! Comprehensive water volume and temperature calculations for brewing.
//!
//! Calculates total water needed, strike water volume/temperature,
//! sparge water volume/temperature, and stage volumes.
//! Strike temperature uses the Palmer formula:
//!   T_strike = (0.2/R) × (T_mash − T_grain) + T_mash

use crate::{beerjson_types::*, brewing_model};
use crate::units::{
    mass_to_pounds, specific_volume_to_gal_per_lb, temperature_to_fahrenheit, time_to_hours,
    volume_to_gallons,
};

// ============================================================================
// TYPES
// ============================================================================

brewing_model!{
pub struct WaterCalculatorOptions {
    pub volume_unit: VolumeUnitType,
    pub temperature_unit: TemperatureUnitType,
}
}

impl Default for WaterCalculatorOptions {
    fn default() -> Self {
        Self {
            volume_unit: VolumeUnitType::L,
            temperature_unit: TemperatureUnitType::C,
        }
    }
}

brewing_model!{
pub struct WaterCalculatorInput {
    pub target_batch_size: VolumeType,
    pub boil_time: TimeType,
    pub grain_bill: Vec<FermentableAdditionType>,
    pub mash_steps: Vec<MashStepType>,
    pub grain_temperature: TemperatureType,
    pub equipment: Option<EquipmentType>,
    pub units: Option<WaterCalculatorOptions>,
}
}

#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct StageVolumes {
    pub volume_in: VolumeType,
    pub volume_out: VolumeType,
}

#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct WaterStrikeResult {
    pub volume: VolumeType,
    pub temperature: TemperatureType,
}

#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct WaterSpargeResult {
    pub volume: VolumeType,
    pub temperature: TemperatureType,
}

#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct WaterStages {
    pub mash: StageVolumes,
    pub kettle: StageVolumes,
    pub fermenter: StageVolumes,
}

#[derive(Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct WaterCalculatorResult {
    pub total_water_needed: VolumeType,
    pub strike_water: WaterStrikeResult,
    pub sparge_water: Option<WaterSpargeResult>,
    pub stages: WaterStages,
    pub calculated_stages: Vec<CalculatedWaterStage>,
}

// --- Stage-based calculation types ---

brewing_model!{
#[serde(rename_all = "camelCase")]
pub enum WaterStageType {
    Source,
    Mash,
    Boil,
    Fermenter,
    Packaging,
}
}

brewing_model!{
#[serde(rename_all = "camelCase")]
pub enum WaterLossType {
    Flat,
    Rate,
}
}

brewing_model!{
#[serde(rename_all = "camelCase")]
pub struct WaterLoss {
    pub id: String,
    pub label: String,
    pub value: f64,
    #[serde(rename = "type")]
    pub loss_type: WaterLossType,
    pub unit: String,
}
}

brewing_model!{
#[serde(rename_all = "camelCase")]
pub struct WaterStage {
    pub id: String,
    pub label: String,
    pub stage_type: WaterStageType,
    pub losses: Vec<WaterLoss>,
}
}

brewing_model!{
#[serde(rename_all = "camelCase")]
pub struct CalculatedWaterStage {
    pub id: String,
    pub label: String,
    pub stage_type: WaterStageType,
    pub losses: Vec<WaterLoss>,
    pub volume_in: f64,
    pub volume_out: f64,
    pub total_loss: f64,
    pub is_source: bool,
}
}

brewing_model!{
#[serde(rename_all = "camelCase")]
pub struct StageCalculationResult {
    pub total_water_needed: f64,
    pub stages: Vec<CalculatedWaterStage>,
}
}

// ============================================================================
// DEFAULTS
// ============================================================================

const GRAIN_ABSORPTION_GAL_PER_LB: f64 = 0.125;
const MASH_TUN_DEADSPACE_LOSS_GAL: f64 = 0.25;
const BOIL_OFF_RATE_GAL_PER_HR: f64 = 1.0;
const KETTLE_TRUB_LOSS_GAL: f64 = 0.5;
const FERMENTER_TRUB_LOSS_GAL: f64 = 0.5;
const PACKAGING_TRANSFER_LOSS_GAL: f64 = 0.1;
const SPARGE_TEMPERATURE_F: f64 = 168.0;
const WATER_TO_GRAIN_RATIO_QT_PER_LB: f64 = 1.25;

// ============================================================================
// UNIT OUTPUT HELPERS
// ============================================================================

const GAL_TO_L: f64 = 3.78541;

fn gal_to_volume(gal: f64, unit: &VolumeUnitType) -> VolumeType {
    let value = match unit {
        VolumeUnitType::Gal => gal,
        VolumeUnitType::L => gal * GAL_TO_L,
        VolumeUnitType::Ml => gal * GAL_TO_L * 1000.0,
        VolumeUnitType::Qt => gal * 4.0,
        VolumeUnitType::Pt => gal * 8.0,
        VolumeUnitType::Floz => gal * 128.0,
        VolumeUnitType::Bbl => gal / 31.0,
        _ => gal * GAL_TO_L, // default to liters
    };
    VolumeType {
        value,
        unit: unit.clone(),
    }
}

fn f_to_temp(f: f64, unit: &TemperatureUnitType) -> TemperatureType {
    match unit {
        TemperatureUnitType::F => TemperatureType {
            value: f,
            unit: TemperatureUnitType::F,
        },
        TemperatureUnitType::C => TemperatureType {
            value: (f - 32.0) * (5.0 / 9.0),
            unit: TemperatureUnitType::C,
        },
    }
}

// ============================================================================
// EQUIPMENT HELPERS
// ============================================================================

fn find_equipment_item<'a>(
    equipment: Option<&'a EquipmentType>,
    form: EquipmentItemTypeForm,
) -> Option<&'a EquipmentItemType> {
    equipment
        .and_then(|e| e.equipment_items.iter().find(|item| item.form == form))
}

fn get_grain_absorption_rate(equipment: Option<&EquipmentType>) -> f64 {
    find_equipment_item(equipment, EquipmentItemTypeForm::MashTun)
        .and_then(|mash_tun| mash_tun.grain_absorption_rate.as_ref())
        .map(|rate| specific_volume_to_gal_per_lb(rate))
        .unwrap_or(GRAIN_ABSORPTION_GAL_PER_LB)
}

fn get_mash_tun_loss(equipment: Option<&EquipmentType>) -> f64 {
    find_equipment_item(equipment, EquipmentItemTypeForm::MashTun)
        .map(|mash_tun| volume_to_gallons(&mash_tun.loss))
        .unwrap_or(MASH_TUN_DEADSPACE_LOSS_GAL)
}

fn get_boil_off_rate(equipment: Option<&EquipmentType>) -> f64 {
    find_equipment_item(equipment, EquipmentItemTypeForm::BrewKettle)
        .and_then(|kettle| kettle.boil_rate_per_hour.as_ref())
        .map(|rate| volume_to_gallons(rate))
        .unwrap_or(BOIL_OFF_RATE_GAL_PER_HR)
}

fn get_kettle_loss(equipment: Option<&EquipmentType>) -> f64 {
    find_equipment_item(equipment, EquipmentItemTypeForm::BrewKettle)
        .map(|kettle| volume_to_gallons(&kettle.loss))
        .unwrap_or(KETTLE_TRUB_LOSS_GAL)
}

fn get_fermenter_loss(equipment: Option<&EquipmentType>) -> f64 {
    find_equipment_item(equipment, EquipmentItemTypeForm::Fermenter)
        .map(|fermenter| volume_to_gallons(&fermenter.loss))
        .unwrap_or(FERMENTER_TRUB_LOSS_GAL)
}

fn get_packaging_loss(equipment: Option<&EquipmentType>) -> f64 {
    find_equipment_item(equipment, EquipmentItemTypeForm::PackagingVessel)
        .map(|pkg| volume_to_gallons(&pkg.loss))
        .unwrap_or(PACKAGING_TRANSFER_LOSS_GAL)
}

// ============================================================================
// GRAIN WEIGHT
// ============================================================================

fn total_grain_weight_lb(grain_bill: &[FermentableAdditionType]) -> f64 {
    grain_bill
        .iter()
        .filter(|f| matches!(f.type_, FermentableAdditionTypeType::Grain))
        .map(|f| match &f.amount {
            FermentableAdditionTypeAmount::MassType(mass) => mass_to_pounds(mass),
            FermentableAdditionTypeAmount::VolumeType(_) => 0.0,
        })
        .sum()
}

// ============================================================================
// STRIKE TEMPERATURE (Palmer formula)
// ============================================================================

/// T_strike = (0.2 / R) × (T_mash − T_grain) + T_mash
fn calc_strike_temperature(
    water_to_grain_ratio_qt_per_lb: f64,
    target_mash_temp_f: f64,
    grain_temp_f: f64,
) -> f64 {
    (0.2 / water_to_grain_ratio_qt_per_lb) * (target_mash_temp_f - grain_temp_f)
        + target_mash_temp_f
}

// ============================================================================
// STAGE-BASED CALCULATION
// ============================================================================

/// Build default brewing stages from equipment parameters.
pub fn build_stages(
    equipment: Option<&EquipmentType>,
    grain_weight_lb: f64,
) -> Vec<WaterStage> {
    let grain_absorption = get_grain_absorption_rate(equipment);
    let mash_tun_loss = get_mash_tun_loss(equipment);
    let boil_off_rate = get_boil_off_rate(equipment);
    let kettle_loss = get_kettle_loss(equipment);
    let fermenter_loss = get_fermenter_loss(equipment);
    let packaging_loss = get_packaging_loss(equipment);

    vec![
        WaterStage {
            id: "mash".to_string(),
            label: "Mash Tun".to_string(),
            stage_type: WaterStageType::Mash,
            losses: vec![
                WaterLoss {
                    id: "grainAbs".to_string(),
                    label: "Grain Absorption".to_string(),
                    value: grain_absorption * grain_weight_lb,
                    loss_type: WaterLossType::Flat,
                    unit: "gal".to_string(),
                },
                WaterLoss {
                    id: "tunDead".to_string(),
                    label: "Tun Deadspace".to_string(),
                    value: mash_tun_loss,
                    loss_type: WaterLossType::Flat,
                    unit: "gal".to_string(),
                },
            ],
        },
        WaterStage {
            id: "kettle".to_string(),
            label: "Boil Kettle".to_string(),
            stage_type: WaterStageType::Boil,
            losses: vec![
                WaterLoss {
                    id: "boilOff".to_string(),
                    label: "Boil Off Rate".to_string(),
                    value: boil_off_rate,
                    loss_type: WaterLossType::Rate,
                    unit: "gal/hr".to_string(),
                },
                WaterLoss {
                    id: "trub".to_string(),
                    label: "Trub / Hop Loss".to_string(),
                    value: kettle_loss,
                    loss_type: WaterLossType::Flat,
                    unit: "gal".to_string(),
                },
            ],
        },
        WaterStage {
            id: "fermenter".to_string(),
            label: "Fermenter".to_string(),
            stage_type: WaterStageType::Fermenter,
            losses: vec![
                WaterLoss {
                    id: "trub_ferm".to_string(),
                    label: "Yeast/Cake Loss".to_string(),
                    value: fermenter_loss,
                    loss_type: WaterLossType::Flat,
                    unit: "gal".to_string(),
                },
            ],
        },
        WaterStage {
            id: "packaging".to_string(),
            label: "Kegging".to_string(),
            stage_type: WaterStageType::Packaging,
            losses: vec![
                WaterLoss {
                    id: "lines".to_string(),
                    label: "Transfer Loss".to_string(),
                    value: packaging_loss,
                    loss_type: WaterLossType::Flat,
                    unit: "gal".to_string(),
                },
            ],
        },
    ]
}

/// Low-level stage-based water volume calculation.
/// Works backwards from target_volume through each stage, accumulating losses.
/// Returns calculated volumes for each stage plus a prepended source tank.
pub fn calculate_water_from_stages(
    target_volume: f64,
    boil_time_min: f64,
    stages: &[WaterStage],
) -> StageCalculationResult {
    let mut current_vol = target_volume;
    let mut processed: Vec<CalculatedWaterStage> = Vec::new();

    for stage in stages.iter().rev() {
        let mut total_loss = 0.0;
        for loss in &stage.losses {
            let val = match loss.loss_type {
                WaterLossType::Rate => loss.value * (boil_time_min / 60.0),
                WaterLossType::Flat => loss.value,
            };
            total_loss += val;
        }

        let volume_in = current_vol + total_loss;
        processed.push(CalculatedWaterStage {
            id: stage.id.clone(),
            label: stage.label.clone(),
            stage_type: stage.stage_type.clone(),
            losses: stage.losses.clone(),
            volume_in,
            volume_out: current_vol,
            total_loss,
            is_source: false,
        });
        current_vol = volume_in;
    }

    processed.reverse();

    let source = CalculatedWaterStage {
        id: "source".to_string(),
        label: "Strike Water".to_string(),
        stage_type: WaterStageType::Source,
        losses: vec![],
        volume_in: current_vol,
        volume_out: current_vol,
        total_loss: 0.0,
        is_source: true,
    };

    let mut all_stages = vec![source];
    all_stages.extend(processed);

    StageCalculationResult {
        total_water_needed: current_vol,
        stages: all_stages,
    }
}

// ============================================================================
// MAIN CALCULATION
// ============================================================================

pub fn calculate_water(input: &WaterCalculatorInput) -> WaterCalculatorResult {
    let units = input.units.as_ref();
    let vol_unit = units
        .map(|u| u.volume_unit.clone())
        .unwrap_or(VolumeUnitType::L);
    let temp_unit = units
        .map(|u| u.temperature_unit.clone())
        .unwrap_or(TemperatureUnitType::C);

    let target_vol_gal = volume_to_gallons(&input.target_batch_size);
    let boil_time_min = time_to_hours(&input.boil_time) * 60.0;
    let grain_weight_lb = total_grain_weight_lb(&input.grain_bill);
    let grain_temp_f = temperature_to_fahrenheit(&input.grain_temperature);

    // Build stages and calculate volumes
    let stages = build_stages(input.equipment.as_ref(), grain_weight_lb);
    let stage_result = calculate_water_from_stages(target_vol_gal, boil_time_min, &stages);
    let total_water_needed = stage_result.total_water_needed;

    // Determine whether a sparge step is present
    let has_sparge = input
        .mash_steps
        .iter()
        .any(|s| matches!(s.type_, MashStepTypeType::Sparge));

    // Strike water volume from first infusion mash step
    let first_infusion = input
        .mash_steps
        .iter()
        .find(|s| matches!(s.type_, MashStepTypeType::Infusion));

    let (mut strike_volume_gal, water_to_grain_ratio_qt_per_lb) = if !has_sparge {
        // No-sparge / BIAB: all water goes to strike
        let strike = total_water_needed;
        let ratio = if grain_weight_lb > 0.0 {
            (strike * 4.0) / grain_weight_lb
        } else {
            WATER_TO_GRAIN_RATIO_QT_PER_LB
        };
        (strike, ratio)
    } else if let Some(infusion) = first_infusion {
        if let Some(ref amount) = infusion.amount {
            let strike = volume_to_gallons(&amount);
            let ratio = if grain_weight_lb > 0.0 {
                (strike * 4.0) / grain_weight_lb
            } else {
                WATER_TO_GRAIN_RATIO_QT_PER_LB
            };
            (strike, ratio)
        } else {
            let ratio = WATER_TO_GRAIN_RATIO_QT_PER_LB;
            let strike = (ratio * grain_weight_lb) / 4.0;
            (strike, ratio)
        }
    } else {
        let ratio = WATER_TO_GRAIN_RATIO_QT_PER_LB;
        let strike = (ratio * grain_weight_lb) / 4.0;
        (strike, ratio)
    };

    // Clamp strike volume to total water needed
    strike_volume_gal = strike_volume_gal.min(total_water_needed);
    let sparge_volume_gal = total_water_needed - strike_volume_gal;

    // Strike temperature
    let target_mash_temp_f = first_infusion
        .map(|inf| temperature_to_fahrenheit(&inf.step_temperature))
        .unwrap_or(152.0);

    let strike_temp_f = if grain_weight_lb > 0.0 {
        calc_strike_temperature(
            water_to_grain_ratio_qt_per_lb,
            target_mash_temp_f,
            grain_temp_f,
        )
    } else {
        target_mash_temp_f
    };

    // Convert to output units
    let vol = |gal: f64| gal_to_volume(gal, &vol_unit);
    let temp = |f: f64| f_to_temp(f, &temp_unit);

    // Derive WaterStages from calculated stages for backward compat
    let find_stage = |id: &str| stage_result.stages.iter().find(|s| s.id == id);
    let mash_s = find_stage("mash").unwrap();
    let kettle_s = find_stage("kettle").unwrap();
    let fermenter_s = find_stage("fermenter").unwrap();

    // Convert calculated_stages volumes to output units
    let unit_factor = gal_to_volume(1.0, &vol_unit).value;
    let calculated_stages: Vec<CalculatedWaterStage> = stage_result
        .stages
        .iter()
        .map(|s| CalculatedWaterStage {
            id: s.id.clone(),
            label: s.label.clone(),
            stage_type: s.stage_type.clone(),
            losses: s.losses.clone(),
            volume_in: s.volume_in * unit_factor,
            volume_out: s.volume_out * unit_factor,
            total_loss: s.total_loss * unit_factor,
            is_source: s.is_source,
        })
        .collect();

    WaterCalculatorResult {
        total_water_needed: vol(total_water_needed),
        strike_water: WaterStrikeResult {
            volume: vol(strike_volume_gal),
            temperature: temp(strike_temp_f),
        },
        sparge_water: if has_sparge {
            Some(WaterSpargeResult {
                volume: vol(sparge_volume_gal),
                temperature: temp(SPARGE_TEMPERATURE_F),
            })
        } else {
            None
        },
        stages: WaterStages {
            mash: StageVolumes {
                volume_in: vol(mash_s.volume_in),
                volume_out: vol(mash_s.volume_out),
            },
            kettle: StageVolumes {
                volume_in: vol(kettle_s.volume_in),
                volume_out: vol(kettle_s.volume_out),
            },
            fermenter: StageVolumes {
                volume_in: vol(fermenter_s.volume_in),
                volume_out: vol(fermenter_s.volume_out),
            },
        },
        calculated_stages,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // HELPERS
    // ========================================================================

    fn grain(
        lbs: f64,
        type_: FermentableAdditionTypeType,
    ) -> FermentableAdditionType {
        FermentableAdditionType {
            name: "Test Grain".to_string(),
            type_,
            amount: FermentableAdditionTypeAmount::MassType(MassType {
                value: lbs,
                unit: MassUnitType::Lb,
            }),
            yield_: YieldType {
                fine_grind: Some(PercentType { value: 80.0, unit: PercentUnitType::X }),
                coarse_grind: None,
                fine_coarse_difference: None,
                potential: None,
            },
            color: ColorType { value: 3.0, unit: ColorUnitType::Srm },
            timing: None,
            grain_group: None,
            origin: None,
            producer: None,
            product_id: None,
        }
    }

    fn grain_kg(kg: f64) -> FermentableAdditionType {
        FermentableAdditionType {
            name: "Test Grain".to_string(),
            type_: FermentableAdditionTypeType::Grain,
            amount: FermentableAdditionTypeAmount::MassType(MassType {
                value: kg,
                unit: MassUnitType::Kg,
            }),
            yield_: YieldType {
                fine_grind: Some(PercentType { value: 80.0, unit: PercentUnitType::X }),
                coarse_grind: None,
                fine_coarse_difference: None,
                potential: None,
            },
            color: ColorType { value: 3.0, unit: ColorUnitType::Srm },
            timing: None,
            grain_group: None,
            origin: None,
            producer: None,
            product_id: None,
        }
    }

    fn infusion_step(
        temp_f: f64,
        time_min: i64,
        amount_gal: Option<f64>,
    ) -> MashStepType {
        MashStepType {
            name: "Mash".to_string(),
            type_: MashStepTypeType::Infusion,
            step_temperature: TemperatureType { value: temp_f, unit: TemperatureUnitType::F },
            step_time: TimeType { value: time_min, unit: TimeUnitType::Min },
            amount: amount_gal.map(|v| VolumeType { value: v, unit: VolumeUnitType::Gal }),
            description: None,
            end_ph: None,
            end_temperature: None,
            infuse_temperature: None,
            ramp_time: None,
            start_ph: None,
            water_grain_ratio: None,
        }
    }

    fn sparge_step() -> MashStepType {
        MashStepType {
            name: "Sparge".to_string(),
            type_: MashStepTypeType::Sparge,
            step_temperature: TemperatureType { value: 168.0, unit: TemperatureUnitType::F },
            step_time: TimeType { value: 10, unit: TimeUnitType::Min },
            amount: None,
            description: None,
            end_ph: None,
            end_temperature: None,
            infuse_temperature: None,
            ramp_time: None,
            start_ph: None,
            water_grain_ratio: None,
        }
    }

    struct TypicalInputBuilder {
        target_batch_size: VolumeType,
        boil_time: TimeType,
        grain_bill: Vec<FermentableAdditionType>,
        mash_steps: Vec<MashStepType>,
        grain_temperature: TemperatureType,
        equipment: Option<EquipmentType>,
        units: Option<WaterCalculatorOptions>,
    }

    impl TypicalInputBuilder {
        fn new() -> Self {
            Self {
                target_batch_size: VolumeType { value: 5.0, unit: VolumeUnitType::Gal },
                boil_time: TimeType { value: 60, unit: TimeUnitType::Min },
                grain_bill: vec![grain(10.0, FermentableAdditionTypeType::Grain)],
                mash_steps: vec![infusion_step(152.0, 60, None), sparge_step()],
                grain_temperature: TemperatureType { value: 68.0, unit: TemperatureUnitType::F },
                equipment: None,
                units: None,
            }
        }

        fn us_units(mut self) -> Self {
            self.units = Some(WaterCalculatorOptions {
                volume_unit: VolumeUnitType::Gal,
                temperature_unit: TemperatureUnitType::F,
            });
            self
        }

        fn calculate(&self) -> WaterCalculatorResult {
            let input = WaterCalculatorInput {
                target_batch_size: self.target_batch_size.clone(),
                boil_time: self.boil_time.clone(),
                grain_bill: self.grain_bill.clone(),
                mash_steps: self.mash_steps.clone(),
                grain_temperature: self.grain_temperature.clone(),
                equipment: self.equipment.clone(),
                units: self.units.as_ref().map(|u| WaterCalculatorOptions {
                    volume_unit: u.volume_unit.clone(),
                    temperature_unit: u.temperature_unit.clone(),
                }),
            };
            calculate_water(&input)
        }
    }

    // ========================================================================
    // TESTS
    // ========================================================================

    mod default_output_units {
        use super::*;

        #[test]
        fn defaults_to_litres_and_celsius() {
            let result = TypicalInputBuilder::new().calculate();
            assert!(matches!(result.total_water_needed.unit, VolumeUnitType::L));
            assert!(matches!(result.strike_water.temperature.unit, TemperatureUnitType::C));
            assert!(matches!(result.sparge_water.as_ref().unwrap().temperature.unit, TemperatureUnitType::C));
            assert!(matches!(result.stages.mash.volume_in.unit, VolumeUnitType::L));
        }
    }

    mod output_unit_selection {
        use super::*;

        #[test]
        fn returns_gallons_and_fahrenheit_when_requested() {
            let result = TypicalInputBuilder::new().us_units().calculate();
            assert!(matches!(result.total_water_needed.unit, VolumeUnitType::Gal));
            assert!(matches!(result.strike_water.temperature.unit, TemperatureUnitType::F));
            assert!(matches!(result.sparge_water.as_ref().unwrap().temperature.unit, TemperatureUnitType::F));
            assert!(matches!(result.stages.kettle.volume_in.unit, VolumeUnitType::Gal));
        }

        #[test]
        fn same_calculation_different_units_values_are_equivalent() {
            let mut builder = TypicalInputBuilder::new();
            builder.units = Some(WaterCalculatorOptions {
                volume_unit: VolumeUnitType::L,
                temperature_unit: TemperatureUnitType::C,
            });
            let metric = builder.calculate();

            let us = TypicalInputBuilder::new().us_units().calculate();
            // 1 gal ≈ 3.785 l
            assert!((metric.total_water_needed.value - us.total_water_needed.value * 3.78541).abs() < 0.1);
            // strike temp: C = (F - 32) * 5/9
            assert!(
                (metric.strike_water.temperature.value
                    - (us.strike_water.temperature.value - 32.0) * 5.0 / 9.0)
                    .abs()
                    < 0.5
            );
        }
    }

    mod typical_5_gal_all_grain_batch {
        use super::*;

        #[test]
        fn calculates_total_water_needed() {
            let result = TypicalInputBuilder::new().us_units().calculate();
            // target 5 + packaging 0.1 = 5.1
            // + fermenter 0.5 = 5.6
            // + trub 0.5 + boil-off 1.0 = 7.1
            // + grain absorption 1.25 + tun deadspace 0.25 = 8.6
            assert!((result.total_water_needed.value - 8.6).abs() < 0.05);
        }

        #[test]
        fn calculates_strike_and_sparge_volumes_that_sum_to_total() {
            let result = TypicalInputBuilder::new().us_units().calculate();
            let sum = result.strike_water.volume.value
                + result.sparge_water.as_ref().unwrap().volume.value;
            assert!((sum - result.total_water_needed.value).abs() < 1e-5);
        }

        #[test]
        fn strike_volume_defaults_from_water_grain_ratio() {
            let result = TypicalInputBuilder::new().us_units().calculate();
            // default ratio 1.25 qt/lb * 10 lb = 12.5 qt = 3.125 gal
            assert!((result.strike_water.volume.value - 3.125).abs() < 0.05);
        }

        #[test]
        fn sparge_volume_is_remainder() {
            let result = TypicalInputBuilder::new().us_units().calculate();
            assert!(
                (result.sparge_water.as_ref().unwrap().volume.value - (8.6 - 3.125)).abs() < 0.05
            );
        }
    }

    mod strike_temperature {
        use super::*;

        #[test]
        fn calculates_strike_temperature_using_palmer_formula() {
            // R = 1.25 qt/lb, T_mash = 152°F, T_grain = 68°F
            // T_strike = (0.2 / 1.25) * (152 - 68) + 152 = 165.44°F
            let result = TypicalInputBuilder::new().us_units().calculate();
            assert!((result.strike_water.temperature.value - 165.44).abs() < 0.5);
        }

        #[test]
        fn returns_strike_temperature_in_celsius_by_default() {
            let result = TypicalInputBuilder::new().calculate();
            // 165.44°F = 74.13°C
            assert!((result.strike_water.temperature.value - 74.13).abs() < 1.0);
            assert!(matches!(result.strike_water.temperature.unit, TemperatureUnitType::C));
        }

        #[test]
        fn higher_grain_temp_requires_lower_strike_temp() {
            let mut cold = TypicalInputBuilder::new().us_units();
            cold.grain_temperature = TemperatureType { value: 50.0, unit: TemperatureUnitType::F };
            let cold_result = cold.calculate();

            let mut warm = TypicalInputBuilder::new().us_units();
            warm.grain_temperature = TemperatureType { value: 80.0, unit: TemperatureUnitType::F };
            let warm_result = warm.calculate();

            assert!(cold_result.strike_water.temperature.value > warm_result.strike_water.temperature.value);
        }

        #[test]
        fn accepts_grain_temperature_in_celsius() {
            // 68°F = 20°C
            let in_f = TypicalInputBuilder::new().us_units().calculate();

            let mut in_c = TypicalInputBuilder::new().us_units();
            in_c.grain_temperature = TemperatureType { value: 20.0, unit: TemperatureUnitType::C };
            let in_c_result = in_c.calculate();

            assert!(
                (in_f.strike_water.temperature.value - in_c_result.strike_water.temperature.value)
                    .abs()
                    < 0.5
            );
        }
    }

    mod sparge_temperature {
        use super::*;

        #[test]
        fn is_always_168f_or_7556c() {
            let us = TypicalInputBuilder::new().us_units().calculate();
            assert!(
                (us.sparge_water.as_ref().unwrap().temperature.value - 168.0).abs() < 0.5
            );

            let metric = TypicalInputBuilder::new().calculate();
            assert!(
                (metric.sparge_water.as_ref().unwrap().temperature.value - 75.56).abs() < 1.0
            );
        }
    }

    mod equipment_overrides {
        use super::*;

        #[test]
        fn uses_equipment_losses_when_provided() {
            let mut builder = TypicalInputBuilder::new().us_units();
            builder.equipment = Some(EquipmentType {
                name: "My Brewery".to_string(),
                equipment_items: vec![
                    EquipmentItemType {
                        name: "Mash Tun".to_string(),
                        form: EquipmentItemTypeForm::MashTun,
                        maximum_volume: VolumeType { value: 15.0, unit: VolumeUnitType::Gal },
                        loss: VolumeType { value: 0.5, unit: VolumeUnitType::Gal },
                        grain_absorption_rate: Some(SpecificVolumeType {
                            value: 0.1,
                            unit: SpecificVolumeUnitType::GalLb,
                        }),
                        boil_rate_per_hour: None,
                        drain_rate_per_minute: None,
                        notes: None,
                        specific_heat: None,
                        type_: None,
                        weight: None,
                    },
                    EquipmentItemType {
                        name: "Kettle".to_string(),
                        form: EquipmentItemTypeForm::BrewKettle,
                        maximum_volume: VolumeType { value: 15.0, unit: VolumeUnitType::Gal },
                        loss: VolumeType { value: 1.0, unit: VolumeUnitType::Gal },
                        boil_rate_per_hour: Some(VolumeType { value: 1.5, unit: VolumeUnitType::Gal }),
                        drain_rate_per_minute: None,
                        grain_absorption_rate: None,
                        notes: None,
                        specific_heat: None,
                        type_: None,
                        weight: None,
                    },
                    EquipmentItemType {
                        name: "Fermenter".to_string(),
                        form: EquipmentItemTypeForm::Fermenter,
                        maximum_volume: VolumeType { value: 7.0, unit: VolumeUnitType::Gal },
                        loss: VolumeType { value: 0.25, unit: VolumeUnitType::Gal },
                        boil_rate_per_hour: None,
                        drain_rate_per_minute: None,
                        grain_absorption_rate: None,
                        notes: None,
                        specific_heat: None,
                        type_: None,
                        weight: None,
                    },
                ],
            });

            let result = builder.calculate();

            // target 5 + packaging 0.1 = 5.1
            // + fermenter 0.25 = 5.35
            // + trub 1.0 + boil-off 1.5 = 7.85
            // + grain absorption 1.0 + tun deadspace 0.5 = 9.35
            assert!((result.total_water_needed.value - 9.35).abs() < 0.05);
            assert!((result.stages.fermenter.volume_in.value - 5.35).abs() < 0.05);
        }
    }

    mod default_equipment {
        use super::*;

        #[test]
        fn uses_sensible_defaults_when_no_equipment_provided() {
            let result = TypicalInputBuilder::new().us_units().calculate();
            // fermenter loss = volumeIn - volumeOut = 0.5
            let loss = result.stages.fermenter.volume_in.value
                - result.stages.fermenter.volume_out.value;
            assert!((loss - 0.5).abs() < 0.05);
        }
    }

    mod batch_size_units {
        use super::*;

        #[test]
        fn liters_input_gives_same_result_as_equivalent_gallons() {
            let in_gal = TypicalInputBuilder::new().us_units().calculate();

            let mut in_l = TypicalInputBuilder::new().us_units();
            in_l.target_batch_size = VolumeType { value: 18.9271, unit: VolumeUnitType::L };
            let in_l_result = in_l.calculate();

            assert!(
                (in_gal.total_water_needed.value - in_l_result.total_water_needed.value).abs()
                    < 0.5
            );
        }
    }

    mod grain_weight_units {
        use super::*;

        #[test]
        fn kg_vs_lb_equivalence() {
            let in_lb = TypicalInputBuilder::new().us_units().calculate();

            let mut in_kg = TypicalInputBuilder::new().us_units();
            in_kg.grain_bill = vec![grain_kg(4.53592)];
            let in_kg_result = in_kg.calculate();

            assert!(
                (in_lb.total_water_needed.value - in_kg_result.total_water_needed.value).abs()
                    < 0.5
            );
        }
    }

    mod volume_accounting {
        use super::*;

        #[test]
        fn total_water_equals_strike_plus_sparge() {
            let result = TypicalInputBuilder::new().us_units().calculate();
            let sum = result.strike_water.volume.value
                + result.sparge_water.as_ref().unwrap().volume.value;
            assert!((sum - result.total_water_needed.value).abs() < 1e-5);
        }

        #[test]
        fn stage_volumes_chain_correctly() {
            let result = TypicalInputBuilder::new().us_units().calculate();
            // In the stage model, each stage's volume_out == next stage's volume_in
            assert!(
                (result.stages.kettle.volume_in.value
                    - result.stages.mash.volume_out.value)
                    .abs()
                    < 0.05
            );
            assert!(
                (result.stages.fermenter.volume_in.value
                    - result.stages.kettle.volume_out.value)
                    .abs()
                    < 0.05
            );
        }

        #[test]
        fn fermenter_output_feeds_packaging() {
            let result = TypicalInputBuilder::new().us_units().calculate();
            // Fermenter output = target (5.0) + packaging loss (0.1)
            assert!((result.stages.fermenter.volume_out.value - 5.1).abs() < 0.05);
        }

        #[test]
        fn fermenter_output_in_litres_matches_target_in_litres() {
            let mut builder = TypicalInputBuilder::new();
            builder.target_batch_size = VolumeType { value: 20.0, unit: VolumeUnitType::L };
            builder.units = Some(WaterCalculatorOptions {
                volume_unit: VolumeUnitType::L,
                temperature_unit: TemperatureUnitType::C,
            });
            let result = builder.calculate();
            assert!((result.stages.fermenter.volume_out.value - 20.0).abs() < 1.0);
        }
    }

    mod mash_step_amount_override {
        use super::*;

        #[test]
        fn uses_mash_step_amount_when_provided() {
            let mut builder = TypicalInputBuilder::new().us_units();
            builder.mash_steps = vec![infusion_step(152.0, 60, Some(4.0)), sparge_step()];
            let result = builder.calculate();
            assert!((result.strike_water.volume.value - 4.0).abs() < 0.05);
        }

        #[test]
        fn clamps_strike_volume_to_total_water_needed() {
            let mut builder = TypicalInputBuilder::new().us_units();
            builder.mash_steps = vec![infusion_step(152.0, 60, Some(20.0)), sparge_step()];
            let result = builder.calculate();
            assert!(result.strike_water.volume.value <= result.total_water_needed.value + 1e-5);
            assert!(result.sparge_water.as_ref().unwrap().volume.value.abs() < 1e-5);
        }
    }

    mod no_sparge_biab {
        use super::*;

        #[test]
        fn sparge_water_is_none_when_no_sparge_step() {
            let mut builder = TypicalInputBuilder::new().us_units();
            builder.mash_steps = vec![infusion_step(152.0, 60, None)];
            let result = builder.calculate();
            assert!(result.sparge_water.is_none());
        }

        #[test]
        fn all_water_goes_to_strike_when_no_sparge_step() {
            let mut builder = TypicalInputBuilder::new().us_units();
            builder.mash_steps = vec![infusion_step(152.0, 60, None)];
            let result = builder.calculate();
            assert!(
                (result.strike_water.volume.value - result.total_water_needed.value).abs() < 1e-5
            );
        }

        #[test]
        fn total_water_is_the_same_regardless_of_sparge_presence() {
            let with_sparge = TypicalInputBuilder::new().us_units().calculate();
            let mut no_sparge = TypicalInputBuilder::new().us_units();
            no_sparge.mash_steps = vec![infusion_step(152.0, 60, None)];
            let no_sparge_result = no_sparge.calculate();
            assert!(
                (with_sparge.total_water_needed.value - no_sparge_result.total_water_needed.value)
                    .abs()
                    < 1e-5
            );
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn no_grains_extract_brew() {
            let mut builder = TypicalInputBuilder::new().us_units();
            builder.grain_bill = vec![grain(5.0, FermentableAdditionTypeType::Extract)];
            let result = builder.calculate();
            // Mash loss is only tun deadspace (no grain absorption)
            let mash_loss =
                result.stages.mash.volume_in.value - result.stages.mash.volume_out.value;
            assert!((mash_loss - MASH_TUN_DEADSPACE_LOSS_GAL).abs() < 1e-5);
            assert!(result.strike_water.volume.value.abs() < 1e-5);
            assert!(
                (result.sparge_water.as_ref().unwrap().volume.value
                    - result.total_water_needed.value)
                    .abs()
                    < 1e-5
            );
        }

        #[test]
        fn empty_mash_steps_no_sparge_all_water_to_strike() {
            let mut builder = TypicalInputBuilder::new().us_units();
            builder.mash_steps = vec![];
            let result = builder.calculate();
            assert!(result.total_water_needed.value > 0.0);
            assert!(result.sparge_water.is_none());
            assert!(
                (result.strike_water.volume.value - result.total_water_needed.value).abs() < 1e-5
            );
        }

        #[test]
        fn zero_boil_time() {
            let with_boil = TypicalInputBuilder::new().us_units().calculate();
            let mut no_boil = TypicalInputBuilder::new().us_units();
            no_boil.boil_time = TimeType { value: 0, unit: TimeUnitType::Min };
            let no_boil_result = no_boil.calculate();

            assert!(no_boil_result.total_water_needed.value < with_boil.total_water_needed.value);
            // With zero boil, kettle loss is only trub (0.5), no evaporation
            let kettle_loss = no_boil_result.stages.kettle.volume_in.value
                - no_boil_result.stages.kettle.volume_out.value;
            assert!((kettle_loss - KETTLE_TRUB_LOSS_GAL).abs() < 0.05);
        }
    }

    mod stage_calculation {
        use super::*;

        fn default_stages() -> Vec<WaterStage> {
            build_stages(None, 10.0)
        }

        #[test]
        fn calculate_from_stages_matches_manual_backward_calculation() {
            // target=5.0, boil=60min, default stages with 10lb grain
            let result = calculate_water_from_stages(5.0, 60.0, &default_stages());
            // packaging: 5.0+0.1=5.1
            // fermenter: 5.1+0.5=5.6
            // boil: 5.6+1.0+0.5=7.1
            // mash: 7.1+1.25+0.25=8.6
            assert!((result.total_water_needed - 8.6).abs() < 0.01);
        }

        #[test]
        fn packaging_stage_appears_in_result() {
            let result = calculate_water_from_stages(5.0, 60.0, &default_stages());
            let packaging = result.stages.iter().find(|s| s.id == "packaging");
            assert!(packaging.is_some());
            let pkg = packaging.unwrap();
            assert!(matches!(pkg.stage_type, WaterStageType::Packaging));
            assert!((pkg.volume_out - 5.0).abs() < 0.01);
            assert!((pkg.volume_in - 5.1).abs() < 0.01);
        }

        #[test]
        fn stage_volumes_chain_through_all_stages() {
            let result = calculate_water_from_stages(5.0, 60.0, &default_stages());
            // Skip source (index 0), check that each stage's volume_out == next stage's volume_in
            let stages = &result.stages;
            for i in 1..stages.len() - 1 {
                assert!(
                    (stages[i].volume_out - stages[i + 1].volume_in).abs() < 1e-10,
                    "stage {} volume_out ({}) != stage {} volume_in ({})",
                    stages[i].id, stages[i].volume_out,
                    stages[i + 1].id, stages[i + 1].volume_in,
                );
            }
        }

        #[test]
        fn source_tank_has_correct_total_and_is_source() {
            let result = calculate_water_from_stages(5.0, 60.0, &default_stages());
            let source = &result.stages[0];
            assert!(source.is_source);
            assert!(matches!(source.stage_type, WaterStageType::Source));
            assert!((source.volume_in - result.total_water_needed).abs() < 1e-10);
            assert!((source.volume_out - result.total_water_needed).abs() < 1e-10);
            assert!((source.total_loss - 0.0).abs() < 1e-10);
        }

        #[test]
        fn calculated_stages_in_high_level_api() {
            let result = TypicalInputBuilder::new().us_units().calculate();
            // Should have 5 stages: source + mash + kettle + fermenter + packaging
            assert_eq!(result.calculated_stages.len(), 5);
            assert!(result.calculated_stages[0].is_source);
            assert!(matches!(result.calculated_stages[1].stage_type, WaterStageType::Mash));
            assert!(matches!(result.calculated_stages[2].stage_type, WaterStageType::Boil));
            assert!(matches!(result.calculated_stages[3].stage_type, WaterStageType::Fermenter));
            assert!(matches!(result.calculated_stages[4].stage_type, WaterStageType::Packaging));
        }

        #[test]
        fn packaging_output_equals_target_batch_size() {
            let result = TypicalInputBuilder::new().us_units().calculate();
            let packaging = result.calculated_stages.iter()
                .find(|s| matches!(s.stage_type, WaterStageType::Packaging))
                .unwrap();
            assert!((packaging.volume_out - 5.0).abs() < 0.05);
        }

        #[test]
        fn calculated_stages_use_output_units() {
            let mut builder = TypicalInputBuilder::new();
            builder.units = Some(WaterCalculatorOptions {
                volume_unit: VolumeUnitType::L,
                temperature_unit: TemperatureUnitType::C,
            });
            let result = builder.calculate();
            // Source volume should be in liters (total_water_needed)
            let source = &result.calculated_stages[0];
            assert!((source.volume_in - result.total_water_needed.value).abs() < 0.1);
        }
    }
}
