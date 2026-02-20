import { useState } from "react";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Trash2 } from "lucide-react";
import { InlineField } from "@/components/inline-field";
import { InlineSelect } from "@/components/inline-select";
import { InlineTemperature } from "@/components/inline-temperature";
import { InlineTime } from "@/components/inline-time";
import { InlineVolume } from "@/components/inline-volume";
import type { MashStepType } from "@beerjson/beerjson";

interface MashStepCardProps {
  step: MashStepType;
  index: number;
  isEditing: boolean;
  onUpdate: (index: number, updates: Partial<MashStepType>) => void;
  onRemove?: (index: number) => void;
}

const MASH_STEP_TYPES = [
  { value: "infusion", label: "Infusion" },
  { value: "temperature", label: "Temperature" },
  { value: "decoction", label: "Decoction" },
  { value: "souring mash", label: "Souring Mash" },
  { value: "souring wort", label: "Souring Wort" },
  { value: "drain mash tun", label: "Drain Mash Tun" },
  { value: "sparge", label: "Sparge" },
];

export function MashStepCard({
  step,
  index,
  isEditing,
  onUpdate,
  onRemove,
}: MashStepCardProps) {
  const [editingField, setEditingField] = useState<string | null>(null);

  const handleFieldUpdate = (field: string, value: any) => {
    onUpdate(index, { [field]: value });
    setEditingField(null);
  };

  return (
    <Card className="p-4">
      <div className="flex justify-between items-start gap-4">
        <div className="flex-1 space-y-2">
          {/* Name - inline editable */}
          <InlineField
            label="Step"
            value={step.name}
            isEditing={isEditing && editingField === "name"}
            onEdit={() => setEditingField("name")}
            onSave={(val) => handleFieldUpdate("name", val)}
            onCancel={() => setEditingField(null)}
          />

          {/* Type - dropdown */}
          <InlineSelect
            label="Type"
            value={step.type}
            options={MASH_STEP_TYPES}
            isEditing={isEditing && editingField === "type"}
            onEdit={() => setEditingField("type")}
            onSave={(val) => handleFieldUpdate("type", val)}
            onCancel={() => setEditingField(null)}
          />

          {/* Temperature - with unit */}
          <InlineTemperature
            label="Temperature"
            value={step.step_temperature}
            isEditing={isEditing && editingField === "temperature"}
            onEdit={() => setEditingField("temperature")}
            onSave={(val) => handleFieldUpdate("step_temperature", val)}
            onCancel={() => setEditingField(null)}
          />

          {/* Time - with unit */}
          <InlineTime
            label="Duration"
            value={step.step_time}
            isEditing={isEditing && editingField === "time"}
            onEdit={() => setEditingField("time")}
            onSave={(val) => handleFieldUpdate("step_time", val)}
            onCancel={() => setEditingField(null)}
          />

          {/* Optional fields - show only if present or in edit mode */}
          {(step.ramp_time || isEditing) && (
            <InlineTime
              label="Ramp Time"
              value={step.ramp_time}
              optional
              isEditing={isEditing && editingField === "ramp_time"}
              onEdit={() => setEditingField("ramp_time")}
              onSave={(val) => handleFieldUpdate("ramp_time", val)}
              onCancel={() => setEditingField(null)}
            />
          )}

          {(step.end_temperature || isEditing) && (
            <InlineTemperature
              label="End Temperature"
              value={step.end_temperature}
              optional
              isEditing={isEditing && editingField === "end_temperature"}
              onEdit={() => setEditingField("end_temperature")}
              onSave={(val) => handleFieldUpdate("end_temperature", val)}
              onCancel={() => setEditingField(null)}
            />
          )}

          {(step.infuse_temperature || isEditing) && (
            <InlineTemperature
              label="Infuse Temperature"
              value={step.infuse_temperature}
              optional
              isEditing={isEditing && editingField === "infuse_temperature"}
              onEdit={() => setEditingField("infuse_temperature")}
              onSave={(val) => handleFieldUpdate("infuse_temperature", val)}
              onCancel={() => setEditingField(null)}
            />
          )}

          {(step.amount || isEditing) && (
            <InlineVolume
              label="Amount"
              value={step.amount}
              optional
              isEditing={isEditing && editingField === "amount"}
              onEdit={() => setEditingField("amount")}
              onSave={(val) => handleFieldUpdate("amount", val)}
              onCancel={() => setEditingField(null)}
            />
          )}

          {/* Description */}
          {(step.description || isEditing) && (
            <InlineField
              label="Description"
              value={step.description}
              optional
              multiline
              isEditing={isEditing && editingField === "description"}
              onEdit={() => setEditingField("description")}
              onSave={(val) => handleFieldUpdate("description", val)}
              onCancel={() => setEditingField(null)}
            />
          )}
        </div>

        {isEditing && onRemove && (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onRemove(index)}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        )}
      </div>
    </Card>
  );
}
