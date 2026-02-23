# Service Worker/Web Worker (Experimental)

Run Uzumibi directly in the browser using Service Workers or Web Workers.

**Status**: Experimental - For demonstration and testing purposes

### Features

- **Browser-Based**: Runs entirely in the browser
- **Offline Support**: Service Workers enable offline functionality
- **Client-Side Routing**: Handle requests without a server
- **Development Tool**: Useful for testing and development

### Project Structure

The Service Worker spike project demonstrates:

- Loading WASM in a Service Worker
- Intercepting fetch requests
- Processing requests through Uzumibi
- Returning responses to the browser

### Use Cases

- **Offline-First Apps**: Progressive Web Apps with offline routing
- **Development/Testing**: Test Uzumibi logic in the browser
- **Client-Side APIs**: Mock APIs or client-side data processing
- **Educational**: Learn how Uzumibi works

### Limitations

- **Browser Only**: Not suitable for production server workloads
- **Security Restrictions**: Subject to browser security policies
- **Limited Storage**: Browser storage APIs only
- **Performance**: May be slower than server-side execution

### How It Works

1. Register Service Worker
2. Service Worker loads WASM module
3. Intercept fetch events
4. Route through Uzumibi Router
5. Return response to page

See the [uzumibi-on-serviceworker-spike](https://github.com/mrubyedge/uzumibi/tree/main/uzumibi-on-serviceworker-spike) directory for the complete implementation.
