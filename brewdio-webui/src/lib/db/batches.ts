import { useQuery, type QueryClient } from '@tanstack/react-query';
import type { RecipeType, EquipmentType } from 'brewdio-wasm';
import { getRecipeDb } from './recipes';

// ---------------------------------------------------------------------------
// Types — matches the old BatchDocument shape for consumer compatibility
// ---------------------------------------------------------------------------

export interface BatchDocument {
  id: string;
  name: string;
  recipeId: string;
  equipmentId: string;
  recipe: RecipeType;
  equipment: EquipmentType;
  brewDate: number;
  notes?: string;
  createdAt: number;
  updatedAt: number;
}

/** Raw shape returned by the WASM layer */
interface BatchRaw {
  id: string;
  name: string;
  recipeId: string;
  data: {
    recipeId?: string;
    equipmentId: string;
    recipe: RecipeType;
    equipment: EquipmentType;
    brewDate: number;
    notes?: string;
    createdAt: number;
    updatedAt: number;
  };
}

function rawToBatchDoc(raw: BatchRaw): BatchDocument {
  return {
    id: raw.id,
    name: raw.name,
    recipeId: raw.recipeId,
    equipmentId: raw.data.equipmentId,
    recipe: raw.data.recipe,
    equipment: raw.data.equipment,
    brewDate: raw.data.brewDate,
    notes: raw.data.notes,
    createdAt: raw.data.createdAt,
    updatedAt: raw.data.updatedAt,
  };
}

function batchDocToData(doc: Omit<BatchDocument, 'id' | 'name' | 'recipeId'>) {
  return {
    equipmentId: doc.equipmentId,
    recipe: doc.recipe,
    equipment: doc.equipment,
    brewDate: doc.brewDate,
    notes: doc.notes,
    createdAt: doc.createdAt,
    updatedAt: doc.updatedAt,
  };
}

// ---------------------------------------------------------------------------
// Query keys
// ---------------------------------------------------------------------------

export const batchKeys = {
  all: ['batches'] as const,
  detail: (id: string) => ['batches', id] as const,
};

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

export function useBatches() {
  return useQuery<BatchDocument[]>({
    queryKey: batchKeys.all,
    queryFn: () => {
      const db = getRecipeDb();
      const raws = db.listBatches() as unknown as BatchRaw[];
      return raws.map(rawToBatchDoc);
    },
  });
}

export function useBatch(id: string) {
  return useQuery<BatchDocument | null>({
    queryKey: batchKeys.detail(id),
    queryFn: () => {
      const db = getRecipeDb();
      const raw = db.getBatch(id) as unknown as BatchRaw | null;
      return raw ? rawToBatchDoc(raw) : null;
    },
  });
}

// ---------------------------------------------------------------------------
// Imperative helpers
// ---------------------------------------------------------------------------

export function createBatchImperative(
  name: string,
  recipeId: string,
  data: Omit<BatchDocument, 'id' | 'name' | 'recipeId'>,
): string {
  const db = getRecipeDb();
  return db.createBatch(name, recipeId, batchDocToData(data));
}

export function updateBatchImperative(
  id: string,
  name: string,
  data: Omit<BatchDocument, 'id' | 'name' | 'recipeId'>,
): void {
  const db = getRecipeDb();
  db.updateBatch(id, name, batchDocToData(data));
}

export function deleteBatchImperative(id: string): void {
  const db = getRecipeDb();
  db.deleteBatch(id);
}

export function listBatchesImperative(): BatchDocument[] {
  const db = getRecipeDb();
  const raws = db.listBatches() as unknown as BatchRaw[];
  return raws.map(rawToBatchDoc);
}

// ---------------------------------------------------------------------------
// Cache invalidation helper
// ---------------------------------------------------------------------------

export function invalidateBatches(queryClient: QueryClient) {
  queryClient.invalidateQueries({ queryKey: batchKeys.all });
}
