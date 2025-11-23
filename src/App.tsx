import { useEffect } from "react";
import Layout from "./app/layout";
import { stores, wireCalculations } from "./lib/calculate";
import { OG } from "./calculations/og";

function App() {
  useEffect(() => {
    wireCalculations(stores, [
      OG,
      {
        type: "dynamic",
        dependsOn: ["recipe.ingredients.fermentable_additions", "recipe.batch_size"],
        expr: `(fermentables, batchSize) => {
        const colors = fermentables.map((fermentable) => fermentable.color?.value);

        return batchSize.value + colors.filter((color) => color).length + 0.5;
      }`,
        id: "fermentable_colors",
      },
      {
        type: "dynamic",
        dependsOn: ["calculations.fermentable_colors", "recipe.batch_size"],
        expr: `(fermentable_colors, batchSize) => {
        return fermentable_colors + batchSize.value;
      }`,
        id: "my_var",
      },
    ]);
  });
  return (
    <>
      <Layout />
    </>
  );
}

export default App;
