import { useState } from "react";
import { useForm } from "@tanstack/react-form";
import { useLiveQuery } from "@tanstack/react-db";
import { settingsCollection } from "@/db";
import { useRecipeEdit } from "@/contexts/recipe-edit-context";
import type { FermentableAdditionType, FermentableType, MassUnitType } from "@beerjson/beerjson";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Combobox } from "@/components/ui/combobox";
import { Plus } from "lucide-react";
import fermentables from "@/data/fermentables.json";

interface AddFermentableDialogProps {
  existingFermentable?: FermentableAdditionType;
  index?: number;
  trigger?: React.ReactNode;
}

const massUnits: MassUnitType[] = ["mg", "g", "kg", "lb", "oz"];

export function AddFermentableDialog({
  existingFermentable,
  index,
  trigger,
}: AddFermentableDialogProps) {
  const [open, setOpen] = useState(false);
  const { id: recipeId, collection } = useRecipeEdit();
  const { data: settingsData } = useLiveQuery(settingsCollection);
  const settings = settingsData?.find((s) => s.id === "user-settings");
  const isEditing = existingFermentable !== undefined && index !== undefined;

  const form = useForm({
    defaultValues: {
      fermentableName: existingFermentable?.name || "",
      amount: existingFermentable?.amount.value.toString() || "1.0",
      unit: (existingFermentable?.amount.unit || settings?.defaultMassUnit || "lb") as MassUnitType,
    },
    onSubmit: async ({ value }) => {
      // Find the selected fermentable from the fermentables data
      const selectedFermentable = fermentables.find((f) => f.name === value.fermentableName);

      if (!selectedFermentable) {
        return;
      }

      // Create the fermentable addition
      const fermentableAddition: FermentableAdditionType = {
        name: selectedFermentable.name,
        type: selectedFermentable.type as FermentableAdditionType["type"],
        producer: selectedFermentable.producer,
        yield: selectedFermentable.yield,
        color: selectedFermentable.color,
        amount: {
          value: parseFloat(value.amount),
          unit: value.unit,
        },
      };

      // Update in the database
      await collection.update(recipeId, (draft) => {
        if (isEditing && index !== undefined) {
          // Edit existing fermentable
          draft.recipe.ingredients.fermentable_additions =
            draft.recipe.ingredients.fermentable_additions?.map((item, idx) =>
              idx === index ? fermentableAddition : item
            ) || [];
        } else {
          // Add new fermentable
          draft.recipe.ingredients.fermentable_additions = [
            ...(draft.recipe.ingredients.fermentable_additions || []),
            fermentableAddition,
          ];
        }
        draft.updatedAt = Date.now();
      });

      // Reset form and close dialog
      form.reset();
      setOpen(false);
    },
  });

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        {trigger || (
          <Button size="sm" className="gap-2">
            <Plus className="h-4 w-4" />
            Add Fermentable
          </Button>
        )}
      </DialogTrigger>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>{isEditing ? "Edit Fermentable" : "Add Fermentable"}</DialogTitle>
          <DialogDescription>
            {isEditing
              ? "Update the fermentable details."
              : "Select a fermentable and specify the amount to add to your recipe."}
          </DialogDescription>
        </DialogHeader>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            e.stopPropagation();
            form.handleSubmit();
          }}
          className="space-y-4 py-4"
        >
          <form.Field
            name="fermentableName"
            validators={{
              onChange: ({ value }) =>
                !value ? "Fermentable selection is required" : undefined,
            }}
          >
            {(field) => (
              <div className="space-y-2">
                <Label htmlFor={field.name}>Fermentable</Label>
                <Combobox
                  options={fermentables.map((f) => ({
                    value: f.name,
                    label: `${f.name} (${f.type})`,
                  }))}
                  value={field.state.value}
                  onValueChange={field.handleChange}
                  placeholder="Select a fermentable"
                  searchPlaceholder="Search fermentables..."
                  emptyText="No fermentable found."
                />
                {field.state.meta.errors && (
                  <p className="text-sm text-destructive">
                    {field.state.meta.errors.join(", ")}
                  </p>
                )}
              </div>
            )}
          </form.Field>

          <div className="grid grid-cols-2 gap-4">
            <form.Field
              name="amount"
              validators={{
                onChange: ({ value }) => {
                  const num = parseFloat(value);
                  if (isNaN(num) || num <= 0) {
                    return "Amount must be a positive number";
                  }
                  return undefined;
                },
              }}
            >
              {(field) => (
                <div className="space-y-2">
                  <Label htmlFor={field.name}>Amount</Label>
                  <Input
                    id={field.name}
                    type="number"
                    step="0.01"
                    min="0.01"
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onChange={(e) => field.handleChange(e.target.value)}
                    placeholder="1.0"
                  />
                  {field.state.meta.errors && (
                    <p className="text-sm text-destructive">
                      {field.state.meta.errors.join(", ")}
                    </p>
                  )}
                </div>
              )}
            </form.Field>

            <form.Field name="unit">
              {(field) => (
                <div className="space-y-2">
                  <Label htmlFor={field.name}>Unit</Label>
                  <Select
                    value={field.state.value}
                    onValueChange={field.handleChange}
                  >
                    <SelectTrigger id={field.name}>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {massUnits.map((unit) => (
                        <SelectItem key={unit} value={unit}>
                          {unit}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              )}
            </form.Field>
          </div>

          <div className="flex justify-end gap-3 pt-4">
            <Button
              type="button"
              variant="outline"
              onClick={() => setOpen(false)}
            >
              Cancel
            </Button>
            <form.Subscribe
              selector={(state) => [state.canSubmit, state.isSubmitting]}
            >
              {([canSubmit, isSubmitting]) => (
                <Button type="submit" disabled={!canSubmit || isSubmitting}>
                  {isSubmitting
                    ? isEditing
                      ? "Updating..."
                      : "Adding..."
                    : isEditing
                    ? "Update Fermentable"
                    : "Add Fermentable"}
                </Button>
              )}
            </form.Subscribe>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
