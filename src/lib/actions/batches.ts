import { batchesCollection } from '@/db';

/**
 * Delete a batch by ID
 */
export async function deleteBatch(batchId: string): Promise<void> {
  await batchesCollection.delete(batchId);
}
