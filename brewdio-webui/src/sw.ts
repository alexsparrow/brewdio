/// <reference lib="webworker" />

declare const self: ServiceWorkerGlobalScope;

const CACHE_NAME = "brewdio-v1";

const CACHEABLE_EXTENSIONS = [
  ".js",
  ".css",
  ".wasm",
  ".html",
  ".png",
  ".svg",
  ".ico",
  ".json",
  ".woff",
  ".woff2",
];

function isCacheable(url: URL): boolean {
  return CACHEABLE_EXTENSIONS.some((ext) => url.pathname.endsWith(ext));
}

self.addEventListener("install", () => {
  // Activate immediately — don't wait for existing tabs to close.
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  // Clean up old caches.
  event.waitUntil(
    caches.keys().then((names) =>
      Promise.all(
        names
          .filter((name) => name !== CACHE_NAME)
          .map((name) => caches.delete(name)),
      ),
    ),
  );
  // Take control of all open tabs immediately.
  self.clients.claim();
});

self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);

  // Skip non-GET requests and WebSocket upgrades.
  if (event.request.method !== "GET") return;
  if (url.protocol === "ws:" || url.protocol === "wss:") return;

  // Only cache static assets.
  if (!isCacheable(url)) return;

  event.respondWith(
    caches.match(event.request).then((cached) => {
      if (cached) return cached;

      return fetch(event.request).then((response) => {
        // Only cache successful responses.
        if (!response.ok) return response;

        const clone = response.clone();
        caches.open(CACHE_NAME).then((cache) => {
          cache.put(event.request, clone);
        });
        return response;
      });
    }),
  );
});

export {};
