import { useMemo } from 'react';
import { WaterFlowVisualization } from './water-flow-visualization';
import { calculate_water } from 'brewdio-wasm';
import type { CalculatedWaterStage, WaterCalculatorResult } from 'brewdio-wasm';
import type { BatchDocument } from '@/lib/db/batches';

interface BatchWaterCalculatorProps {
  batch: BatchDocument;
  beerColor?: string;
}

export function BatchWaterCalculator({ batch, beerColor = "#F59E0B" }: BatchWaterCalculatorProps) {
  const result = useMemo(() => {
    const input = {
      target_batch_size: batch.recipe.batch_size,
      boil_time: batch.recipe.boil?.boil_time ?? { value: 60, unit: "min" },
      grain_bill: batch.recipe.ingredients?.fermentable_additions ?? [],
      mash_steps: batch.recipe.mash?.mash_steps ?? [],
      grain_temperature: { value: 20, unit: "C" },
      equipment: batch.equipment ?? null,
      units: { volume_unit: "gal", temperature_unit: "F" },
    };

    return calculate_water(input as any) as WaterCalculatorResult;
  }, [batch]);

  const stages: CalculatedWaterStage[] = result.calculatedStages;
  const strikeWater = result.strikeWater;
  const spargeWater = result.spargeWater;

  return (
    <div className="border rounded-lg p-4">
      <div className="mb-4">
        <h3 className="text-xl font-semibold mb-2">Water Requirements</h3>
        <div className="flex items-baseline gap-2">
          <span className="text-3xl font-bold text-blue-500">
            {result.totalWaterNeeded.value.toFixed(2)}
          </span>
          <span className="text-sm text-muted-foreground">
            {result.totalWaterNeeded.unit} total strike water
          </span>
        </div>
        <p className="text-xs text-muted-foreground mt-1">
          Based on {batch.recipe.batch_size.value} {batch.recipe.batch_size.unit} batch size
          and {batch.recipe.boil?.boil_time?.value ?? 60} {batch.recipe.boil?.boil_time?.unit ?? "min"} boil time
        </p>
      </div>

      {/* Strike & Sparge Info */}
      <div className="grid grid-cols-2 gap-4 mb-4">
        <div className="bg-muted rounded-lg p-3">
          <h4 className="text-sm font-semibold mb-1">Strike Water</h4>
          <div className="text-lg font-bold">{strikeWater.volume.value.toFixed(2)} {strikeWater.volume.unit}</div>
          <div className="text-xs text-muted-foreground">
            Heat to {strikeWater.temperature.value.toFixed(1)}&deg;{strikeWater.temperature.unit}
          </div>
        </div>
        {spargeWater && (
          <div className="bg-muted rounded-lg p-3">
            <h4 className="text-sm font-semibold mb-1">Sparge Water</h4>
            <div className="text-lg font-bold">{spargeWater.volume.value.toFixed(2)} {spargeWater.volume.unit}</div>
            <div className="text-xs text-muted-foreground">
              Heat to {spargeWater.temperature.value.toFixed(1)}&deg;{spargeWater.temperature.unit}
            </div>
          </div>
        )}
      </div>

      {/* Visualization */}
      <div className="bg-slate-900 rounded-xl p-4 flex items-center justify-center min-h-[450px]">
        <WaterFlowVisualization stages={stages} beerColor={beerColor} />
      </div>

      {/* Stage Breakdown (Read-only) */}
      <div className="mt-4 grid grid-cols-2 md:grid-cols-4 gap-4">
        {stages.map((stage) => {
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
