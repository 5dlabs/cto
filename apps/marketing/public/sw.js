// Service Worker — cache-first for all static assets
// Updates automatically when new content is deployed

const CACHE_NAME = "cto-v1";

// Cache-first: serve from cache instantly, fetch update in background
self.addEventListener("fetch", (event) => {
  // Skip non-GET and chrome-extension requests
  if (event.request.method !== "GET") return;
  const url = new URL(event.request.url);
  if (!url.protocol.startsWith("http")) return;

  event.respondWith(
    caches.open(CACHE_NAME).then((cache) =>
      cache.match(event.request).then((cached) => {
        const fetched = fetch(event.request).then((response) => {
          // Only cache successful responses
          if (response.ok) cache.put(event.request, response.clone());
          return response;
        });
        // Return cached immediately, update in background
        return cached || fetched;
      })
    )
  );
});

// Activate immediately, claim all clients
self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys().then((names) =>
      Promise.all(
        names
          .filter((name) => name !== CACHE_NAME)
          .map((name) => caches.delete(name))
      )
    ).then(() => self.clients.claim())
  );
});

self.addEventListener("install", () => self.skipWaiting());
