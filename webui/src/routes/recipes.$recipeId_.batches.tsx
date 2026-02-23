import { createFileRoute, Link } from "@tanstack/react-router";
import { useLiveQuery } from "@tanstack/react-db";
import { recipesCollection, batchesCollection } from "@/db";
import { Button } from "@/components/ui/button";
import { Beaker, Calendar, ExternalLink } from "lucide-react";
import { DeleteBatchDialog } from "@/components/delete-batch-dialog";

export const Route = createFileRoute("/recipes/$recipeId_/batches")({
  component: RecipeBatchesComponent,
});

function RecipeBatchesComponent() {
  const { recipeId } = Route.useParams();
  const { data: recipes } = useLiveQuery(recipesCollection);
  const { data: batches } = useLiveQuery(batchesCollection);

  const recipe = recipes?.find((r) => r.id === recipeId);
  const recipeBatches = batches?.filter((b) => b.recipeId === recipeId) || [];

  // Sort batches by brew date (most recent first)
  const sortedBatches = [...recipeBatches].sort((a, b) => b.brewDate - a.brewDate);

  if (!recipe) {
    return <div>Recipe not found</div>;
  }

  // Format date helper
  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  };

  return (
    <div className="flex flex-col gap-6">
      <div>
        <h2 className="text-2xl font-bold mb-2">Batches</h2>
        <p className="text-sm text-muted-foreground">
          All batches brewed from {recipe.recipe.name}
        </p>
      </div>

      {sortedBatches.length === 0 ? (
        <div className="border rounded-lg p-8 text-center">
          <Beaker className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
          <h3 className="text-lg font-semibold mb-2">No batches yet</h3>
          <p className="text-muted-foreground mb-4">
            You haven't brewed this recipe yet. Start by clicking the Brew button.
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {sortedBatches.map((batch) => (
            <div key={batch.id} className="border rounded-lg p-4">
              <div className="flex items-start justify-between">
                <Link
                  to="/batches/$batchId"
                  params={{ batchId: batch.id }}
                  className="flex-1 hover:opacity-80 transition-opacity"
                >
                  <div className="flex items-center gap-2 mb-1">
                    <h3 className="text-lg font-semibold">{batch.name}</h3>
                    <ExternalLink className="h-4 w-4 text-muted-foreground" />
                  </div>
                  <div className="flex items-center gap-4 text-sm text-muted-foreground">
                    <div className="flex items-center gap-1">
                      <Calendar className="h-4 w-4" />
                      <span>Brewed on {formatDate(batch.brewDate)}</span>
                    </div>
                    {batch.equipment && (
                      <span>• {batch.equipment.name}</span>
                    )}
                  </div>
                  {batch.notes && (
                    <p className="mt-2 text-sm text-muted-foreground line-clamp-2">
                      {batch.notes}
                    </p>
                  )}
                </Link>
                <div className="ml-4 flex items-start gap-3">
                  <div className="text-right">
                    <div className="text-sm font-medium">
                      {batch.recipe.batch_size.value} {batch.recipe.batch_size.unit}
                    </div>
                    {batch.recipe.style && (
                      <div className="text-xs text-muted-foreground mt-1">
                        {batch.recipe.style.name}
                      </div>
                    )}
                  </div>
                  <div onClick={(e) => e.stopPropagation()}>
                    <DeleteBatchDialog
                      batchId={batch.id}
                      batchName={batch.name}
                      variant="destructive"
                      size="sm"
                      redirectOnDelete={false}
                    />
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {sortedBatches.length > 0 && (
        <div className="text-sm text-muted-foreground text-center">
          Total: {sortedBatches.length} batch{sortedBatches.length === 1 ? '' : 'es'}
        </div>
      )}
    </div>
  );
}
