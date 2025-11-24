import { useState } from "react";
import { useForm } from "@tanstack/react-form";
import { useLiveQuery } from "@tanstack/react-db";
import { settingsCollection } from "@/db";
import { useRecipeEdit } from "@/contexts/recipe-edit-context";
import type { HopAdditionType, MassUnitType } from "@beerjson/beerjson";
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
import hops from "@/data/hops.json";

interface AddHopDialogProps {
  existingHop?: HopAdditionType;
  index?: number;
  trigger?: React.ReactNode;
}

const massUnits: MassUnitType[] = ["mg", "g", "kg", "lb", "oz"];

export function AddHopDialog({
  existingHop,
  index,
  trigger,
}: AddHopDialogProps) {
  const [open, setOpen] = useState(false);
  const { id: recipeId, collection } = useRecipeEdit();
  const { data: settingsData } = useLiveQuery(settingsCollection);
  const settings = settingsData?.find((s) => s.id === "user-settings");
  const isEditing = existingHop !== undefined && index !== undefined;

  const form = useForm({
    defaultValues: {
      hopName: existingHop?.name || "",
      amount: existingHop?.amount.value.toString() || "1.0",
      unit: (existingHop?.amount.unit || settings?.defaultMassUnit || "oz") as MassUnitType,
      time: existingHop?.timing?.time?.value.toString() || "60",
    },
    onSubmit: async ({ value }) => {
      // Find the selected hop from the hops data
      const selectedHop = hops.find((h) => h.name === value.hopName);

      if (!selectedHop) {
        return;
      }

      // Create the hop addition
      const hopAddition: HopAdditionType = {
        name: selectedHop.name,
        origin: selectedHop.origin,
        alpha_acid: selectedHop.alpha_acid,
        beta_acid: selectedHop.beta_acid,
        amount: {
          value: parseFloat(value.amount),
          unit: value.unit,
        },
        timing: {
          use: "boil",
          time: {
            value: parseFloat(value.time),
            unit: "min",
          },
        },
      };

      // Update in the database
      await collection.update(recipeId, (draft) => {
        if (isEditing && index !== undefined) {
          // Edit existing hop
          draft.recipe.ingredients.hop_additions =
            draft.recipe.ingredients.hop_additions?.map((item, idx) =>
              idx === index ? hopAddition : item
            ) || [];
        } else {
          // Add new hop
          draft.recipe.ingredients.hop_additions = [
            ...(draft.recipe.ingredients.hop_additions || []),
            hopAddition,
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
            Add Hop
          </Button>
        )}
      </DialogTrigger>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>{isEditing ? "Edit Hop" : "Add Hop"}</DialogTitle>
          <DialogDescription>
            {isEditing
              ? "Update the hop details."
              : "Select a hop variety and specify the amount and boil time to add to your recipe."}
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
            name="hopName"
            validators={{
              onChange: ({ value }) =>
                !value ? "Hop selection is required" : undefined,
            }}
          >
            {(field) => (
              <div className="space-y-2">
                <Label htmlFor={field.name}>Hop Variety</Label>
                <Combobox
                  options={hops.map((h) => ({
                    value: h.name,
                    label: `${h.name} (${h.origin})`,
                  }))}
                  value={field.state.value}
                  onValueChange={field.handleChange}
                  placeholder="Select a hop"
                  searchPlaceholder="Search hops..."
                  emptyText="No hop found."
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

          <form.Field
            name="time"
            validators={{
              onChange: ({ value }) => {
                const num = parseFloat(value);
                if (isNaN(num) || num < 0) {
                  return "Time must be a non-negative number";
                }
                return undefined;
              },
            }}
          >
            {(field) => (
              <div className="space-y-2">
                <Label htmlFor={field.name}>Boil Time (minutes)</Label>
                <Input
                  id={field.name}
                  type="number"
                  step="1"
                  min="0"
                  value={field.state.value}
                  onBlur={field.handleBlur}
                  onChange={(e) => field.handleChange(e.target.value)}
                  placeholder="60"
                />
                {field.state.meta.errors && (
                  <p className="text-sm text-destructive">
                    {field.state.meta.errors.join(", ")}
                  </p>
                )}
              </div>
            )}
          </form.Field>

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
                    ? "Update Hop"
                    : "Add Hop"}
                </Button>
              )}
            </form.Subscribe>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
