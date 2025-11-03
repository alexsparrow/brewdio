import type { EfficiencyType, FermentableAdditionType, RecipeType, VolumeType } from "@beerjson/beerjson";
import type { Path } from "@clickbar/dot-diver"

interface RuntimeState {
  fermentables: FermentableAdditionType[];
  recipe: RecipeType;
}

interface Calculation<T> {
    dependsOn: Path<RuntimeState>[];
    function: (...args: any[]) => T;
}

const OriginalGravity: Calculation<number> = {
    dependsOn: ["fermentables", "recipe.batch_size",  "recipe.efficiency.mash"],
    function: (fermentables:  FermentableAdditionType[], batchSize: VolumeType, efficiency: EfficiencyType) => {

    }
}