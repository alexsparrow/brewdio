import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Pencil, Check, X } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { cn } from "@/lib/utils";

const RECIPE_TYPES = [
  { value: "all grain", label: "All Grain" },
  { value: "partial mash", label: "Partial Mash" },
  { value: "extract", label: "Extract" },
  { value: "wine", label: "Wine" },
  { value: "mead", label: "Mead" },
  { value: "cider", label: "Cider" },
  { value: "kombucha", label: "Kombucha" },
  { value: "soda", label: "Soda" },
  { value: "other", label: "Other" },
];

function formatTypeName(type: string): string {
  const found = RECIPE_TYPES.find((t) => t.value === type);
  if (found) return found.label;
  return type
    .split(" ")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

interface EditableTypeSelectorProps {
  type: string;
  onSave: (newType: string) => void | Promise<void>;
  className?: string;
}

export function EditableTypeSelector({
  type,
  onSave,
  className,
}: EditableTypeSelectorProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [selectedType, setSelectedType] = useState(type);
  const [isSaving, setIsSaving] = useState(false);

  const handleSave = async () => {
    if (selectedType === type) {
      setIsEditing(false);
      return;
    }

    setIsSaving(true);
    try {
      await onSave(selectedType);
      setIsEditing(false);
    } catch (error) {
      console.error("Failed to save type:", error);
      setSelectedType(type);
    } finally {
      setIsSaving(false);
    }
  };

  const handleCancel = () => {
    setSelectedType(type);
    setIsEditing(false);
  };

  if (isEditing) {
    return (
      <div className={cn("flex items-center gap-2", className)}>
        <Select value={selectedType} onValueChange={setSelectedType}>
          <SelectTrigger className="h-7 text-xs w-[140px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {RECIPE_TYPES.map((opt) => (
              <SelectItem key={opt.value} value={opt.value}>
                {opt.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Button
          type="button"
          size="sm"
          variant="ghost"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            handleSave();
          }}
          disabled={isSaving}
          className="h-6 w-6 p-0 shrink-0 text-green-600 hover:text-green-700 hover:bg-green-50"
        >
          <Check className="h-3 w-3" />
        </Button>
        <Button
          type="button"
          size="sm"
          variant="ghost"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            handleCancel();
          }}
          disabled={isSaving}
          className="h-6 w-6 p-0 shrink-0 text-muted-foreground hover:text-destructive hover:bg-destructive/10"
        >
          <X className="h-3 w-3" />
        </Button>
      </div>
    );
  }

  return (
    <div className={cn("flex items-center gap-1 group", className)}>
      <p className="text-xs text-muted-foreground/70">
        {formatTypeName(type)}
      </p>
      <Button
        size="sm"
        variant="ghost"
        onClick={() => setIsEditing(true)}
        className="h-5 w-5 p-0 opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-primary"
      >
        <Pencil className="h-2.5 w-2.5" />
      </Button>
    </div>
  );
}
