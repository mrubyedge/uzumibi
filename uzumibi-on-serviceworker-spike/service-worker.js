// Service Worker installation
self.addEventListener('install', event => {
    console.log('Service Worker installing...');
    // Activate immediately
    self.skipWaiting();
});

// Service Worker activation
self.addEventListener('activate', event => {
    console.log('Service Worker activating...');
    // Start taking control immediately
    event.waitUntil(self.clients.claim());
});

// Handle fetch events
self.addEventListener('fetch', event => {
    const url = new URL(event.request.url);

    console.log('Fetch event for:', url.pathname);

    // Return dummy response for root path (/) requests
    if (url.pathname === '/' && event.request.method === 'GET') {
        // Handle index.html requests normally
        if (event.request.mode === 'navigate') {
            event.respondWith(fetch(event.request));
            return;
        }

        // Return dummy response for other / fetch requests
        event.respondWith(
            new Response(JSON.stringify({
                message: 'Hello from Service Worker!',
                timestamp: new Date().toISOString(),
                path: url.pathname
            }), {
                status: 200,
                headers: {
                    'Content-Type': 'application/json'
                }
            })
        );
    } else {
        // Handle other requests normally
        event.respondWith(fetch(event.request));
    }
});
