import { createFileRoute, Link } from "@tanstack/react-router";
import { useBatch, updateBatchImperative, batchKeys } from "@/lib/db/batches";
import { useQueryClient } from "@tanstack/react-query";
import { get_equipment } from "brewdio-wasm";
import { EditableNotes } from "@/components/editable-notes";
import { InlineEditable } from "@/components/inline-editable";
import { useEffect } from "react";
import { useCalculation } from "@/hooks/use-calculation";
import { stores } from "@/lib/calculate";
import { RetroCockpitDial } from "@/components/retro-cockpit-dial";
import { GravitySightGlass } from "@/components/gravity-sight-glass";
import { Screw } from "@/components/screw";
import { srm_to_srgb, rgb_to_hex, color_to_srm } from "brewdio-wasm";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Info, Calendar } from "lucide-react";
import { GrainIcon, HopIcon, YeastIcon } from "@/components/ingredient-icons";
import type { FermentableAdditionType } from "brewdio-wasm";
import { BatchWaterCalculator } from "@/components/batch-water-calculator";
import { Input } from "@/components/ui/input";
import { DeleteBatchDialog } from "@/components/delete-batch-dialog";

export const Route = createFileRoute("/batches/$batchId_/overview")({
  component: BatchOverviewComponent,
});

/**
 * Get hex color for a fermentable based on its SRM value
 */
function getFermentableColorHex(fermentable: FermentableAdditionType): string | undefined {
  const colorData = (fermentable.type as any)?.color || (fermentable as any).color;

  if (!colorData) {
    return undefined;
  }

  try {
    const srm = color_to_srm(colorData);
    const rgb = srm_to_srgb(srm) as [number, number, number];
    return rgb_to_hex(rgb[0], rgb[1], rgb[2]);
  } catch {
    return undefined;
  }
}

