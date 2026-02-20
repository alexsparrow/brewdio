import { useState, useEffect } from "react";
import { useLiveQuery } from "@tanstack/react-db";
import { mashProfilesCollection } from "@/db";
import { useRecipeEdit } from "@/contexts/recipe-edit-context";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { MashStepList } from "@/components/mash-step-list";
import { Edit, Save, RotateCcw } from "lucide-react";
import type { MashProcedureType, MashStepType } from "@beerjson/beerjson";

export function RecipeMashSection() {
  const { id, collection, document: recipe } = useRecipeEdit();
  const { data: mashProfiles } = useLiveQuery(mashProfilesCollection);

  const [isEditing, setIsEditing] = useState(false);
  const [editedMash, setEditedMash] = useState<MashProcedureType | null>(null);
  const [hasChanges, setHasChanges] = useState(false);

  // Early return if recipe not loaded yet
  if (!recipe) {
    return <div>Loading mash data...</div>;
  }

  // Load current mash into editable state
  useEffect(() => {
    if (recipe?.recipe?.mash) {
      setEditedMash(structuredClone(recipe.recipe.mash));
      setHasChanges(false);
    }
  }, [recipe?.recipe?.mash]);

  // Select different mash profile
  const handleSelectProfile = async (profileId: string) => {
    const profile = mashProfiles?.find((p) => p.id === profileId);
    if (!profile) return;

    await collection.update(id, (draft) => {
      draft.recipe.mash = structuredClone(profile.mashProfile);
      draft.updatedAt = Date.now();
    });
  };

  // Save changes
  const handleSave = async () => {
    if (!editedMash) return;

    await collection.update(id, (draft) => {
      draft.recipe.mash = structuredClone(editedMash);
      draft.updatedAt = Date.now();
    });
    setHasChanges(false);
    setIsEditing(false);
  };

  // Revert changes
  const handleRevert = () => {
    setEditedMash(
      recipe?.recipe?.mash ? structuredClone(recipe.recipe.mash) : null
    );
    setHasChanges(false);
  };

  // Update mash step
  const handleUpdateStep = (index: number, updates: Partial<MashStepType>) => {
    if (!editedMash) return;

    const newMash = { ...editedMash };
    newMash.mash_steps[index] = { ...newMash.mash_steps[index], ...updates };
    setEditedMash(newMash);
    setHasChanges(true);
  };

  // Add new mash step
  const handleAddStep = () => {
    if (!editedMash) return;

    const newStep: MashStepType = {
      name: "New Step",
      type: "infusion",
      step_temperature: { value: 152, unit: "F" },
      step_time: { value: 60, unit: "min" },
    };

    const newMash = { ...editedMash };
    newMash.mash_steps = [...newMash.mash_steps, newStep];
    setEditedMash(newMash);
    setHasChanges(true);
  };

  // Remove mash step
  const handleRemoveStep = (index: number) => {
    if (!editedMash) return;

    const newMash = { ...editedMash };
    newMash.mash_steps = newMash.mash_steps.filter((_, i) => i !== index);
    setEditedMash(newMash);
    setHasChanges(true);
  };

  // Update mash name
  const handleUpdateName = (name: string) => {
    if (!editedMash) return;
    setEditedMash({ ...editedMash, name });
    setHasChanges(true);
  };

  // Update grain temperature
  const handleUpdateGrainTemp = (value: number) => {
    if (!editedMash) return;
    setEditedMash({
      ...editedMash,
      grain_temperature: { ...editedMash.grain_temperature, value },
    });
    setHasChanges(true);
  };

  // Update grain temperature unit
  const handleUpdateGrainTempUnit = (unit: "C" | "F") => {
    if (!editedMash) return;
    setEditedMash({
      ...editedMash,
      grain_temperature: { ...editedMash.grain_temperature, unit },
    });
    setHasChanges(true);
  };

  // Update notes
  const handleUpdateNotes = (notes: string) => {
    if (!editedMash) return;
    setEditedMash({ ...editedMash, notes });
    setHasChanges(true);
  };

  if (!recipe?.recipe?.mash && !editedMash) {
    return (
      <Card className="p-6">
        <div className="text-center space-y-4">
          <p className="text-muted-foreground">
            No mash procedure set for this recipe.
          </p>
          {mashProfiles && mashProfiles.length > 0 && (
            <div className="space-y-2">
              <Label>Select a mash profile to get started:</Label>
              <Select onValueChange={handleSelectProfile}>
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Choose a mash profile..." />
                </SelectTrigger>
                <SelectContent>
                  {mashProfiles.map((profile) => (
                    <SelectItem key={profile.id} value={profile.id}>
                      {profile.mashProfile.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}
        </div>
      </Card>
    );
  }

  const currentMash = isEditing ? editedMash : recipe?.recipe?.mash;

  if (!currentMash) return null;

  return (
    <div className="space-y-4">
      {/* Header with Edit/Save buttons */}
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-semibold">Mash Procedure</h3>
        <div className="flex gap-2">
          {!isEditing ? (
            <Button onClick={() => setIsEditing(true)} variant="outline">
              <Edit className="h-4 w-4 mr-2" />
              Edit
            </Button>
          ) : (
            <>
              {hasChanges && (
                <Button onClick={handleRevert} variant="outline">
                  <RotateCcw className="h-4 w-4 mr-2" />
                  Revert
                </Button>
              )}
              <Button onClick={handleSave}>
                <Save className="h-4 w-4 mr-2" />
                Save
              </Button>
            </>
          )}
        </div>
      </div>

      {/* Mash profile selector */}
      {!isEditing && mashProfiles && mashProfiles.length > 0 && (
        <Card className="p-4">
          <div className="space-y-2">
            <Label>Change Mash Profile:</Label>
            <Select onValueChange={handleSelectProfile}>
              <SelectTrigger>
                <SelectValue placeholder="Select a different profile..." />
              </SelectTrigger>
              <SelectContent>
                {mashProfiles.map((profile) => (
                  <SelectItem key={profile.id} value={profile.id}>
                    {profile.mashProfile.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </Card>
      )}

      {/* Mash details */}
      <Card className="p-4 space-y-4">
        <div className="space-y-2">
          <Label>Name</Label>
          {isEditing ? (
            <Input
              value={currentMash.name}
              onChange={(e) => handleUpdateName(e.target.value)}
            />
          ) : (
            <p className="text-sm">{currentMash.name}</p>
          )}
        </div>

        <div className="space-y-2">
          <Label>Grain Temperature</Label>
          {isEditing ? (
            <div className="flex gap-2">
              <Input
                type="number"
                value={currentMash.grain_temperature.value}
                onChange={(e) =>
                  handleUpdateGrainTemp(parseFloat(e.target.value) || 0)
                }
                className="flex-1"
              />
              <Select
                value={currentMash.grain_temperature.unit}
                onValueChange={(val) => handleUpdateGrainTempUnit(val as "C" | "F")}
              >
                <SelectTrigger className="w-20">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="F">°F</SelectItem>
                  <SelectItem value="C">°C</SelectItem>
                </SelectContent>
              </Select>
            </div>
          ) : (
            <p className="text-sm">
              {currentMash.grain_temperature.value}°
              {currentMash.grain_temperature.unit}
            </p>
          )}
        </div>

        <div className="space-y-2">
          <Label>Notes</Label>
          {isEditing ? (
            <Textarea
              value={currentMash.notes || ""}
              onChange={(e) => handleUpdateNotes(e.target.value)}
              rows={3}
            />
          ) : (
            <p className="text-sm text-muted-foreground">
              {currentMash.notes || "No notes"}
            </p>
          )}
        </div>
      </Card>

      {/* Mash steps */}
      <div className="space-y-2">
        <h4 className="font-semibold">Mash Steps</h4>
        <MashStepList
          steps={currentMash.mash_steps}
          isEditing={isEditing}
          onUpdateStep={handleUpdateStep}
          onAddStep={isEditing ? handleAddStep : undefined}
          onRemoveStep={isEditing ? handleRemoveStep : undefined}
        />
      </div>
    </div>
  );
}
