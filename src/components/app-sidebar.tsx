import * as React from "react";
import {
  Home,
  Settings2,
  Wheat,
  Leaf,
  FlaskConical,
  Code,
  Beaker,
  Droplets,
  FileText,
  GitBranch,
} from "lucide-react";
import { Link, useRouterState } from "@tanstack/react-router";
import { useLiveQuery } from "@tanstack/react-db";
import { recipesCollection, batchesCollection } from "@/db";

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
  originalRecipeId?: string;
}

function RecipeMenuItems({
  recipeName,
  baseRoute,
  params,
  showRecipeOverview = true,
  showJsonEditor = true,
  showBatches = true,
  originalRecipeId,
}: RecipeMenuItemsProps) {
  return (
    <>
      <Separator className="my-2" />
      <SidebarGroup>
        <SidebarGroupLabel>{recipeName}</SidebarGroupLabel>
        <SidebarMenu>
          {/* Original Recipe Link - Only show in batch mode */}
          {originalRecipeId && (
            <SidebarMenuItem>
              <SidebarMenuButton asChild className="text-primary">
                <Link to="/recipes/$recipeId" params={{ recipeId: originalRecipeId }}>
                  <GitBranch className="rotate-180" />
                  <span>Original Recipe</span>
                </Link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          )}

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

          {/* Batches - Only show in recipe mode */}
          {showBatches && (
            <SidebarMenuItem>
              <SidebarMenuButton asChild>
                <Link to={`${baseRoute}/batches`} params={params}>
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
                <Link to={`${baseRoute}/json`} params={params}>
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
  const { data: recipesData } = useLiveQuery(recipesCollection);
  const { data: batchesData } = useLiveQuery(batchesCollection);

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
          <>
            {/* Recipe Section (for editing the batch's recipe) */}
            <RecipeMenuItems
              recipeName={`${currentBatch.recipe.name} (batch copy)`}
              baseRoute="/batches/$batchId"
              params={{ batchId }}
              showRecipeOverview={true}
              showJsonEditor={true}
              showBatches={false}
              originalRecipeId={currentBatch.recipeId}
            />

            {/* Batch Section (batch-specific items) */}
            <Separator className="my-2" />
            <SidebarGroup>
              <SidebarGroupLabel>{currentBatch.name}</SidebarGroupLabel>
              <SidebarMenu>
                {/* Batch Overview */}
                <SidebarMenuItem>
                  <SidebarMenuButton asChild>
                    <Link to="/batches/$batchId/overview" params={{ batchId }}>
                      <FileText />
                      <span>Batch Overview</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              </SidebarMenu>
            </SidebarGroup>
          </>
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
            <ThemeToggle />
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