function BatchOverviewComponent() {
  const { batchId } = Route.useParams();
  const { data: batch, status } = useBatch(batchId);
  const queryClient = useQueryClient();

  // Static equipment data
  const equipmentProfiles = get_equipment();

  const og = useCalculation<number>("og");
  const fg = useCalculation<number>("fg");
  const abv = useCalculation<number>("abv");
  const color = useCalculation<number>("color");
  const ibu = useCalculation<number>("ibu");

  // Helper to update the batch
  const updateBatch = (updater: (b: typeof batch) => Partial<typeof batch>) => {
    if (!batch) return;
    const updates = updater(batch);
    const updated = { ...batch, ...updates, updatedAt: Date.now() };
    updateBatchImperative(batchId, updated.name, {
      equipmentId: updated.equipmentId,
      recipe: updated.recipe,
      equipment: updated.equipment,
      brewDate: updated.brewDate,
      notes: updated.notes,
      createdAt: updated.createdAt,
      updatedAt: updated.updatedAt,
    });
    queryClient.invalidateQueries({ queryKey: batchKeys.all });
    queryClient.invalidateQueries({ queryKey: batchKeys.detail(batchId) });
  };

  // Sync batch's recipe data to calculation store
  useEffect(() => {
    if (batch) {
      stores.state.setState(() => ({
        recipe: batch.recipe,
      }));
    }
  }, [batch]);

  if (status === "pending") {
    return <div>Loading batch...</div>;
  }

  if (!batch) {
    return <div>Batch not found</div>;
  }

  const { recipe: beerRecipe } = batch;

  const handleNameUpdate = (newName: string) => {
    updateBatch(() => ({ name: newName }));
  };

  const handleNotesUpdate = (newNotes: string) => {
    updateBatch(() => ({ notes: newNotes }));
  };

  const handleEquipmentChange = (newEquipmentId: string) => {
    const selectedEquipment = equipmentProfiles.find(e => e.name === newEquipmentId);
    if (!selectedEquipment) return;

    updateBatch(() => ({
      equipmentId: newEquipmentId,
      equipment: structuredClone(selectedEquipment),
    }));
  };

  const handleBrewDateChange = (newDate: string) => {
    const timestamp = new Date(newDate).getTime();
    updateBatch(() => ({ brewDate: timestamp }));
  };

  const liquidColor = color ? (() => { const rgb = srm_to_srgb(color) as [number, number, number]; return rgb_to_hex(rgb[0], rgb[1], rgb[2]); })() : "#FBB123";

  // Format brew date for display and input
  const brewDate = new Date(batch.brewDate);
  const brewDateStr = brewDate.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  });
  const brewDateInputValue = brewDate.toISOString().split('T')[0]; // YYYY-MM-DD format for input

  // Extract style ranges if available
  const styleRanges = beerRecipe.style ? {
    og: beerRecipe.style.original_gravity ? {
      min: beerRecipe.style.original_gravity.minimum?.value ?? null,
      max: beerRecipe.style.original_gravity.maximum?.value ?? null
    } : null,
    fg: beerRecipe.style.final_gravity ? {
      min: beerRecipe.style.final_gravity.minimum?.value ?? null,
      max: beerRecipe.style.final_gravity.maximum?.value ?? null
    } : null,
    ibu: beerRecipe.style.international_bitterness_units ? {
      min: beerRecipe.style.international_bitterness_units.minimum?.value ?? null,
      max: beerRecipe.style.international_bitterness_units.maximum?.value ?? null
    } : null,
    color: beerRecipe.style.color ? {
      min: beerRecipe.style.color.minimum?.value ?? null,
      max: beerRecipe.style.color.maximum?.value ?? null
    } : null,
    abv: beerRecipe.style.alcohol_by_volume ? {
      min: beerRecipe.style.alcohol_by_volume.minimum?.value ?? null,
      max: beerRecipe.style.alcohol_by_volume.maximum?.value ?? null
    } : null
  } : null;

  return (
    <div className="flex flex-col gap-6">
      {/* Editable Batch Header */}
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <InlineEditable
            value={batch.name}
            onSave={handleNameUpdate}
            displayAs="heading"
            placeholder="Batch name"
          />
          <p className="text-sm text-muted-foreground mt-1">
            Batch overview and brew day guide
          </p>
        </div>
        <DeleteBatchDialog
          batchId={batchId}
          batchName={batch.name}
          variant="destructive"
          size="sm"
          redirectOnDelete={true}
        />
      </div>

      {/* Batch Configuration */}
      <div className="border rounded-lg p-4">
        <h3 className="text-lg font-semibold mb-4">Batch Configuration</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* Brew Date */}
          <div>
            <label htmlFor="brew-date" className="block text-sm font-medium mb-2">
              <Calendar className="h-4 w-4 inline mr-1" />
              Brew Date
            </label>
            <Input
              id="brew-date"
              type="date"
              value={brewDateInputValue}
              onChange={(e) => handleBrewDateChange(e.target.value)}
              className="w-full"
            />
          </div>

          {/* Equipment Profile */}
          <div>
            <label htmlFor="equipment-select" className="block text-sm font-medium mb-2">
              Equipment Profile
            </label>
            <select
              id="equipment-select"
              value={batch.equipmentId}
              onChange={(e) => handleEquipmentChange(e.target.value)}
              className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
            >
              {equipmentProfiles.map((profile) => (
                <option key={profile.name} value={profile.name}>
                  {profile.name}
                </option>
              ))}
            </select>
          </div>

          {/* Packaging Type */}
          <div>
            <label htmlFor="packaging-type" className="block text-sm font-medium mb-2">
              Packaging Type
            </label>
            <select
              id="packaging-type"
              className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
            >
              <option value="keg">Keg</option>
              <option value="bottle">Bottle</option>
              <option value="cask">Cask</option>
              <option value="tank">Tank</option>
              <option value="firkin">Firkin</option>
              <option value="other">Other</option>
            </select>
          </div>
        </div>
      </div>

      {/* Info Alert */}
      <Alert>
        <Info className="h-4 w-4" />
        <AlertDescription>
          This page is designed to guide you through your brew day.
          <Link
            to="/batches/$batchId"
            params={{ batchId }}
            className="ml-1 underline hover:text-primary font-medium"
          >
            Edit the recipe
          </Link> if you need to make adjustments.
        </AlertDescription>
      </Alert>

      {/* Calculated Values Section - Retro Cockpit Panel */}
      <div className="relative bg-gray-100 dark:bg-gray-900 border-t-2 border-gray-300 dark:border-gray-700 rounded-xl p-4 shadow-xl overflow-hidden">
        {/* Subtle texture overlay */}
        <div className="absolute inset-0 opacity-5 pointer-events-none bg-[url('https://www.transparenttextures.com/patterns/brushed-alum-dark.png')]"></div>

        {/* Panel Mounting Screws */}
        <Screw tl />
        <Screw tr />
        <Screw bl />
        <Screw br />

        {/* Main Grid Layout */}
        <div className="grid grid-cols-1 md:grid-cols-12 gap-4 relative z-10 p-2">
          {/* LEFT COLUMN: GRAVITY SECTION */}
          <div className="md:col-span-3 flex justify-around bg-white/50 dark:bg-gray-950/30 p-3 rounded-lg border border-gray-300 dark:border-gray-800">
            <GravitySightGlass
              label="OG"
              value={og ?? 1.0}
              liquidColor={liquidColor}
              percentage={0}
              min={1.0}
              max={1.1}
              targetMin={styleRanges?.og?.min ?? 1.04}
              targetMax={styleRanges?.og?.max ?? 1.07}
            />
            <div className="self-center text-gray-600 dark:text-gray-500 text-2xl">
              →
            </div>
            <GravitySightGlass
              label="FG"
              value={fg ?? 1.0}
              liquidColor={liquidColor}
              percentage={0}
              min={1.0}
              max={1.04}
              targetMin={styleRanges?.fg?.min ?? 1.008}
              targetMax={styleRanges?.fg?.max ?? 1.016}
            />
          </div>

          {/* RIGHT COLUMN: DIALS SECTION */}
          <div className="md:col-span-9 grid grid-cols-1 sm:grid-cols-3 gap-3">
            {/* IBU DIAL */}
            <div className="bg-white/50 dark:bg-gray-950/30 p-2 rounded-lg border border-gray-300 dark:border-gray-800 flex items-center justify-center">
              <RetroCockpitDial
                label="IBU"
                value={ibu ?? 0}
                min={0}
                max={100}
                targetMin={styleRanges?.ibu?.min ?? 20}
                targetMax={styleRanges?.ibu?.max ?? 60}
              />
            </div>

            {/* COLOUR DIAL */}
            <div className="bg-white/50 dark:bg-gray-950/30 p-2 rounded-lg border border-gray-300 dark:border-gray-800 flex items-center justify-center">
              <RetroCockpitDial
                label="SRM"
                value={color !== undefined ? color : 0}
                min={0}
                max={40}
                targetMin={styleRanges?.color ? styleRanges.color.min : 4}
                targetMax={styleRanges?.color ? styleRanges.color.max : 20}
                showSrmGradient={true}
              />
            </div>

            {/* ABV DIAL */}
            <div className="bg-white/50 dark:bg-gray-950/30 p-2 rounded-lg border border-gray-300 dark:border-gray-800 flex items-center justify-center">
              <RetroCockpitDial
                label="ABV"
                value={abv ?? 0}
                min={0}
                max={12}
                targetMin={styleRanges?.abv?.min ?? 4}
                targetMax={styleRanges?.abv?.max ?? 7}
              />
            </div>
          </div>
        </div>
      </div>

      {/* Recipe Details and Notes */}
      <div className="grid gap-4 md:grid-cols-2">
        <div className="border rounded-lg p-4">
          <h3 className="text-xl font-semibold mb-3">Recipe Details</h3>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Style:</span>
              <span className="font-medium">{beerRecipe.style?.name || "N/A"}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Batch Size:</span>
              <span className="font-medium">{beerRecipe.batch_size.value} {beerRecipe.batch_size.unit}</span>
            </div>
            {beerRecipe.boil?.pre_boil_size && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Boil Size:</span>
                <span className="font-medium">{beerRecipe.boil.pre_boil_size.value} {beerRecipe.boil.pre_boil_size.unit}</span>
              </div>
            )}
            {beerRecipe.boil?.boil_time && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Boil Time:</span>
                <span className="font-medium">{beerRecipe.boil.boil_time.value} {beerRecipe.boil.boil_time.unit}</span>
              </div>
            )}
            {beerRecipe.efficiency && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">Brewhouse Efficiency:</span>
                <span className="font-medium">{beerRecipe.efficiency.brewhouse.value}%</span>
              </div>
            )}
          </div>
        </div>

        <div className="border rounded-lg p-4">
          <h3 className="text-xl font-semibold mb-3">Brew Day Notes</h3>
          <EditableNotes
            value={batch.notes || ""}
            onSave={handleNotesUpdate}
            placeholder="Record observations during brewing: mash temperature, gravity readings, hop additions, fermentation notes, packaging dates, tasting notes, etc..."
          />
        </div>
      </div>

      {/* Water Calculator */}
      <BatchWaterCalculator batch={batch} beerColor={liquidColor} />

      {/* Recipe Ingredients - Read Only */}
      <div className="space-y-4">
        <div className="border rounded-lg p-4">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-xl font-semibold">Fermentables</h3>
            <Link
              to="/batches/$batchId"
              params={{ batchId }}
              hash="#fermentables"
              className="text-sm text-muted-foreground hover:text-primary underline"
            >
              Edit recipe
            </Link>
          </div>
          {beerRecipe.ingredients.fermentable_additions?.length === 0 ? (
            <div className="text-muted-foreground text-sm">
              No fermentables added yet
            </div>
          ) : (
            <div className="space-y-2">
              {beerRecipe.ingredients.fermentable_additions?.map((ferm, idx) => (
                <div
                  key={idx}
                  className="flex justify-between items-center text-sm border-b pb-2 last:border-0"
                >
                  <div className="flex items-center gap-1.5">
                    <GrainIcon
                      className="h-6 w-6 flex-shrink-0"
                      srmColor={getFermentableColorHex(ferm)}
                    />
                    <span className="font-medium">
                      {ferm.name || "Unnamed fermentable"}
                    </span>
                  </div>
                  <span className="text-muted-foreground">
                    {ferm.amount.value} {ferm.amount.unit}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="border rounded-lg p-4">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-xl font-semibold">Hops</h3>
            <Link
              to="/batches/$batchId"
              params={{ batchId }}
              hash="#hops"
              className="text-sm text-muted-foreground hover:text-primary underline"
            >
              Edit recipe
            </Link>
          </div>
          {beerRecipe.ingredients.hop_additions?.length === 0 ? (
            <div className="text-muted-foreground text-sm">No hops added yet</div>
          ) : (
            <div className="space-y-2">
              {beerRecipe.ingredients.hop_additions?.map((hop, idx) => (
                <div
                  key={idx}
                  className="flex justify-between items-center text-sm border-b pb-2 last:border-0"
                >
                  <div className="flex items-center gap-1.5">
                    <HopIcon className="h-6 w-6 flex-shrink-0" />
                    <div>
                      <span className="font-medium">
                        {hop.name || "Unnamed hop"}
                      </span>
                      {hop.timing && (
                        <span className="text-muted-foreground ml-2">
                          {hop.timing.use === "add_to_boil" && "Boil"}
                          {hop.timing.use === "add_to_fermentation" && "Dry Hop"}
                          {hop.timing.time && (
                            <> @ {hop.timing.time.value} {hop.timing.time.unit}</>
                          )}
                          {hop.timing.duration && (
                            <> ({hop.timing.duration.value} {hop.timing.duration.unit})</>
                          )}
                        </span>
                      )}
                    </div>
                  </div>
                  <span className="text-muted-foreground">
                    {hop.amount.value} {hop.amount.unit}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="border rounded-lg p-4">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-xl font-semibold">Cultures (Yeast)</h3>
            <Link
              to="/batches/$batchId"
              params={{ batchId }}
              hash="#cultures"
              className="text-sm text-muted-foreground hover:text-primary underline"
            >
              Edit recipe
            </Link>
          </div>
          {beerRecipe.ingredients.culture_additions?.length === 0 ? (
            <div className="text-muted-foreground text-sm">
              No cultures added yet
            </div>
          ) : (
            <div className="space-y-2">
              {beerRecipe.ingredients.culture_additions?.map((culture, idx) => (
                <div
                  key={idx}
                  className="flex justify-between items-center text-sm border-b pb-2 last:border-0"
                >
                  <div className="flex items-center gap-1.5">
                    <YeastIcon className="h-6 w-6 flex-shrink-0" />
                    <span className="font-medium">
                      {culture.name || "Unnamed culture"}
                    </span>
                  </div>
                  {culture.amount && (
                    <span className="text-muted-foreground">
                      {culture.amount.value} {culture.amount.unit}
                    </span>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
