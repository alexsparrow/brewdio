import { createRootRoute, Outlet, useRouterState } from "@tanstack/react-router";
import { useLiveQuery } from "@tanstack/react-db";
import { recipesCollection, equipmentCollection, DEFAULT_EQUIPMENT } from "@/db";
import { SidebarProvider, SidebarInset } from "@/components/ui/sidebar";
import { AppSidebar } from "@/components/app-sidebar";
import { ChatSidebar } from "@/components/chat-sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbList,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbSeparator,
  BreadcrumbPage,
} from "@/components/ui/breadcrumb";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { useEffect } from "react";
import { stores, wireCalculations } from "@/lib/calculate";
import { OG } from "@/calculations/og";
import { FG } from "@/calculations/fg";
import { ABV } from "@/calculations/abv";
import { Color } from "@/calculations/color";
import { IBU } from "@/calculations/ibu";

// Wire calculations at module level (runs once when module loads)
wireCalculations(stores, [
  OG,
  FG,
  ABV,
  Color,
  IBU,
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

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  const routerState = useRouterState();
  const { data: recipesData } = useLiveQuery(recipesCollection);
  const { data: equipmentData, status: equipmentStatus } = useLiveQuery(equipmentCollection);

  // Initialize default equipment if none exists
  useEffect(() => {
    console.log("Equipment initialization check:", {
      status: equipmentStatus,
      dataLength: equipmentData?.length,
      data: equipmentData,
      defaultEquipment: DEFAULT_EQUIPMENT
    });

    if (equipmentStatus === "ready" && equipmentData) {
      const defaultExists = equipmentData.some(e => e.id === "default");
      console.log("Default exists?", defaultExists);

      if (!defaultExists) {
        console.log("Inserting default equipment...");
        equipmentCollection.insert(DEFAULT_EQUIPMENT);
      }
    }
  }, [equipmentStatus, equipmentData]);

  // Extract recipeId from the route if we're on a recipe page
  const recipeId = routerState.location.pathname.startsWith('/recipes/')
    ? routerState.matches.find(match => match.params?.recipeId)?.params?.recipeId as string | undefined
    : undefined;

  // Find the current recipe
  const currentRecipe = recipeId ? recipesData?.find((r) => r.id === recipeId) : undefined;

  // Determine if we're on the JSON editor route
  const isJsonEditor = routerState.location.pathname.endsWith('/json');

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <header className="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-[[data-collapsible=icon]]/sidebar-wrapper:h-12">
          <div className="flex items-center gap-2 px-4">
            <SidebarTrigger className="-ml-1" />
            <Separator orientation="vertical" className="mr-2 h-4" />
            <Breadcrumb>
              <BreadcrumbList>
                <BreadcrumbItem className="hidden md:block">
                  <BreadcrumbLink to="/">Home</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator className="hidden md:block" />
                {currentRecipe ? (
                  <>
                    <BreadcrumbItem>
                      <BreadcrumbLink to={`/recipes/${recipeId}`}>
                        {currentRecipe.recipe.name}
                      </BreadcrumbLink>
                    </BreadcrumbItem>
                    {isJsonEditor && (
                      <>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem>
                          <BreadcrumbPage>JSON Editor</BreadcrumbPage>
                        </BreadcrumbItem>
                      </>
                    )}
                  </>
                ) : (
                  <BreadcrumbItem>
                    <BreadcrumbPage>Recipes</BreadcrumbPage>
                  </BreadcrumbItem>
                )}
              </BreadcrumbList>
            </Breadcrumb>
          </div>
        </header>
        <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
          <Outlet />
        </div>
      </SidebarInset>
      <ChatSidebar recipeId={recipeId} />
    </SidebarProvider>
  );
}
