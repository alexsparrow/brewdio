import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Combobox } from '@/components/ui/combobox';
import { Pencil, Check, X } from 'lucide-react';
import { getAvailableStyles } from '@/lib/actions/recipes';
import { cn } from '@/lib/utils';

interface EditableStyleSelectorProps {
  styleName: string;
  styleCategory?: string;
  onSave: (newStyleName: string) => void | Promise<void>;
  className?: string;
}

export function EditableStyleSelector({
  styleName,
  styleCategory,
  onSave,
  className,
}: EditableStyleSelectorProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [selectedStyle, setSelectedStyle] = useState(styleName);
  const [isSaving, setIsSaving] = useState(false);
  const availableStyles = getAvailableStyles();

  useEffect(() => {
    setSelectedStyle(styleName);
  }, [styleName]);

  const handleSave = async () => {
    if (!selectedStyle) {
      setSelectedStyle(styleName);
      setIsEditing(false);
      return;
    }

    if (selectedStyle === styleName) {
      setIsEditing(false);
      return;
    }

    setIsSaving(true);
    try {
      await onSave(selectedStyle);
      setIsEditing(false);
    } catch (error) {
      console.error('Failed to save:', error);
      setSelectedStyle(styleName);
    } finally {
      setIsSaving(false);
    }
  };

  const handleCancel = () => {
    setSelectedStyle(styleName);
    setIsEditing(false);
  };

  if (isEditing) {
    return (
      <div className={cn('flex items-center gap-2 flex-wrap', className)}>
        <div className="min-w-[300px]">
          <Combobox
            options={availableStyles.map((s) => ({
              value: s.name,
              label: `${s.name} - ${s.category}`,
            }))}
            value={selectedStyle}
            onValueChange={setSelectedStyle}
            placeholder="Select a beer style"
            searchPlaceholder="Search styles..."
            emptyText="No style found."
            disabled={isSaving}
          />
        </div>
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
          className="h-8 w-8 p-0 shrink-0 text-green-600 hover:text-green-700 hover:bg-green-50"
        >
          <Check className="h-4 w-4" />
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
          className="h-8 w-8 p-0 shrink-0 text-muted-foreground hover:text-destructive hover:bg-destructive/10"
        >
          <X className="h-4 w-4" />
        </Button>
      </div>
    );
  }

  return (
    <div className={cn('flex items-center gap-2 group', className)}>
      <p className="text-muted-foreground">
        {styleName}
        {styleCategory && ` • ${styleCategory}`}
      </p>
      <Button
        size="sm"
        variant="ghost"
        onClick={() => setIsEditing(true)}
        className="h-6 w-6 p-0 opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-primary"
      >
        <Pencil className="h-3 w-3" />
      </Button>
    </div>
  );
}
