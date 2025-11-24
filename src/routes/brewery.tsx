import { createFileRoute } from "@tanstack/react-router";
import { useState, useMemo } from "react";
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
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Plus, Trash2 } from "lucide-react";
import { BreweryVisualization } from "@/components/brewery-visualization";
import { BreweryBatchSettings } from "@/components/brewery-batch-settings";
import { BreweryLossControls, type Stage, type CalculatedStage } from "@/components/brewery-loss-controls";

export const Route = createFileRoute("/brewery")({
  component: BreweryComponent,
});

interface EquipmentProfile {
  id: string;
  name: string;
}

// Default equipment profile
const DEFAULT_PROFILE: EquipmentProfile = {
  id: "default",
  name: "Default Setup",
};

// Initial stages configuration
const INITIAL_STAGES: Stage[] = [
  {
    id: "mash",
    label: "Mash Tun",
    shape: "dome",
    losses: [
      { id: "grainAbs", label: "Grain Absorption", value: 0.2, type: "rate", unit: "gal/kg" },
      { id: "tunDead", label: "Tun Deadspace", value: 0.25, type: "flat", unit: "gal" }
    ]
  },
  {
    id: "kettle",
    label: "Boil Kettle",
    shape: "chimney",
    losses: [
      { id: "boilOff", label: "Boil Off Rate", value: 1.0, type: "rate", unit: "gal/hr" },
      { id: "trub", label: "Trub / Hop Loss", value: 0.5, type: "flat", unit: "gal" }
    ]
  },
  {
    id: "fermenter",
    label: "Fermenter",
    shape: "conical",
    losses: [
      { id: "trub_ferm", label: "Yeast/Cake Loss", value: 0.5, type: "flat", unit: "gal" }
    ]
  },
  {
    id: "packaging",
    label: "Kegging",
    shape: "keg",
    losses: []
  }
];

interface CalculationResult {
  totalWaterNeeded: number;
  stages: CalculatedStage[];
}

