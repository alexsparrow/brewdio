import { createRootRoute, Outlet, useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback } from "react";
import { useBatches } from "@/lib/db/batches";
import { useRecipes } from "@/lib/db/recipes";
import { SidebarProvider, SidebarInset, useSidebar } from "@/components/ui/sidebar";
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
import { stores, wireCalculations } from "@/lib/calculate";
import { OG, FG, ABV, Color, IBU } from "@/lib/calculations";
import { KeyboardShortcutsProvider } from "@/components/keyboard-shortcuts-provider";
import { CommandPalette } from "@/components/command-palette";
import { KeyboardShortcutsDialog } from "@/components/keyboard-shortcuts-dialog";
import { useCommandHandler } from "@/hooks/use-command";

export const Route = createRootRoute({
  component: RootComponent,
});

let calculationsWired = false;

function getSidebarDefault(): boolean {
  const match = document.cookie.match(/(?:^|;\s*)sidebar_state=(\w+)/);
  return match ? match[1] === "true" : true;
}

function RootComponent() {
  // Wire calculations on first render (WASM is initialized by this point)
  if (!calculationsWired) {
    calculationsWired = true;
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
  }

  const routerState = useRouterState();
  const { data: recipesData } = useRecipes();
  const { data: batchesData } = useBatches();

  // Extract recipeId from the route if we're on a recipe page
  const matchParams = (match: { params?: Record<string, unknown> }) => match.params as Record<string, string> | undefined;
  const recipeId = routerState.location.pathname.startsWith('/recipes/')
    ? matchParams(routerState.matches.find(match => matchParams(match)?.recipeId) ?? {})?.recipeId
    : undefined;

  // Extract batchId from the route if we're on a batch page
  const batchId = routerState.location.pathname.startsWith('/batches/')
    ? matchParams(routerState.matches.find(match => matchParams(match)?.batchId) ?? {})?.batchId
    : undefined;

  // Find the current recipe or batch
  const currentRecipe = recipeId ? recipesData?.find((r) => r.id === recipeId) : undefined;
  const currentBatch = batchId ? batchesData?.find((b) => b.id === batchId) : undefined;

  // Look up the original recipe name for batch breadcrumbs
  const batchOriginalRecipe = currentBatch
    ? recipesData?.find((r) => r.id === currentBatch.recipeId)
    : undefined;
  const batchRecipeName = batchOriginalRecipe?.recipe.name ?? currentBatch?.recipe.name;

  // Determine if we're on the JSON editor route
  const isJsonEditor = routerState.location.pathname.endsWith('/json');

  return (
    <SidebarProvider defaultOpen={getSidebarDefault()}>
      <GlobalCommandHandlers />
      <KeyboardShortcutsProvider />
      <CommandPalette />
      <KeyboardShortcutsDialog />
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
                {currentBatch ? (
                  <>
                    <BreadcrumbSeparator className="hidden md:block" />
                    <BreadcrumbItem>
                      <BreadcrumbLink to={`/recipes/${currentBatch.recipeId}`}>
                        {batchRecipeName}
                      </BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator />
                    <BreadcrumbItem>
                      <BreadcrumbPage>{currentBatch.name}</BreadcrumbPage>
                    </BreadcrumbItem>
                  </>
                ) : currentRecipe ? (
                  <>
                    <BreadcrumbSeparator className="hidden md:block" />
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
                ) : null}
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

function GlobalCommandHandlers() {
  const navigate = useNavigate();
  const { toggleSidebar } = useSidebar();

  useCommandHandler("sidebar.toggle", toggleSidebar);
  useCommandHandler(
    "navigate.home",
    useCallback(() => navigate({ to: "/" }), [navigate])
  );
  useCommandHandler(
    "navigate.settings",
    useCallback(() => navigate({ to: "/settings" }), [navigate])
  );
  useCommandHandler(
    "theme.toggle",
    useCallback(() => {
      const html = document.documentElement;
      const isDark = html.classList.contains("dark");
      const newTheme = isDark ? "light" : "dark";
      html.classList.toggle("dark", newTheme === "dark");
      localStorage.setItem("theme", newTheme);
    }, [])
  );

  return null;
}
