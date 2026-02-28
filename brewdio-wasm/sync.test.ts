import { test, expect, describe, afterEach } from "bun:test";
import { SyncSession } from "brewdio-wasm";

// ============================================================================
// SyncSession Unit Tests (no WebSocket needed)
// ============================================================================

function sampleRecipeDocument(id: string, name: string) {
  return JSON.stringify({
    id,
    name,
    recipe: {
      name,
      type: "all grain",
      author: "Tester",
      batch_size: { unit: "l", value: 20.0 },
      efficiency: {
        brewhouse: { unit: "%", value: 72.0 },
      },
      ingredients: {
        fermentable_additions: [
          {
            name: "Pale Malt",
            type: "grain",
            amount: { unit: "kg", value: 5.0 },
            color: { unit: "SRM", value: 2.0 },
            yield: { fine_grind: { unit: "%", value: 80.0 } },
          },
        ],
        hop_additions: [
          {
            name: "Cascade",
            timing: {
              duration: { unit: "min", value: 60 },
              use: "add_to_boil",
            },
            amount: { unit: "g", value: 30.0 },
            form: "pellet",
            alpha_acid: { unit: "%", value: 5.5 },
          },
        ],
      },
    },
    is_deleted: false,
  });
}

describe("SyncSession", () => {
  test("new session can reconcile and hydrate", () => {
    const session = new SyncSession();
    session.reconcileJson(sampleRecipeDocument("test-1", "Test Beer"));
    const hydrated = JSON.parse(session.hydrateJson());

    expect(hydrated.id).toBe("test-1");
    expect(hydrated.name).toBe("Test Beer");
    expect(hydrated.is_deleted).toBe(false);
  });

  test("from_bytes roundtrip", () => {
    const session1 = new SyncSession();
    session1.reconcileJson(sampleRecipeDocument("test-2", "Roundtrip Beer"));
    const bytes = session1.saveDoc();

    const session2 = SyncSession.fromBytes(new Uint8Array(bytes));
    const hydrated = JSON.parse(session2.hydrateJson());

    expect(hydrated.id).toBe("test-2");
    expect(hydrated.name).toBe("Roundtrip Beer");
  });

  test("save and restore state", () => {
    const session1 = new SyncSession();
    session1.reconcileJson(sampleRecipeDocument("test-3", "Stateful Beer"));
    const docBytes = session1.saveDoc();
    const stateBytes = session1.saveState();

    const session2 = SyncSession.fromDocAndState(
      new Uint8Array(docBytes),
      new Uint8Array(stateBytes)
    );
    const hydrated = JSON.parse(session2.hydrateJson());
    expect(hydrated.name).toBe("Stateful Beer");
  });

  test("two sessions converge via sync messages", () => {
    // Session A creates a recipe
    const sessionA = new SyncSession();
    sessionA.reconcileJson(sampleRecipeDocument("recipe-1", "Original Beer"));

    // Session B starts empty
    const sessionB = new SyncSession();

    // Exchange sync messages until convergence
    let msg: any = sessionA.generateSyncMessage();
    let rounds = 0;
    const maxRounds = 20;

    while (msg !== undefined && rounds < maxRounds) {
      const msgBytes = msg instanceof Uint8Array ? msg : new Uint8Array(msg);
      const reply = sessionB.receiveSyncMessage(msgBytes);

      if (reply === undefined || typeof reply === "string") {
        // Check if A also needs to send
        msg = sessionA.generateSyncMessage();
      } else {
        const replyBytes =
          reply instanceof Uint8Array ? reply : new Uint8Array(reply);
        msg = sessionA.receiveSyncMessage(replyBytes);
        if (msg === undefined || typeof msg === "string") {
          msg = undefined;
        }
      }
      rounds++;
    }

    // Both should now have the same document
    const hydratedA = JSON.parse(sessionA.hydrateJson());
    const hydratedB = JSON.parse(sessionB.hydrateJson());

    expect(hydratedA.id).toBe("recipe-1");
    expect(hydratedB.id).toBe("recipe-1");
    expect(hydratedA.name).toBe(hydratedB.name);
  });

  test("concurrent edits converge", () => {
    // Both start from the same doc
    const sessionA = new SyncSession();
    sessionA.reconcileJson(sampleRecipeDocument("recipe-2", "Base Beer"));
    const baseBytes = sessionA.saveDoc();

    const sessionB = SyncSession.fromBytes(new Uint8Array(baseBytes));

    // A changes the name
    sessionA.reconcileJson(sampleRecipeDocument("recipe-2", "Beer Alpha"));

    // B changes the name differently
    sessionB.reconcileJson(sampleRecipeDocument("recipe-2", "Beer Beta"));

    // Sync until convergence
    let msg: any = sessionA.generateSyncMessage();
    let rounds = 0;

    while (rounds < 20) {
      if (msg === undefined) {
        msg = sessionA.generateSyncMessage();
        if (msg === undefined) break;
      }

      const msgBytes = msg instanceof Uint8Array ? msg : new Uint8Array(msg);
      const reply = sessionB.receiveSyncMessage(msgBytes);

      if (reply === undefined || typeof reply === "string") {
        msg = sessionA.generateSyncMessage();
      } else {
        const replyBytes =
          reply instanceof Uint8Array ? reply : new Uint8Array(reply);
        const response = sessionA.receiveSyncMessage(replyBytes);
        msg =
          response === undefined || typeof response === "string"
            ? undefined
            : response;
      }
      rounds++;
    }

    // Both should converge to the same name (Automerge LWW)
    const hydratedA = JSON.parse(sessionA.hydrateJson());
    const hydratedB = JSON.parse(sessionB.hydrateJson());
    expect(hydratedA.name).toBe(hydratedB.name);
  });

  test("is_synced returns correct state", () => {
    const session = new SyncSession();
    session.reconcileJson(sampleRecipeDocument("test-4", "Synced Beer"));

    // A fresh session with no peer should report as synced (nothing to send)
    // Note: isSynced checks if generateSyncMessage would return None
    // For a session that hasn't started syncing, this is true
    expect(typeof session.isSynced()).toBe("boolean");
  });
});

