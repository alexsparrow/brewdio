import * as React from "react";
import {
  Home,
  Settings2,
  Wheat,
  Leaf,
  FlaskConical,
  Code,
  Beaker,
} from "lucide-react";
import { Link, useRouterState } from "@tanstack/react-router";
import { useLiveQuery } from "@tanstack/react-db";
import { recipesCollection } from "@/db";

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

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const router = useRouterState();
  const isRecipeView = router.location.pathname.startsWith("/recipes/");
  const { data: recipesData } = useLiveQuery(recipesCollection);

  // Extract recipe ID from pathname
  const recipeId = React.useMemo(() => {
    const match = router.location.pathname.match(/^\/recipes\/([^/]+)/);
    return match ? match[1] : null;
  }, [router.location.pathname]);

  // Get current recipe
  const currentRecipe = React.useMemo(() => {
    if (!recipeId || !recipesData) return null;
    return recipesData.find((r) => r.id === recipeId);
  }, [recipeId, recipesData]);

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

        {/* Context-dependent sections */}
        {isRecipeView && recipeId && currentRecipe && (
          <>
            <Separator className="my-2" />
            <SidebarGroup>
              <SidebarGroupLabel>{currentRecipe.recipe.name}</SidebarGroupLabel>
              <SidebarMenu>
                <SidebarMenuItem>
                  <SidebarMenuButton asChild>
                    <Link to="/recipes/$recipeId" params={{ recipeId }}>
                      <Home />
                      <span>Recipe Overview</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              </SidebarMenu>
            </SidebarGroup>

            <Separator className="my-2" />
            <SidebarGroup>
              <SidebarGroupLabel>Recipe Sections</SidebarGroupLabel>
              <SidebarMenu>
                {recipeContextNav.map((item) => (
                  <SidebarMenuItem key={item.title}>
                    <SidebarMenuButton asChild>
                      <Link
                        to="/recipes/$recipeId"
                        params={{ recipeId }}
                        hash={item.url}
                      >
                        <item.icon />
                        <span>{item.title}</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </SidebarGroup>

            <Separator className="my-2" />
            <SidebarGroup>
              <SidebarGroupLabel>Advanced</SidebarGroupLabel>
              <SidebarMenu>
                <SidebarMenuItem>
                  <SidebarMenuButton asChild>
                    <Link to="/recipes/$recipeId/json" params={{ recipeId }}>
                      <Code />
                      <span>JSON Editor</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              </SidebarMenu>
            </SidebarGroup>
          </>
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
