import { useState, useEffect } from "react";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Check, X, Pencil } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { TemperatureType } from "@beerjson/beerjson";

interface InlineTemperatureProps {
  label: string;
  value: TemperatureType | undefined;
  optional?: boolean;
  isEditing: boolean;
  onEdit: () => void;
  onSave: (value: TemperatureType) => void;
  onCancel: () => void;
}

export function InlineTemperature({
  label,
  value,
  optional = false,
  isEditing,
  onEdit,
  onSave,
  onCancel,
}: InlineTemperatureProps) {
  const [editValue, setEditValue] = useState<number>(value?.value || 0);
  const [editUnit, setEditUnit] = useState<"C" | "F">(value?.unit || "F");

  useEffect(() => {
    if (value) {
      setEditValue(value.value);
      setEditUnit(value.unit);
    }
  }, [value]);

  const handleSave = () => {
    onSave({ value: editValue, unit: editUnit });
  };

  if (isEditing) {
    return (
      <div className="flex items-center gap-2">
        <Label className="min-w-32">{label}:</Label>
        <Input
          type="number"
          value={editValue}
          onChange={(e) => setEditValue(parseFloat(e.target.value) || 0)}
          className="flex-1"
        />
        <Select value={editUnit} onValueChange={(val) => setEditUnit(val as "C" | "F")}>
          <SelectTrigger className="w-20">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="F">°F</SelectItem>
            <SelectItem value="C">°C</SelectItem>
          </SelectContent>
        </Select>
        <Button size="sm" onClick={handleSave}>
          <Check />
        </Button>
        <Button size="sm" variant="ghost" onClick={onCancel}>
          <X />
        </Button>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2 group">
      <Label className="min-w-32">{label}:</Label>
      <span className="flex-1">
        {value
          ? `${value.value}°${value.unit}`
          : optional
          ? "—"
          : "Not set"}
      </span>
      <Button
        size="sm"
        variant="ghost"
        onClick={onEdit}
        className="opacity-0 group-hover:opacity-100"
      >
        <Pencil className="h-4 w-4" />
      </Button>
    </div>
  );
}