function BreweryComponent() {
  const [profiles, setProfiles] = useState<EquipmentProfile[]>([DEFAULT_PROFILE]);
  const [selectedProfileId, setSelectedProfileId] = useState<string>(DEFAULT_PROFILE.id);
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [newProfileName, setNewProfileName] = useState("");

  // Brewery state
  const [targetVolume, setTargetVolume] = useState<number>(5.0);
  const [boilTime, setBoilTime] = useState<number>(60);
  const [grainWeight, setGrainWeight] = useState<number>(4.0); // Default 4kg for 5 gallon batch
  const [beerColor, setBeerColor] = useState<string>("#F59E0B");
  const [stages, setStages] = useState<Stage[]>(INITIAL_STAGES);
  const [selectedStageId, setSelectedStageId] = useState<string | null>(null);

  // Calculate water needs
  const calculationData = useMemo((): CalculationResult => {
    let currentVol = targetVolume;
    const processedStages: CalculatedStage[] = [];

    [...stages].reverse().forEach(stage => {
      let stageLossTotal = 0;
      stage.losses.forEach(loss => {
        let val = loss.value;
        if (loss.type === 'rate') {
          // For grain absorption, multiply by grain weight; for boil off, multiply by boil time
          if (loss.unit.includes('kg')) {
            val = val * grainWeight;
          } else if (loss.unit.includes('hr')) {
            val = val * (boilTime / 60);
          }
        }
        stageLossTotal += val;
      });

      const volIn = currentVol + stageLossTotal;
      processedStages.push({
        ...stage,
        volumeIn: volIn,
        volumeOut: currentVol,
        totalLoss: stageLossTotal,
      });
      currentVol = volIn;
    });

    const orderedStages = processedStages.reverse();

    const sourceTank: CalculatedStage = {
      id: 'source',
      label: 'Strike Water',
      shape: 'rect',
      losses: [],
      volumeIn: currentVol,
      volumeOut: currentVol,
      totalLoss: 0,
      isSource: true
    };

    return {
      totalWaterNeeded: currentVol,
      stages: [sourceTank, ...orderedStages]
    };
  }, [targetVolume, boilTime, grainWeight, stages]);

  // Find selected stage and its index
  const selectedStageIndex = selectedStageId
    ? calculationData.stages.findIndex(s => s.id === selectedStageId)
    : -1;
  const selectedStage = selectedStageIndex > 0
    ? calculationData.stages[selectedStageIndex]
    : null;

  const handleAddProfile = () => {
    if (!newProfileName.trim()) return;

    const newProfile: EquipmentProfile = {
      id: `profile-${Date.now()}`,
      name: newProfileName.trim(),
    };

    setProfiles((prev) => [...prev, newProfile]);
    setSelectedProfileId(newProfile.id);
    setNewProfileName("");
    setIsAddDialogOpen(false);
  };

  const handleDeleteProfile = (profileId: string) => {
    // Prevent deleting the default profile
    if (profileId === DEFAULT_PROFILE.id) return;

    setProfiles((prev) => {
      const filtered = prev.filter((p) => p.id !== profileId);

      // If we're deleting the selected profile, switch to default
      if (selectedProfileId === profileId) {
        setSelectedProfileId(DEFAULT_PROFILE.id);
      }

      return filtered;
    });
  };

  const handleLossChange = (stageIndex: number, lossIndex: number, val: number) => {
    const realIndex = stageIndex - 1;
    if (realIndex < 0) return;
    const newStages = [...stages];
    newStages[realIndex].losses[lossIndex].value = val;
    setStages(newStages);
  };

  const handleAddCustomLoss = (stageIndex: number) => {
    const realIndex = stageIndex - 1;
    if (realIndex < 0) return;
    const newStages = [...stages];
    newStages[realIndex].losses.push({
      id: `custom_${Date.now()}`,
      label: "Custom Loss",
      value: 0.1,
      type: "flat",
      unit: "gal"
    });
    setStages(newStages);
  };

  return (
    <div className="flex flex-col gap-6">
      {/* Header with Equipment Profile Selector */}
      <div className="flex items-center justify-between gap-6">
        <div>
          <h1 className="text-3xl font-bold">Brewery</h1>
          <p className="text-muted-foreground mt-2">
            Configure your brewery equipment and visualize water losses.
          </p>
        </div>

        {/* Equipment Profile Selector - Right Side */}
        <div className="flex items-center gap-3">
          <Select value={selectedProfileId} onValueChange={setSelectedProfileId}>
            <SelectTrigger className="w-[240px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {profiles.map((profile) => (
                <SelectItem key={profile.id} value={profile.id}>
                  {profile.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Dialog open={isAddDialogOpen} onOpenChange={setIsAddDialogOpen}>
            <DialogTrigger asChild>
              <Button size="default" variant="outline">
                <Plus className="h-4 w-4 mr-2" />
                Add Profile
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Add Equipment Profile</DialogTitle>
                <DialogDescription>
                  Create a new equipment profile to track different brewery setups.
                </DialogDescription>
              </DialogHeader>
              <div className="space-y-4 py-4">
                <div className="space-y-2">
                  <Label htmlFor="profile-name">Profile Name</Label>
                  <Input
                    id="profile-name"
                    placeholder="e.g., 10 Gallon System"
                    value={newProfileName}
                    onChange={(e) => setNewProfileName(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        handleAddProfile();
                      }
                    }}
                  />
                </div>
              </div>
              <DialogFooter>
                <Button variant="outline" onClick={() => setIsAddDialogOpen(false)}>
                  Cancel
                </Button>
                <Button onClick={handleAddProfile} disabled={!newProfileName.trim()}>
                  Add Profile
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>

          {selectedProfileId !== DEFAULT_PROFILE.id && (
            <Button
              variant="destructive"
              size="icon"
              onClick={() => handleDeleteProfile(selectedProfileId)}
              title="Delete profile"
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          )}
        </div>
      </div>

      {/* Visualization - Full Width */}
      <div className="mt-2">
        <BreweryVisualization
          stages={calculationData.stages}
          beerColor={beerColor}
          totalWaterNeeded={calculationData.totalWaterNeeded}
          selectedStageId={selectedStageId}
          onBeerColorChange={setBeerColor}
          onStageSelect={setSelectedStageId}
        />
      </div>

      {/* Controls Below - Two Columns */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Left - Batch Settings */}
        <BreweryBatchSettings
          targetVolume={targetVolume}
          boilTime={boilTime}
          grainWeight={grainWeight}
          onTargetVolumeChange={setTargetVolume}
          onBoilTimeChange={setBoilTime}
          onGrainWeightChange={setGrainWeight}
        />

        {/* Right - Selected Stage Loss Controls */}
        <BreweryLossControls
          stage={selectedStage}
          stageIndex={selectedStageIndex}
          onLossChange={handleLossChange}
          onAddCustomLoss={handleAddCustomLoss}
        />
      </div>
    </div>
  );
}