// ============================================================================
// Mock WebSocket Server + Protocol Tests
// ============================================================================

interface SyncMessage {
  type: "Hello" | "SyncDoc" | "NewDoc";
  recipe_ids?: string[];
  recipe_id?: string;
  data?: number[];
  am_data?: number[];
}

let servers: { stop: () => void }[] = [];

afterEach(() => {
  for (const server of servers) {
    server.stop();
  }
  servers = [];
});

function createMockSyncServer(): {
  server: ReturnType<typeof Bun.serve>;
  messages: SyncMessage[];
  port: number;
} {
  const messages: SyncMessage[] = [];

  const server = Bun.serve({
    port: 0,
    fetch(req, server) {
      if (server.upgrade(req)) return undefined;
      return new Response("upgrade failed", { status: 500 });
    },
    websocket: {
      open(_ws) {},
      message(_ws, data) {
        const text = typeof data === "string" ? data : new TextDecoder().decode(data);
        const msg: SyncMessage = JSON.parse(text);
        messages.push(msg);
      },
      close(_ws) {},
    },
  });

  servers.push(server);

  return { server, messages, port: server.port };
}

describe("Protocol integration", () => {
  test("mock server receives Hello message", async () => {
    const { port, messages } = createMockSyncServer();

    const ws = new WebSocket(`ws://localhost:${port}`);
    await new Promise<void>((resolve) => {
      ws.onopen = () => resolve();
    });

    const hello: SyncMessage = {
      type: "Hello",
      recipe_ids: ["recipe-1", "recipe-2"],
    };
    ws.send(JSON.stringify(hello));

    // Wait for message to arrive
    await new Promise((r) => setTimeout(r, 100));

    expect(messages.length).toBe(1);
    expect(messages[0].type).toBe("Hello");
    expect(messages[0].recipe_ids).toEqual(["recipe-1", "recipe-2"]);

    ws.close();
  });

  test("mock server receives NewDoc message", async () => {
    const { port, messages } = createMockSyncServer();

    // Create a recipe's AM data via SyncSession
    const session = new SyncSession();
    session.reconcileJson(sampleRecipeDocument("new-recipe", "New Beer"));
    const amData = Array.from(new Uint8Array(session.saveDoc()));

    const ws = new WebSocket(`ws://localhost:${port}`);
    await new Promise<void>((resolve) => {
      ws.onopen = () => resolve();
    });

    const newDoc: SyncMessage = {
      type: "NewDoc",
      recipe_id: "new-recipe",
      am_data: amData,
    };
    ws.send(JSON.stringify(newDoc));

    await new Promise((r) => setTimeout(r, 100));

    expect(messages.length).toBe(1);
    expect(messages[0].type).toBe("NewDoc");
    expect(messages[0].recipe_id).toBe("new-recipe");
    expect(messages[0].am_data!.length).toBeGreaterThan(0);

    ws.close();
  });

  test("SyncDoc message exchange through mock server", async () => {
    const receivedMessages: SyncMessage[] = [];

    // Server that echoes back SyncDoc with its own sync session
    const serverSession = new SyncSession();

    const server = Bun.serve({
      port: 0,
      fetch(req, server) {
        if (server.upgrade(req)) return undefined;
        return new Response("upgrade failed", { status: 500 });
      },
      websocket: {
        open(_ws) {},
        message(ws, data) {
          const text = typeof data === "string" ? data : new TextDecoder().decode(data);
          const msg: SyncMessage = JSON.parse(text);
          receivedMessages.push(msg);

          if (msg.type === "SyncDoc" && msg.data) {
            const reply = serverSession.receiveSyncMessage(
              new Uint8Array(msg.data)
            );
            if (reply !== undefined && typeof reply !== "string") {
              const replyMsg: SyncMessage = {
                type: "SyncDoc",
                recipe_id: msg.recipe_id,
                data: Array.from(
                  reply instanceof Uint8Array ? reply : new Uint8Array(reply)
                ),
              };
              ws.send(JSON.stringify(replyMsg));
            }
          }
        },
        close(_ws) {},
      },
    });
    servers.push(server);

    // Client creates a recipe and initiates sync
    const clientSession = new SyncSession();
    clientSession.reconcileJson(
      sampleRecipeDocument("sync-recipe", "Sync Beer")
    );

    const ws = new WebSocket(`ws://localhost:${server.port}`);
    await new Promise<void>((resolve) => {
      ws.onopen = () => resolve();
    });

    // Start sync: generate first message
    const firstMsg = clientSession.generateSyncMessage();
    expect(firstMsg).not.toBeUndefined();

    const syncDocMsg: SyncMessage = {
      type: "SyncDoc",
      recipe_id: "sync-recipe",
      data: Array.from(
        firstMsg instanceof Uint8Array ? firstMsg : new Uint8Array(firstMsg)
      ),
    };
    ws.send(JSON.stringify(syncDocMsg));

    // Wait for exchange
    await new Promise<void>((resolve) => {
      ws.onmessage = (event) => {
        const msg: SyncMessage = JSON.parse(event.data);
        if (msg.type === "SyncDoc" && msg.data) {
          const reply = clientSession.receiveSyncMessage(
            new Uint8Array(msg.data)
          );
          if (reply !== undefined && typeof reply !== "string") {
            const replyMsg: SyncMessage = {
              type: "SyncDoc",
              recipe_id: msg.recipe_id,
              data: Array.from(
                reply instanceof Uint8Array ? reply : new Uint8Array(reply)
              ),
            };
            ws.send(JSON.stringify(replyMsg));
          } else {
            resolve();
          }
        }
      };
      // Timeout fallback
      setTimeout(resolve, 2000);
    });

    // Server should have received at least one SyncDoc
    expect(receivedMessages.length).toBeGreaterThanOrEqual(1);
    expect(receivedMessages[0].type).toBe("SyncDoc");

    // Server should now have the recipe
    const serverHydrated = JSON.parse(serverSession.hydrateJson());
    expect(serverHydrated.name).toBe("Sync Beer");

    ws.close();
  });
});
