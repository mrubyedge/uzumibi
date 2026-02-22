# Service Worker Sample

A sample project for using WASM with Service Workers.

## File Structure

- `index.html`: Main HTML file. Registers the Service Worker and displays results.
- `service-worker.js`: Service Worker implementation. Intercepts fetch requests to `/` and returns a dummy response.

## Flow

1. Access `index.html`
2. JavaScript registers the Service Worker
3. Service Worker starts up
4. Send a fetch request to `/`
5. Service Worker intercepts the request and returns a dummy JSON response
6. Display the response on the page

## How to Run

Service Workers only work over HTTPS or localhost. Start a local server using one of the following methods:

### Using Ruby

```bash
ruby -run -e httpd . -p 8000
```

### Using Node.js http-server

```bash
npx http-server -p 8000
```

After starting the server, access `http://localhost:8000/` in your browser.

## Verification

1. Open browser developer tools
2. Check logs in the Console tab
3. Check registration status in Application tab → Service Workers
4. Confirm that the dummy response is displayed on the page

## Next Steps

Based on this sample, you can add functionality to load and execute WASM modules within the Service Worker.
