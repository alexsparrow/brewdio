import { useEffect } from "react";
import Layout from "./app/layout";
import { stores, wireCalculations } from "./lib/calculate";

function App() {
  useEffect(() => {
    wireCalculations(stores, [
      {
        dependsOn: ["fermentables", "recipe.batch_size"],
        expr: `(fermentables, batchSize) => {
        const colors = fermentables.map((fermentable) => fermentable.color?.value);

        return batchSize.value + colors.filter((color) => color).length + 0.5;
      }`,
        id: "fermentable_colors",
      },
      {
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
