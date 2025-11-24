import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/batches/$batchId_/water")({
  component: BatchWaterComponent,
});

function BatchWaterComponent() {
  return (
    <div className="flex flex-col gap-6">
      <div>
        <h2 className="text-2xl font-bold mb-2">Water Calculator</h2>
        <p className="text-muted-foreground">
          Coming soon: Water chemistry calculator for this batch.
        </p>
      </div>
    </div>
  );
}
