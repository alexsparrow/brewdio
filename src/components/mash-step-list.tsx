import { Button } from "@/components/ui/button";
import { Plus } from "lucide-react";
import { MashStepCard } from "@/components/mash-step-card";
import type { MashStepType } from "@beerjson/beerjson";

interface MashStepListProps {
  steps: MashStepType[];
  isEditing: boolean;
  onUpdateStep: (index: number, updates: Partial<MashStepType>) => void;
  onAddStep?: () => void;
  onRemoveStep?: (index: number) => void;
}

export function MashStepList({
  steps,
  isEditing,
  onUpdateStep,
  onAddStep,
  onRemoveStep,
}: MashStepListProps) {
  return (
    <div className="space-y-3">
      {steps.map((step, idx) => (
        <MashStepCard
          key={idx}
          step={step}
          index={idx}
          isEditing={isEditing}
          onUpdate={onUpdateStep}
          onRemove={onRemoveStep}
        />
      ))}
      {isEditing && onAddStep && (
        <Button onClick={onAddStep} variant="outline" className="w-full">
          <Plus className="h-4 w-4 mr-2" />
          Add Step
        </Button>
      )}
    </div>
  );
}
