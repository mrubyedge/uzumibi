// WASM instance and exports
let wasmExports = null;

// Load and initialize WASM module
async function initWasm() {
    if (wasmExports) {
        return wasmExports;
    }

    const importObject = {
        env: {
            debug_console_log: (ptr, size) => {
                const memory = wasmExports.memory;
                let str = "";
                const buffer = new Uint8Array(memory.buffer);
                for (let i = ptr; i < ptr + size; i++) {
                    str += String.fromCharCode(buffer[i]);
                }
                console.log(`[WASM debug]: ${str}`);
                return 0;
            },
        },
    };

    const response = await fetch('/app.wasm');
    const wasmModule = await WebAssembly.instantiate(await response.arrayBuffer(), importObject);
    console.log('[Service Worker] WASM module loaded and instantiated');
    wasmExports = wasmModule.instance.exports;
    return wasmExports;
}

// Pack request data into WASM memory
function packRequest(exports, request, url) {
    const reqResult = exports.uzumibi_initialize_request(65536);
    const reqOffset = Number(reqResult & 0xFFFFFFFFn);
    if (reqOffset === 0) {
        const errOffset = Number((reqResult >> 32n) & 0xFFFFFFFFn);
        let errStr = "";
        const buffer = new Uint8Array(exports.memory.buffer, errOffset);
        for (let i = 0; buffer[i] !== 0; i++) {
            errStr += String.fromCharCode(buffer[i]);
        }
        throw new Error(`Failed to initialize request: ${errStr}`);
    }

    const requestBuffer = new Uint8Array(exports.memory.buffer, reqOffset, 65536);
    const encoder = new TextEncoder();
    const dataView = new DataView(exports.memory.buffer, reqOffset);

    let pos = 0;

    // Method (6 bytes, null-padded)
    const method = encoder.encode(request.method);
    requestBuffer.fill(0, pos, pos + 6);
    requestBuffer.set(method.slice(0, 6), pos);
    pos += 6;

    // Path size (u16 little-endian)
    const path = url.pathname;
    const pathBytes = encoder.encode(path);
    dataView.setUint16(pos, pathBytes.length, true);
    pos += 2;

    // Path
    requestBuffer.set(pathBytes, pos);
    pos += pathBytes.length;

    // Query string size (u16 little-endian)
    const queryString = url.search.slice(1); // Remove leading '?'
    const queryBytes = encoder.encode(queryString);
    dataView.setUint16(pos, queryBytes.length, true);
    pos += 2;

    // Query string
    requestBuffer.set(queryBytes, pos);
    pos += queryBytes.length;

    // Headers
    const headers = [];
    for (const [key, value] of request.headers.entries()) {
        headers.push({ key, value });
    }

    // Headers count (u16 little-endian)
    dataView.setUint16(pos, headers.length, true);
    pos += 2;

    // Each header
    for (const header of headers) {
        // Header key size (u16 little-endian)
        const keyBytes = encoder.encode(header.key);
        dataView.setUint16(pos, keyBytes.length, true);
        pos += 2;

        // Header key
        requestBuffer.set(keyBytes, pos);
        pos += keyBytes.length;

        // Header value size (u16 little-endian)
        const valueBytes = encoder.encode(header.value);
        dataView.setUint16(pos, valueBytes.length, true);
        pos += 2;

        // Header value
        requestBuffer.set(valueBytes, pos);
        pos += valueBytes.length;
    }

    // Request body size (u32 little-endian) - always 0 for now
    dataView.setUint32(pos, 0, true);
    pos += 4;

    if (pos > 65536) {
        throw new Error("Request data exceeds allocated buffer size");
    }

    return reqOffset;
}

// Unpack response data from WASM memory
function unpackResponse(exports, resOffset) {
    const decoder = new TextDecoder();
    const resDataView = new DataView(exports.memory.buffer, resOffset);

    let resPos = 0;

    // Status code (u16 little-endian)
    const statusCode = resDataView.getUint16(resPos, true);
    resPos += 2;

    // Headers count (u16 little-endian)
    const headersCount = resDataView.getUint16(resPos, true);
    resPos += 2;

    // Parse headers
    const responseHeaders = {};
    for (let i = 0; i < headersCount; i++) {
        // Header key size (u16 little-endian)
        const keySize = resDataView.getUint16(resPos, true);
        resPos += 2;

        // Header key
        const keyBytes = new Uint8Array(exports.memory.buffer, resOffset + resPos, keySize);
        const key = decoder.decode(keyBytes);
        resPos += keySize;

        // Header value size (u16 little-endian)
        const valueSize = resDataView.getUint16(resPos, true);
        resPos += 2;

        // Header value
        const valueBytes = new Uint8Array(exports.memory.buffer, resOffset + resPos, valueSize);
        const value = decoder.decode(valueBytes);
        resPos += valueSize;

        responseHeaders[key] = value;
    }

    // Body size (u32 little-endian)
    const bodySize = resDataView.getUint32(resPos, true);
    resPos += 4;

    // Body
    const bodyBuffer = new Uint8Array(exports.memory.buffer, resOffset + resPos, bodySize);
    const responseText = decoder.decode(bodyBuffer);

    return { statusCode, headers: responseHeaders, body: responseText };
}

// Handle request through WASM
async function handleWithWasm(request) {
    const exports = await initWasm();
    const url = new URL(request.url);

    console.log('[Service Worker] Handling request with WASM:', url.pathname);

    // Pack request
    packRequest(exports, request, url);

    // Execute WASM
    const resResult = exports.uzumibi_start_request();
    const resOffset = Number(resResult & 0xFFFFFFFFn);
    if (resOffset === 0) {
        const errOffset = Number((resResult >> 32n) & 0xFFFFFFFFn);
        let errStr = "";
        const buffer = new Uint8Array(exports.memory.buffer, errOffset);
        for (let i = 0; buffer[i] !== 0; i++) {
            errStr += String.fromCharCode(buffer[i]);
        }
        return new Response(`Failed to start request: ${errStr}`, { status: 500 });

    }

    // Unpack response
    const { statusCode, headers, body } = unpackResponse(exports, resOffset);

    return new Response(body, {
        status: statusCode,
        headers: headers
    });
}

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

    // Handle index.html requests normally
    if (event.request.mode === 'navigate') {
        event.respondWith(fetch(event.request));
        return;
    }

    // Handle app.wasm requests normally
    if (url.pathname === '/app.wasm') {
        event.respondWith(fetch(event.request));
        return;
    }

    if (url.pathname === '/service-worker.js') {
        event.respondWith(fetch(event.request));
        return;
    }

    // Route API requests through WASM
    if (event.request.method === 'GET') {
        event.respondWith(handleWithWasm(event.request));
        return;
    }

    // Handle other requests normally
    event.respondWith(fetch(event.request));
});
