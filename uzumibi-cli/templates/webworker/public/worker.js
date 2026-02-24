// Web Worker that loads WASM and handles requests via postMessage

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
    console.log('[Worker] WASM module loaded and instantiated');
    wasmExports = wasmModule.instance.exports;
    return wasmExports;
}

// Pack request data into WASM memory
function packRequest(exports, request) {
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
    const pathBytes = encoder.encode(request.path);
    dataView.setUint16(pos, pathBytes.length, true);
    pos += 2;

    // Path
    requestBuffer.set(pathBytes, pos);
    pos += pathBytes.length;

    // Query string size (u16 little-endian)
    const queryString = request.query || '';
    const queryBytes = encoder.encode(queryString);
    dataView.setUint16(pos, queryBytes.length, true);
    pos += 2;

    // Query string
    requestBuffer.set(queryBytes, pos);
    pos += queryBytes.length;

    // Headers
    const headers = request.headers || [];

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
async function handleRequest(request) {
    const exports = await initWasm();

    console.log('[Worker] Handling request:', request.method, request.path);

    // Pack request
    packRequest(exports, request);

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
        return {
            statusCode: 500,
            headers: { 'Content-Type': 'text/plain' },
            body: `Failed to start request: ${errStr}`
        };
    }

    // Unpack response
    return unpackResponse(exports, resOffset);
}

// Listen for messages from main thread
self.addEventListener('message', async (event) => {
    const { id, request } = event.data;

    try {
        const response = await handleRequest(request);
        self.postMessage({ id, success: true, response });
    } catch (error) {
        console.error('[Worker] Error handling request:', error);
        self.postMessage({
            id,
            success: false,
            error: error.message
        });
    }
});

// Initialize WASM when worker starts
initWasm().then(() => {
    console.log('[Worker] Ready');
    self.postMessage({ type: 'ready' });
}).catch((error) => {
    console.error('[Worker] Failed to initialize:', error);
    self.postMessage({ type: 'error', error: error.message });
});
