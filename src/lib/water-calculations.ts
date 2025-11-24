// --- Types ---

export type LossType = 'flat' | 'rate';
export type TankShape = 'rect' | 'dome' | 'chimney' | 'conical' | 'keg';

export interface Loss {
  id: string;
  label: string;
  value: number;
  type: LossType;
  unit: string;
}

export interface Stage {
  id: string;
  label: string;
  shape: TankShape;
  losses: Loss[];
}

export interface CalculatedStage extends Stage {
  volumeIn: number;
  volumeOut: number;
  totalLoss: number;
  isSource?: boolean;
}

export interface CalculationResult {
  totalWaterNeeded: number;
  stages: CalculatedStage[];
}

/**
 * Calculate water volumes for brewing process
 */
export function calculateWaterVolumes(
  targetVolume: number,
  boilTime: number,
  stages: Stage[]
): CalculationResult {
  let currentVol = targetVolume;
  const processedStages: CalculatedStage[] = [];

  // Work backwards through stages
  [...stages].reverse().forEach(stage => {
    let stageLossTotal = 0;
    stage.losses.forEach(loss => {
      let val = loss.value;
      if (loss.type === 'rate') {
        val = val * (boilTime / 60);
      }
      stageLossTotal += val;
    });

    const volIn = currentVol + stageLossTotal;
    processedStages.push({
      ...stage,
      volumeIn: volIn,
      volumeOut: currentVol,
      totalLoss: stageLossTotal,
    });
    currentVol = volIn;
  });

  const orderedStages = processedStages.reverse();

  const sourceTank: CalculatedStage = {
    id: 'source',
    label: 'Strike Water',
    shape: 'rect',
    losses: [],
    volumeIn: currentVol,
    volumeOut: currentVol,
    totalLoss: 0,
    isSource: true
  };

  return {
    totalWaterNeeded: currentVol,
    stages: [sourceTank, ...orderedStages]
  };
}

/**
 * Create default brewing stages
 */
export function getDefaultStages(): Stage[] {
  return [
    {
      id: "mash",
      label: "Mash Tun",
      shape: "dome",
      losses: [
        { id: "grainAbs", label: "Grain Absorption", value: 1.25, type: "flat", unit: "gal" },
        { id: "tunDead", label: "Tun Deadspace", value: 0.25, type: "flat", unit: "gal" }
      ]
    },
    {
      id: "kettle",
      label: "Boil Kettle",
      shape: "chimney",
      losses: [
        { id: "boilOff", label: "Boil Off Rate", value: 1.0, type: "rate", unit: "gal/hr" },
        { id: "trub", label: "Trub / Hop Loss", value: 0.5, type: "flat", unit: "gal" }
      ]
    },
    {
      id: "fermenter",
      label: "Fermenter",
      shape: "conical",
      losses: [
        { id: "trub_ferm", label: "Yeast/Cake Loss", value: 0.5, type: "flat", unit: "gal" }
      ]
    },
    {
      id: "packaging",
      label: "Kegging",
      shape: "keg",
      losses: [
        { id: "lines", label: "Transfer Loss", value: 0.1, type: "flat", unit: "gal" }
      ]
    }
  ];
}

/**
 * Map equipment form to tank shape for visualization
 */
function getShapeForEquipmentForm(form: string): TankShape {
  const formLower = form.toLowerCase();
  if (formLower.includes('mash')) return 'dome';
  if (formLower.includes('kettle') || formLower.includes('brew')) return 'chimney';
  if (formLower.includes('fermenter')) return 'conical';
  if (formLower.includes('keg') || formLower.includes('packaging')) return 'keg';
  return 'rect';
}

/**
 * Convert BeerJSON equipment to water calculator stages
 */
export function equipmentToStages(equipment: any, grainWeight: number = 0): Stage[] {
  if (!equipment?.equipment_items) {
    return getDefaultStages();
  }

  return equipment.equipment_items.map((item: any, index: number) => {
    const losses: Loss[] = [];
    const id = item.name.toLowerCase().replace(/\s+/g, '_');

    // Handle grain absorption for Mash Tun
    if (item.form === 'Mash Tun' && item.grain_absorption_rate) {
      // Convert grain absorption rate to gallons
      // grain_absorption_rate is typically in l/kg or qt/lb
      let absorptionInGal = 0;
      if (grainWeight > 0) {
        const rate = item.grain_absorption_rate.value;
        const unit = item.grain_absorption_rate.unit;

        // Convert to gallons based on unit
        if (unit === 'l/kg') {
          // Assume grain weight in lbs, convert to kg, then to gallons
          absorptionInGal = (grainWeight * 0.453592 * rate) * 0.264172;
        } else if (unit === 'qt/lb') {
          absorptionInGal = (grainWeight * rate) * 0.25;
        }
      } else {
        // Default grain absorption if no grain weight provided
        absorptionInGal = 1.25;
      }

      losses.push({
        id: `${id}_grain_abs`,
        label: 'Grain Absorption',
        value: absorptionInGal,
        type: 'flat',
        unit: 'gal'
      });
    }

    // Handle boil rate for Brew Kettle
    if (item.form === 'Brew Kettle' && item.boil_rate_per_hour) {
      losses.push({
        id: `${id}_boil_off`,
        label: 'Boil Off Rate',
        value: item.boil_rate_per_hour.value,
        type: 'rate',
        unit: 'gal/hr'
      });
    }

    // Add equipment loss
    if (item.loss && item.loss.value > 0) {
      const lossLabel = item.form === 'Brew Kettle' ? 'Trub / Hop Loss' :
                        item.form === 'Fermenter' ? 'Yeast/Cake Loss' :
                        item.form === 'Packaging Vessel' ? 'Transfer Loss' :
                        'Equipment Loss';

      losses.push({
        id: `${id}_loss`,
        label: lossLabel,
        value: item.loss.value,
        type: 'flat',
        unit: item.loss.unit || 'gal'
      });
    }

    return {
      id,
      label: item.name,
      shape: getShapeForEquipmentForm(item.form),
      losses
    };
  });
}
