import { useState, useEffect } from "react";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Check, X, Pencil } from "lucide-react";

interface InlineFieldProps {
  label: string;
  value: string | undefined;
  optional?: boolean;
  multiline?: boolean;
  isEditing: boolean;
  onEdit: () => void;
  onSave: (value: string) => void;
  onCancel: () => void;
}

export function InlineField({
  label,
  value,
  optional = false,
  multiline = false,
  isEditing,
  onEdit,
  onSave,
  onCancel,
}: InlineFieldProps) {
  const [editValue, setEditValue] = useState(value || "");

  useEffect(() => {
    setEditValue(value || "");
  }, [value]);

  if (isEditing) {
    return (
      <div className="flex items-center gap-2">
        <Label className="min-w-32">{label}:</Label>
        {multiline ? (
          <Textarea
            value={editValue}
            onChange={(e) => setEditValue(e.target.value)}
            className="flex-1"
            rows={3}
          />
        ) : (
          <Input
            value={editValue}
            onChange={(e) => setEditValue(e.target.value)}
            className="flex-1"
          />
        )}
        <Button size="sm" onClick={() => onSave(editValue)}>
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
      <span className="flex-1">{value || (optional ? "—" : "Not set")}</span>
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
