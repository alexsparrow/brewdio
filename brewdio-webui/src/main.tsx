import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import initWasm, { RecipeDb, initPersistentStorage } from "brewdio-wasm";
import { initRecipeDb, registerChangeCallback } from "@/lib/db/recipes";
import { startSync } from "@/lib/sync";
import "./index.css";

// Import the generated route tree
import { routeTree } from "./routeTree.gen";

// Create a new router instance
const router = createRouter({ routeTree });

// Register the router instance for type safety
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

const queryClient = new QueryClient();

// Initialize WASM module, install persistent VFS, then open the database.
async function init() {
  // Ensure WASM binary is loaded and initialized.
  await initWasm();

  try {
    await initPersistentStorage();
    console.log("[brewdio] Persistent storage initialized (IndexedDB VFS)");
  } catch (e) {
    console.warn("[brewdio] Failed to initialize persistent storage, falling back to in-memory:", e);
  }

  // Open a named database — persists to IndexedDB if VFS was installed successfully.
  const db = RecipeDb.open("brewdio.db");
  initRecipeDb(db);
  registerChangeCallback(queryClient);

  const syncServer = localStorage.getItem("brewdio_server");
  if (syncServer) startSync(syncServer);

  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </StrictMode>
  );
}

init();
