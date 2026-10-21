/*
 * The desk's service worker. It exists so a phone can install the desk from
 * the browser and open it again without a network round trip for the shell.
 * It caches only the shell: hashed assets cache-first, pages network-first.
 * It never touches the API, so no manuscript, canon or token is ever stored
 * by the browser's cache.
 */
const SHELL = "asten-shell-v1";

self.addEventListener("install", () => {
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys.filter((key) => key !== SHELL).map((key) => caches.delete(key)),
        ),
      )
      .then(() => self.clients.claim()),
  );
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET") return;
  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;
  if (url.pathname.startsWith("/api/") || url.pathname === "/health") return;

  if (url.pathname.startsWith("/assets/")) {
    // Hashed file names: safe to serve from the cache forever.
    event.respondWith(
      caches.open(SHELL).then(async (cache) => {
        const hit = await cache.match(request);
        if (hit) return hit;
        const response = await fetch(request);
        if (response.ok) cache.put(request, response.clone());
        return response;
      }),
    );
    return;
  }

  // Pages, the manifest and icons: network first, cache as the fallback.
  event.respondWith(
    fetch(request)
      .then((response) => {
        if (response.ok) {
          const copy = response.clone();
          caches.open(SHELL).then((cache) => cache.put(request, copy));
        }
        return response;
      })
      .catch(async () => {
        const hit = await caches.match(request);
        if (hit) return hit;
        if (request.mode === "navigate") {
          const shell = await caches.match("/");
          if (shell) return shell;
        }
        return Response.error();
      }),
  );
});
