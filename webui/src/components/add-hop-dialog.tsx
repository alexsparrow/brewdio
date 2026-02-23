import { useState } from "react";
import { useForm } from "@tanstack/react-form";
import { useLiveQuery } from "@tanstack/react-db";
import { settingsCollection } from "@/db";
import { useRecipeEdit } from "@/contexts/recipe-edit-context";
import type { HopAdditionType, MassUnitType, UseType } from "@beerjson/beerjson";
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
      use: (existingHop?.timing?.use || "add_to_boil") as UseType,
      time: existingHop?.timing?.time?.value.toString() || "60",
      duration: existingHop?.timing?.duration?.value.toString() || "",
    },
    onSubmit: async ({ value }) => {
      // Find the selected hop from the hops data
      const selectedHop = hops.find((h) => h.name === value.hopName);

      if (!selectedHop) {
        return;
      }

      // Create the hop addition with timing based on use type
      const isBoil = value.use === "add_to_boil";
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
          use: value.use,
          // For boil: time is boil duration in minutes
          // For fermentation: time is when to add (days into fermentation)
          time: isBoil ? {
            value: parseFloat(value.time),
            unit: "min",
          } : {
            value: parseFloat(value.time),
            unit: "day",
          },
          // Duration only applies to fermentation (how long it stays)
          // Only include if a value is provided (empty string means unlimited)
          ...(!isBoil && value.duration && {
            duration: {
              value: parseFloat(value.duration),
              unit: "day",
            }
          }),
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
              : "Select a hop variety and specify when and how to add it to your recipe."}
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

          <form.Field name="use">
            {(field) => (
              <div className="space-y-2">
                <Label htmlFor={field.name}>Addition Type</Label>
                <Select
                  value={field.state.value}
                  onValueChange={(newValue) => {
                    field.handleChange(newValue);

                    // Reset time field to sensible defaults when switching types
                    const currentTime = form.getFieldValue("time");
                    const timeNum = currentTime ? parseFloat(currentTime) : 0;

                    // If switching to fermentation and time is a typical boil value, reset to 3 days
                    if (newValue === "add_to_fermentation" && timeNum >= 30) {
                      form.setFieldValue("time", "3");
                    }
                    // If switching to boil and time is a typical fermentation value, reset to 60 min
                    else if (newValue === "add_to_boil" && timeNum <= 10) {
                      form.setFieldValue("time", "60");
                    }
                  }}
                >
                  <SelectTrigger id={field.name}>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="add_to_boil">Boil</SelectItem>
                    <SelectItem value="add_to_fermentation">Fermentation (Dry Hop)</SelectItem>
                  </SelectContent>
                </Select>
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

          <form.Subscribe selector={(state) => state.values.use}>
            {(useType) => {
              const isBoil = useType === "add_to_boil";
              return (
                <>
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
                        <Label htmlFor={field.name}>
                          {isBoil ? "Boil Time (minutes)" : "Add After (days)"}
                        </Label>
                        <Input
                          id={field.name}
                          type="number"
                          step={isBoil ? "1" : "0.5"}
                          min="0"
                          value={field.state.value}
                          onBlur={field.handleBlur}
                          onChange={(e) => field.handleChange(e.target.value)}
                          placeholder={isBoil ? "60" : "3"}
                        />
                        {!isBoil && (
                          <p className="text-xs text-muted-foreground">
                            Days into fermentation before adding
                          </p>
                        )}
                        {field.state.meta.errors && (
                          <p className="text-sm text-destructive">
                            {field.state.meta.errors.join(", ")}
                          </p>
                        )}
                      </div>
                    )}
                  </form.Field>

                  {!isBoil && (
                    <form.Field
                      name="duration"
                      validators={{
                        onChange: ({ value }) => {
                          // Allow empty string (unlimited duration)
                          if (value === "") {
                            return undefined;
                          }
                          const num = parseFloat(value);
                          if (isNaN(num) || num <= 0) {
                            return "Duration must be a positive number or empty for unlimited";
                          }
                          return undefined;
                        },
                      }}
                    >
                      {(field) => (
                        <div className="space-y-2">
                          <Label htmlFor={field.name}>Duration (days) - Optional</Label>
                          <Input
                            id={field.name}
                            type="number"
                            step="0.5"
                            min="0.5"
                            value={field.state.value}
                            onBlur={field.handleBlur}
                            onChange={(e) => field.handleChange(e.target.value)}
                            placeholder="Leave empty for unlimited"
                          />
                          <p className="text-xs text-muted-foreground">
                            How long to leave hops in fermenter (empty = unlimited)
                          </p>
                          {field.state.meta.errors && (
                            <p className="text-sm text-destructive">
                              {field.state.meta.errors.join(", ")}
                            </p>
                          )}
                        </div>
                      )}
                    </form.Field>
                  )}
                </>
              );
            }}
          </form.Subscribe>

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
