import * as React from "react";
import {
  Home,
  Settings2,
  Wheat,
  Leaf,
  FlaskConical,
  Code,
  Beaker,
  Flame,
  Droplets,
} from "lucide-react";
import { Link, useRouterState } from "@tanstack/react-router";
import { useBatches } from "@/lib/db/batches";
import { useRecipes } from "@/lib/db/recipes";

import { NavMain } from "@/components/nav-main";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import { ThemeToggle } from "@/components/theme-toggle";
import { SyncStatus } from "@/components/sync-status";
import { getVersion } from "brewdio-wasm";

// Static navigation items
const staticNavItems = [
  {
    title: "Home",
    url: "/",
    icon: Home,
  },
  {
    title: "Brewery",
    url: "/brewery",
    icon: Beaker,
  },
  {
    title: "Settings",
    url: "/settings",
    icon: Settings2,
  },
];

// Context-dependent navigation for recipe view
const recipeContextNav = [
  {
    title: "Fermentables",
    url: "#fermentables",
    icon: Wheat,
  },
  {
    title: "Hops",
    url: "#hops",
    icon: Leaf,
  },
  {
    title: "Cultures (Yeast)",
    url: "#cultures",
    icon: FlaskConical,
  },
];

// Shared Recipe Menu Component
interface RecipeMenuItemsProps {
  recipeName: string;
  baseRoute: string;
  params: Record<string, string>;
  showRecipeOverview?: boolean;
  showJsonEditor?: boolean;
  showBatches?: boolean;
}

function RecipeMenuItems({
  recipeName,
  baseRoute,
  params,
  showRecipeOverview = true,
  showJsonEditor = true,
  showBatches = true,
}: RecipeMenuItemsProps) {
  return (
    <>
      <Separator className="my-2" />
      <SidebarGroup>
        <SidebarGroupLabel>{recipeName}</SidebarGroupLabel>
        <SidebarMenu>
          {/* Recipe Overview */}
          {showRecipeOverview && (
            <SidebarMenuItem>
              <SidebarMenuButton asChild>
                <Link to={baseRoute} params={params}>
                  <Home />
                  <span>Recipe Overview</span>
                </Link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          )}

          {/* Recipe Sections */}
          {recipeContextNav.map((item) => (
            <SidebarMenuItem key={item.title}>
              <SidebarMenuButton asChild>
                <Link to={baseRoute} params={params} hash={item.url}>
                  <item.icon />
                  <span>{item.title}</span>
                </Link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          ))}

          {/* Mash */}
          <SidebarMenuItem>
            <SidebarMenuButton asChild>
              <Link to={`${baseRoute}/mash` as string} params={params}>
                <Flame />
                <span>Mash</span>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>

          {/* Water */}
          <SidebarMenuItem>
            <SidebarMenuButton asChild>
              <Link to={`${baseRoute}/water` as string} params={params}>
                <Droplets />
                <span>Water</span>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>

          {/* Batches - Only show in recipe mode */}
          {showBatches && (
            <SidebarMenuItem>
              <SidebarMenuButton asChild>
                <Link to={`${baseRoute}/batches` as string} params={params}>
                  <Beaker />
                  <span>Batches</span>
                </Link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          )}

          {/* JSON Editor */}
          {showJsonEditor && (
            <SidebarMenuItem>
              <SidebarMenuButton asChild>
                <Link to={`${baseRoute}/json` as string} params={params}>
                  <Code />
                  <span>JSON Editor</span>
                </Link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          )}
        </SidebarMenu>
      </SidebarGroup>
    </>
  );
}

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const router = useRouterState();
  const isRecipeView = router.location.pathname.startsWith("/recipes/");
  const isBatchView = router.location.pathname.startsWith("/batches/");
  const { data: recipesData } = useRecipes();
  const { data: batchesData } = useBatches();

  // Extract recipe ID from pathname
  const recipeId = React.useMemo(() => {
    const match = router.location.pathname.match(/^\/recipes\/([^/]+)/);
    return match ? match[1] : null;
  }, [router.location.pathname]);

  // Extract batch ID from pathname
  const batchId = React.useMemo(() => {
    const match = router.location.pathname.match(/^\/batches\/([^/]+)/);
    return match ? match[1] : null;
  }, [router.location.pathname]);

  // Get current recipe
  const currentRecipe = React.useMemo(() => {
    if (!recipeId || !recipesData) return null;
    return recipesData.find((r) => r.id === recipeId);
  }, [recipeId, recipesData]);

  // Get current batch
  const currentBatch = React.useMemo(() => {
    if (!batchId || !batchesData) return null;
    return batchesData.find((b) => b.id === batchId);
  }, [batchId, batchesData]);

  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              asChild
              className="data-[slot=sidebar-menu-button]:!p-1.5"
            >
              <Link to="/">
                <span className="text-base font-semibold">brewdio.</span>
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        {/* Static Navigation */}
        <NavMain items={staticNavItems} />

        {/* Context-dependent sections for Batch View */}
        {isBatchView && batchId && currentBatch && (
          <RecipeMenuItems
            recipeName={currentBatch.name}
            baseRoute="/batches/$batchId"
            params={{ batchId }}
            showRecipeOverview={true}
            showJsonEditor={true}
            showBatches={false}
          />
        )}

        {/* Context-dependent sections for Recipe View */}
        {isRecipeView && recipeId && currentRecipe && (
          <RecipeMenuItems
            recipeName={currentRecipe.recipe.name}
            baseRoute="/recipes/$recipeId"
            params={{ recipeId }}
            showRecipeOverview={true}
            showJsonEditor={true}
          />
        )}
      </SidebarContent>
      <SidebarFooter>
        <SidebarMenu>
          <SidebarMenuItem>
            <SyncStatus />
          </SidebarMenuItem>
          <SidebarMenuItem>
            <ThemeToggle />
          </SidebarMenuItem>
          <SidebarMenuItem>
            <span className="px-2 py-1 text-xs text-muted-foreground">{getVersion()}</span>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
