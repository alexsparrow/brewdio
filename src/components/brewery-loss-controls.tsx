import React from 'react';
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Plus } from "lucide-react";

// --- Types ---

type LossType = 'flat' | 'rate';
type TankShape = 'rect' | 'dome' | 'chimney' | 'conical' | 'keg';

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
  shape: TankShape;
  volumeIn: number;
  volumeOut: number;
  totalLoss: number;
  isSource?: boolean;
}

interface BreweryLossControlsProps {
  stage: CalculatedStage | null;
  stageIndex: number;
  onLossChange: (stageIndex: number, lossIndex: number, value: number) => void;
  onAddCustomLoss: (stageIndex: number) => void;
}

export const BreweryLossControls: React.FC<BreweryLossControlsProps> = ({
  stage,
  stageIndex,
  onLossChange,
  onAddCustomLoss
}) => {
  if (!stage || stage.isSource) {
    return (
      <Card>
        <CardContent className="py-12">
          <div className="text-center text-muted-foreground">
            <p>Click on a stage in the visualization to view and adjust losses.</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="text-base">{stage.label}</CardTitle>
            <CardDescription className="text-xs">
              Total loss: {stage.totalLoss.toFixed(2)} gal
            </CardDescription>
          </div>
          <Button
            onClick={() => onAddCustomLoss(stageIndex)}
            size="sm"
            variant="outline"
            className="h-8"
          >
            <Plus className="h-3 w-3 mr-1" />
            Add Loss
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {stage.losses.map((loss, lIndex) => (
            <div key={loss.id} className="space-y-2">
              <div className="flex justify-between items-center">
                <Label htmlFor={`loss-${stage.id}-${loss.id}`} className="text-sm">
                  {loss.label}
                </Label>
                <span className="text-sm font-mono text-muted-foreground">
                  {loss.value.toFixed(2)} {loss.unit}
                </span>
              </div>
              <Slider
                id={`loss-${stage.id}-${loss.id}`}
                min={0}
                max={2}
                step={0.05}
                value={[loss.value]}
                onValueChange={([value]) => onLossChange(stageIndex, lIndex, value)}
                className="w-full"
              />
            </div>
          ))}
          {stage.losses.length === 0 && (
            <div className="text-sm text-muted-foreground italic">
              No losses configured.
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
};
