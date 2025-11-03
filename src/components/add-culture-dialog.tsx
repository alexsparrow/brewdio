import { useState } from "react";
import { useForm } from "@tanstack/react-form";
import { recipesCollection, type RecipeDocument } from "@/db";
import type { CultureAdditionType, UnitUnitType } from "@beerjson/beerjson";
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
import cultures from "@/data/cultures.json";

interface AddCultureDialogProps {
  recipeId: string;
  recipe: RecipeDocument;
  existingCulture?: CultureAdditionType;
  index?: number;
  trigger?: React.ReactNode;
}

const unitTypes: UnitUnitType[] = ["1", "unit", "each", "dimensionless", "pkg"];

export function AddCultureDialog({
  recipeId,
  recipe,
  existingCulture,
  index,
  trigger,
}: AddCultureDialogProps) {
  const [open, setOpen] = useState(false);
  const isEditing = existingCulture !== undefined && index !== undefined;

  const form = useForm({
    defaultValues: {
      cultureName: existingCulture?.name || "",
      amount: existingCulture?.amount?.value.toString() || "1",
      unit: (existingCulture?.amount?.unit || "pkg") as UnitUnitType,
    },
    onSubmit: async ({ value }) => {
      // Find the selected culture from the cultures data
      const selectedCulture = cultures.find((c) => c.name === value.cultureName);

      if (!selectedCulture) {
        return;
      }

      // Create the culture addition
      const cultureAddition: CultureAdditionType = {
        name: selectedCulture.name,
        type: selectedCulture.type as CultureAdditionType["type"],
        form: selectedCulture.form as CultureAdditionType["form"],
        producer: selectedCulture.producer,
        attenuation: selectedCulture.attenuation_range?.minimum,
        amount: {
          value: parseFloat(value.amount),
          unit: value.unit,
        },
      };

      // Update in the database
      await recipesCollection.update(recipeId, (draft) => {
        if (isEditing && index !== undefined) {
          // Edit existing culture
          draft.recipe.ingredients.culture_additions =
            draft.recipe.ingredients.culture_additions?.map((item, idx) =>
              idx === index ? cultureAddition : item
            ) || [];
        } else {
          // Add new culture
          draft.recipe.ingredients.culture_additions = [
            ...(draft.recipe.ingredients.culture_additions || []),
            cultureAddition,
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
            Add Culture
          </Button>
        )}
      </DialogTrigger>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>{isEditing ? "Edit Culture (Yeast)" : "Add Culture (Yeast)"}</DialogTitle>
          <DialogDescription>
            {isEditing
              ? "Update the culture details."
              : "Select a yeast culture and specify the amount to add to your recipe."}
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
            name="cultureName"
            validators={{
              onChange: ({ value }) =>
                !value ? "Culture selection is required" : undefined,
            }}
          >
            {(field) => (
              <div className="space-y-2">
                <Label htmlFor={field.name}>Yeast Culture</Label>
                <Combobox
                  options={cultures.map((c) => ({
                    value: c.name,
                    label: `${c.name} (${c.type})`,
                  }))}
                  value={field.state.value}
                  onValueChange={field.handleChange}
                  placeholder="Select a culture"
                  searchPlaceholder="Search cultures..."
                  emptyText="No culture found."
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
                    step="1"
                    min="1"
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onChange={(e) => field.handleChange(e.target.value)}
                    placeholder="1"
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
                      {unitTypes.map((unit) => (
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
                    ? "Update Culture"
                    : "Add Culture"}
                </Button>
              )}
            </form.Subscribe>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
