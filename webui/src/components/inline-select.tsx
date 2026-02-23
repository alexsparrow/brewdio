import { useState, useEffect } from "react";
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

interface InlineSelectProps {
  label: string;
  value: string | undefined;
  options: Array<{ value: string; label: string }>;
  optional?: boolean;
  isEditing: boolean;
  onEdit: () => void;
  onSave: (value: string) => void;
  onCancel: () => void;
}

export function InlineSelect({
  label,
  value,
  options,
  optional = false,
  isEditing,
  onEdit,
  onSave,
  onCancel,
}: InlineSelectProps) {
  const [editValue, setEditValue] = useState(value || options[0]?.value || "");

  useEffect(() => {
    setEditValue(value || options[0]?.value || "");
  }, [value, options]);

  const handleSave = () => {
    onSave(editValue);
  };

  const displayValue = options.find((opt) => opt.value === value)?.label || value;

  if (isEditing) {
    return (
      <div className="flex items-center gap-2">
        <Label className="min-w-32">{label}:</Label>
        <Select value={editValue} onValueChange={setEditValue}>
          <SelectTrigger className="flex-1">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {options.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
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
      <span className="flex-1">{displayValue || (optional ? "—" : "Not set")}</span>
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
