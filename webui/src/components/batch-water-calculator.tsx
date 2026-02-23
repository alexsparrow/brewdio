import React, { useMemo } from 'react';
import { WaterFlowVisualization } from './water-flow-visualization';
import { calculateWaterVolumes, equipmentToStages } from '@/lib/water-calculations';
import type { BatchDocument } from '@/db';

interface BatchWaterCalculatorProps {
  batch: BatchDocument;
  beerColor?: string;
}

export function BatchWaterCalculator({ batch, beerColor = "#F59E0B" }: BatchWaterCalculatorProps) {
  const data = useMemo(() => {
    // Extract batch parameters
    const targetVolume = batch.recipe.batch_size.value; // in gallons
    const boilTime = batch.recipe.boil_time.value; // in minutes

    // Calculate total grain weight from fermentables
    const grainWeight = batch.recipe.ingredients?.fermentable_additions?.reduce(
      (total, fermentable) => {
        const weight = fermentable.amount.value;
        const unit = fermentable.amount.unit;

        // Convert to lbs if needed
        let weightInLbs = weight;
        if (unit === 'kg') {
          weightInLbs = weight * 2.20462;
        } else if (unit === 'g') {
          weightInLbs = weight * 0.00220462;
        }

        return total + weightInLbs;
      },
      0
    ) || 0;

    // Convert equipment to stages using grain weight for absorption calculation
    const stages = equipmentToStages(batch.equipment, grainWeight);

    // Calculate water volumes
    return calculateWaterVolumes(targetVolume, boilTime, stages);
  }, [batch]);

  return (
    <div className="border rounded-lg p-4">
      <div className="mb-4">
        <h3 className="text-xl font-semibold mb-2">Water Requirements</h3>
        <div className="flex items-baseline gap-2">
          <span className="text-3xl font-bold text-blue-500">
            {data.totalWaterNeeded.toFixed(2)}
          </span>
          <span className="text-sm text-muted-foreground">gallons total strike water</span>
        </div>
        <p className="text-xs text-muted-foreground mt-1">
          Based on {batch.recipe.batch_size.value} {batch.recipe.batch_size.unit} batch size
          and {batch.recipe.boil_time.value} {batch.recipe.boil_time.unit} boil time
        </p>
      </div>

      {/* Visualization */}
      <div className="bg-slate-900 rounded-xl p-4 flex items-center justify-center min-h-[450px]">
        <WaterFlowVisualization stages={data.stages} beerColor={beerColor} />
      </div>

      {/* Stage Breakdown (Read-only) */}
      <div className="mt-4 grid grid-cols-2 md:grid-cols-4 gap-4">
        {data.stages.map((stage) => {
          if (stage.isSource) return null;

          return (
            <div key={stage.id} className="bg-muted rounded-lg p-3">
              <h4 className="text-sm font-semibold mb-2">{stage.label}</h4>
              <div className="space-y-1 text-xs">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Volume In:</span>
                  <span className="font-mono">{stage.volumeIn.toFixed(2)} gal</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Volume Out:</span>
                  <span className="font-mono">{stage.volumeOut.toFixed(2)} gal</span>
                </div>
                <div className="flex justify-between text-red-500">
                  <span>Loss:</span>
                  <span className="font-mono">-{stage.totalLoss.toFixed(2)} gal</span>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
