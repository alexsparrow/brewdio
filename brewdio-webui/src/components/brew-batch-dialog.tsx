import { useState } from "react";
import { useNavigate } from "@tanstack/react-router";
import { useBatches, createBatchImperative, batchKeys } from "@/lib/db/batches";
import { useQueryClient } from "@tanstack/react-query";
import type { RecipeDocument } from "@/lib/db/recipes";
import { getEquipment } from "brewdio-wasm";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Beaker } from "lucide-react";

interface BrewBatchDialogProps {
  recipe: RecipeDocument;
}

export function BrewBatchDialog({ recipe }: BrewBatchDialogProps) {
  const navigate = useNavigate();
  const { data: batches } = useBatches();
  const queryClient = useQueryClient();

  // Static equipment data
  const equipmentProfiles = getEquipment();
  const defaultEquipment = equipmentProfiles[0];

  const [open, setOpen] = useState(false);
  const [isCreating, setIsCreating] = useState(false);

  // Generate default batch name
  const recipeBatches = batches?.filter(b => b.recipeId === recipe.id) || [];
  const batchNumber = recipeBatches.length + 1;
  const defaultBatchName = `${recipe.recipe.name} Batch ${batchNumber}`;

  const [batchName, setBatchName] = useState(defaultBatchName);
  const recipeEquipmentName = recipe.equipment?.name;
  const initialEquipmentName = recipeEquipmentName || defaultEquipment?.name || "Default Setup";

  const [selectedEquipmentName, setSelectedEquipmentName] = useState(initialEquipmentName);

  // Reset form when dialog opens
  const handleOpenChange = (newOpen: boolean) => {
    setOpen(newOpen);
    if (newOpen) {
      // Recalculate batch number in case new batches were created
      const currentBatches = batches?.filter(b => b.recipeId === recipe.id) || [];
      const newBatchNumber = currentBatches.length + 1;
      setBatchName(`${recipe.recipe.name} Batch ${newBatchNumber}`);
      setSelectedEquipmentName(recipe.equipment?.name || defaultEquipment?.name || "Default Setup");
    }
  };

  const handleCreate = async () => {
    setIsCreating(true);
    try {
      // Get selected equipment
      const equipment = equipmentProfiles.find(e => e.name === selectedEquipmentName) || defaultEquipment;

      const batchId = createBatchImperative(
        batchName.trim() || defaultBatchName,
        recipe.id,
        {
          equipmentId: equipment?.name || "default",
          recipe: structuredClone(recipe.recipe),
          equipment: structuredClone(equipment),
          brewDate: Date.now(),
          createdAt: Date.now(),
          updatedAt: Date.now(),
        },
      );

      queryClient.invalidateQueries({ queryKey: batchKeys.all });
      setOpen(false);
      navigate({ to: `/batches/${batchId}/overview` });
    } catch (error) {
      console.error("Failed to create batch:", error);
    } finally {
      setIsCreating(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogTrigger asChild>
        <Button variant="default">
          <Beaker className="h-4 w-4 mr-2" />
          Brew
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Brew New Batch</DialogTitle>
          <DialogDescription>
            Create a new batch from this recipe. You can customize the batch name and select your equipment profile.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Batch Name */}
          <div className="space-y-2">
            <Label htmlFor="batch-name">Batch Name</Label>
            <Input
              id="batch-name"
              value={batchName}
              onChange={(e) => setBatchName(e.target.value)}
              placeholder={defaultBatchName}
              disabled={isCreating}
            />
          </div>

          {/* Equipment Profile */}
          <div className="space-y-2">
            <Label htmlFor="equipment-profile">Equipment Profile</Label>
            <select
              id="equipment-profile"
              value={selectedEquipmentName}
              onChange={(e) => setSelectedEquipmentName(e.target.value)}
              disabled={isCreating}
              className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
            >
              {equipmentProfiles.map((profile) => (
                <option key={profile.name} value={profile.name}>
                  {profile.name}
                </option>
              ))}
            </select>
            <p className="text-xs text-muted-foreground">
              This equipment profile will be used for water calculations and brew day planning.
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => setOpen(false)}
            disabled={isCreating}
          >
            Cancel
          </Button>
          <Button
            onClick={handleCreate}
            disabled={isCreating}
          >
            {isCreating ? "Creating..." : "Create Batch"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
