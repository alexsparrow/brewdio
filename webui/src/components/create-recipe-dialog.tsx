import { useState } from "react";
import { useForm } from "@tanstack/react-form";
import { useNavigate } from "@tanstack/react-router";
import { useLiveQuery } from "@tanstack/react-db";
import { settingsCollection } from "@/db";
import { createRecipe, getAvailableStyles } from "@/lib/actions/recipes";
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

export function CreateRecipeDialog() {
  const [open, setOpen] = useState(false);
  const navigate = useNavigate();
  const { data: settingsData } = useLiveQuery(settingsCollection);
  const settings = settingsData?.find((s) => s.id === "user-settings");
  const availableStyles = getAvailableStyles();

  const form = useForm({
    defaultValues: {
      name: "",
      batchSize: "5.0",
      styleName: "",
    },
    onSubmit: async ({ value }) => {
      try {
        const recipeId = await createRecipe({
          name: value.name,
          batchSize: parseFloat(value.batchSize),
          styleName: value.styleName,
          author: settings?.defaultAuthor || "Brewdio User",
        });

        // Reset form and close dialog
        form.reset();
        setOpen(false);

        // Navigate to the new recipe
        navigate({ to: "/recipes/$recipeId", params: { recipeId } });
      } catch (error) {
        console.error("Failed to create recipe:", error);
        // You could add error handling UI here
      }
    },
  });

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button className="gap-2">
          <Plus className="h-4 w-4" />
          Create Recipe
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Create New Recipe</DialogTitle>
          <DialogDescription>
            Enter the details for your new brewing recipe.
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
            name="name"
            validators={{
              onChange: ({ value }) =>
                !value ? "Recipe name is required" : undefined,
            }}
          >
            {(field) => (
              <div className="space-y-2">
                <Label htmlFor={field.name}>Recipe Name</Label>
                <Input
                  id={field.name}
                  value={field.state.value}
                  onBlur={field.handleBlur}
                  onChange={(e) => field.handleChange(e.target.value)}
                  placeholder="e.g., West Coast IPA"
                />
                {field.state.meta.errors && (
                  <p className="text-sm text-destructive">
                    {field.state.meta.errors.join(", ")}
                  </p>
                )}
              </div>
            )}
          </form.Field>

          <form.Field
            name="batchSize"
            validators={{
              onChange: ({ value }) => {
                const num = parseFloat(value);
                if (isNaN(num) || num <= 0) {
                  return "Batch size must be a positive number";
                }
                return undefined;
              },
            }}
          >
            {(field) => (
              <div className="space-y-2">
                <Label htmlFor={field.name}>Batch Size (gallons)</Label>
                <Input
                  id={field.name}
                  type="number"
                  step="0.1"
                  min="0.1"
                  value={field.state.value}
                  onBlur={field.handleBlur}
                  onChange={(e) => field.handleChange(e.target.value)}
                  placeholder="5.0"
                />
                {field.state.meta.errors && (
                  <p className="text-sm text-destructive">
                    {field.state.meta.errors.join(", ")}
                  </p>
                )}
              </div>
            )}
          </form.Field>

          <form.Field
            name="styleName"
            validators={{
              onChange: ({ value }) =>
                !value ? "Style selection is required" : undefined,
            }}
          >
            {(field) => (
              <div className="space-y-2">
                <Label htmlFor={field.name}>Beer Style</Label>
                <Combobox
                  options={availableStyles.map((s) => ({
                    value: s.name,
                    label: `${s.name} - ${s.category}`,
                  }))}
                  value={field.state.value}
                  onValueChange={field.handleChange}
                  placeholder="Select a beer style"
                  searchPlaceholder="Search styles..."
                  emptyText="No style found."
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
                  {isSubmitting ? "Creating..." : "Create Recipe"}
                </Button>
              )}
            </form.Subscribe>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
