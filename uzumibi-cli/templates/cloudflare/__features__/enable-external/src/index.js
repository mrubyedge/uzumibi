import { DurableObject } from "cloudflare:workers";
import { instantiate } from "asyncify-wasm";
import mod from "./$$PROJECT_NAME_UNDERSCORE$$.wasm";

const wasmModule = mod;

/**
 * Durable Object for Uzumibi::KV storage
 */
export class UzumibiKVObject extends DurableObject {
    async get(key) {
        const value = await this.ctx.storage.get(key);
        return value ?? null;
    }

    async set(key, value) {
        await this.ctx.storage.put(key, value);
    }
}

export default {
    async fetch(request, env, ctx) {
        const path = new URL(request.url).pathname;
        if (path === "/favicon.ico") {
            return new Response(null, { status: 404 });
        }

        const query = new URL(request.url).searchParams;

        // Durable Object stub (if binding exists)
        const doStub = env.UZUMIBI_KV_DATA
            ? env.UZUMIBI_KV_DATA.getByName("default")
            : null;

        const decoder = new TextDecoder();
        const encoder = new TextEncoder();

        const importObject = {
            env: {
                debug_console_log: (ptr, size) => {
                    const memory = exports.memory;
                    const buffer = new Uint8Array(memory.buffer, ptr, size);
                    console.log(`[debug]: ${decoder.decode(buffer)}`);
                    return 0;
                },

                // Fetch.fetch(url, method, body, headers) -> packed Uzumibi::Response
                // Format: u16 status | u16 headers_count | (u16 key_size, key, u16 value_size, value)... | u32 body_size | body
                uzumibi_cf_fetch: async (
                    urlPtr, urlSize,
                    methodPtr, methodSize,
                    bodyPtr, bodySize,
                    headersPtr, headersSize,
                    resultPtr, resultMaxSize
                ) => {
                    const memory = exports.memory;
                    const url = decoder.decode(new Uint8Array(memory.buffer, urlPtr, urlSize));
                    const method = decoder.decode(new Uint8Array(memory.buffer, methodPtr, methodSize));
                    const body = bodySize > 0
                        ? decoder.decode(new Uint8Array(memory.buffer, bodyPtr, bodySize))
                        : null;

                    const fetchOptions = { method };
                    if (body && method !== "GET" && method !== "HEAD") {
                        fetchOptions.body = body;
                    }

                    // Unpack request headers: u16 LE count, then (u16 LE key_size, key, u16 LE value_size, value) * count
                    if (headersSize >= 2) {
                        const hView = new DataView(memory.buffer, headersPtr, headersSize);
                        const hCount = hView.getUint16(0, true);
                        if (hCount > 0) {
                            const reqHeaders = {};
                            let hPos = 2;
                            for (let i = 0; i < hCount; i++) {
                                const kLen = hView.getUint16(hPos, true);
                                hPos += 2;
                                const k = decoder.decode(new Uint8Array(memory.buffer, headersPtr + hPos, kLen));
                                hPos += kLen;
                                const vLen = hView.getUint16(hPos, true);
                                hPos += 2;
                                const v = decoder.decode(new Uint8Array(memory.buffer, headersPtr + hPos, vLen));
                                hPos += vLen;
                                reqHeaders[k] = v;
                            }
                            fetchOptions.headers = reqHeaders;
                        }
                    }

                    const response = await fetch(url, fetchOptions);
                    const responseBody = await response.text();

                    // Collect response headers
                    const respHeaders = [];
                    response.headers.forEach((value, key) => {
                        respHeaders.push({ key, value });
                    });

                    // Pack into binary format matching Uzumibi::Response#to_shared_memory
                    const resultView = new DataView(memory.buffer, resultPtr, resultMaxSize);
                    const resultBuffer = new Uint8Array(memory.buffer, resultPtr, resultMaxSize);
                    let pos = 0;

                    // Status code (u16 LE)
                    resultView.setUint16(pos, response.status, true);
                    pos += 2;

                    // Headers count (u16 LE)
                    resultView.setUint16(pos, respHeaders.length, true);
                    pos += 2;

                    // Each header
                    for (const header of respHeaders) {
                        const keyBytes = encoder.encode(header.key);
                        resultView.setUint16(pos, keyBytes.length, true);
                        pos += 2;
                        resultBuffer.set(keyBytes, pos);
                        pos += keyBytes.length;

                        const valueBytes = encoder.encode(header.value);
                        resultView.setUint16(pos, valueBytes.length, true);
                        pos += 2;
                        resultBuffer.set(valueBytes, pos);
                        pos += valueBytes.length;
                    }

                    // Body size (u32 LE)
                    const bodyBytes = encoder.encode(responseBody);
                    resultView.setUint32(pos, bodyBytes.length, true);
                    pos += 4;

                    // Body
                    const bodyLen = Math.min(bodyBytes.length, resultMaxSize - pos);
                    resultBuffer.set(bodyBytes.slice(0, bodyLen), pos);
                    pos += bodyLen;

                    return pos;
                },

                // KV.get(key) -> value string (via Durable Object)
                uzumibi_cf_durable_object_get: async (keyPtr, keySize, resultPtr, resultMaxSize) => {
                    if (!doStub) return -1;
                    const memory = exports.memory;
                    const key = decoder.decode(new Uint8Array(memory.buffer, keyPtr, keySize));

                    const value = await doStub.get(key);
                    if (value === null) {
                        return -1;
                    }
                    const valueBytes = encoder.encode(value);
                    const length = Math.min(valueBytes.length, resultMaxSize);
                    const resultBuffer = new Uint8Array(memory.buffer, resultPtr, resultMaxSize);
                    resultBuffer.set(valueBytes.slice(0, length));
                    return length;
                },

                // KV.set(key, value) (via Durable Object)
                uzumibi_cf_durable_object_set: async (keyPtr, keySize, valuePtr, valueSize) => {
                    if (!doStub) return -1;
                    const memory = exports.memory;
                    const key = decoder.decode(new Uint8Array(memory.buffer, keyPtr, keySize));
                    const value = decoder.decode(new Uint8Array(memory.buffer, valuePtr, valueSize));

                    await doStub.set(key, value);
                    return 0;
                },

                // Queue.send(queue_name, message)
                uzumibi_cf_queue_send: async (queueNamePtr, queueNameSize, messagePtr, messageSize) => {
                    const memory = exports.memory;
                    const queueName = decoder.decode(new Uint8Array(memory.buffer, queueNamePtr, queueNameSize));
                    const message = decoder.decode(new Uint8Array(memory.buffer, messagePtr, messageSize));

                    const queue = env[queueName];
                    if (!queue) {
                        console.error(`Queue binding '${queueName}' not found`);
                        return -1;
                    }
                    await queue.send(message);
                    return 0;
                },
            },
        };

        const instance = await instantiate(wasmModule, importObject);
        const exports = instance.exports;

        const reqResult = await exports.uzumibi_initialize_request(65536);
        const reqOffset = Number(reqResult & 0xFFFFFFFFn);
        if (reqOffset === 0) {
            const errOffset = Number((reqResult >> 32n) & 0xFFFFFFFFn);
            const buffer = new Uint8Array(exports.memory.buffer, errOffset);
            let errStr = "";
            for (let i = 0; buffer[i] !== 0; i++) {
                errStr += String.fromCharCode(buffer[i]);
            }
            throw new Error(`Failed to initialize request: ${errStr}`);
        }
        const requestBuffer = new Uint8Array(exports.memory.buffer, reqOffset, 65536);

        let pos = 0;
        const dataView = new DataView(exports.memory.buffer, reqOffset);

        const method = encoder.encode(request.method);
        requestBuffer.fill(0, pos, pos + 6);
        requestBuffer.set(method.slice(0, 6), pos);
        pos += 6;

        // Path size (u16 little-endian)
        const pathBytes = encoder.encode(path);
        dataView.setUint16(pos, pathBytes.length, true);
        pos += 2;

        // Path
        requestBuffer.set(pathBytes, pos);
        pos += pathBytes.length;

        // Query string size (u16 little-endian)
        const queryString = query.toString();
        const queryBytes = encoder.encode(queryString);
        dataView.setUint16(pos, queryBytes.length, true);
        pos += 2;

        // Query string
        requestBuffer.set(queryBytes, pos);
        pos += queryBytes.length;

        // Headers
        const headers = [];
        request.headers.forEach((value, key) => {
            if (key.toLowerCase() !== 'cf-connecting-ip' &&
                key.toLowerCase() !== 'cf-ray' &&
                !key.toLowerCase().startsWith('x-')) {
                headers.push({ key, value });
            }
        });

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

        // Request body size (u32 little-endian)
        const bodyBytes = request.body ? new Uint8Array(await request.arrayBuffer()) : new Uint8Array(0);
        dataView.setUint32(pos, bodyBytes.length, true);
        pos += 4;

        // Request body
        requestBuffer.set(bodyBytes, pos);
        pos += bodyBytes.length;

        if (pos > 65536) {
            throw new Error("Request data exceeds allocated buffer size");
        }

        const resResult = await exports.uzumibi_start_request();
        const resOffset = Number(resResult & 0xFFFFFFFFn);
        const upperBits = Number((resResult >> 32n) & 0xFFFFFFFFn);

        if (upperBits !== 0) {
            const upperTag = (upperBits >> 16) & 0xFFFF;
            if (upperTag === 0xFEFF) {
                // Special route
                if (upperBits === 0xFEFFFFFF) {
                    // Pass through to assets
                    return env.ASSETS.fetch(request);
                }
                throw new Error(`Unknown routing bits: 0x${upperBits.toString(16)}`);
            }
            // Error case
            const buffer = new Uint8Array(exports.memory.buffer, upperBits);
            let errStr = "";
            for (let i = 0; buffer[i] !== 0; i++) {
                errStr += String.fromCharCode(buffer[i]);
            }
            throw new Error(`Failed to start request: ${errStr}`);
        }

        // Unpack response
        const resDataView = new DataView(exports.memory.buffer, resOffset);

        let resPos = 0;

        // Status code (u16 little-endian)
        const statusCode = resDataView.getUint16(resPos, true);
        resPos += 2;

        // Headers count (u16 little-endian)
        const headersCount = resDataView.getUint16(resPos, true);
        resPos += 2;

        // Parse headers
        const responseHeaders = new Headers();
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

            console.log(`[Response Header] ${key}: ${value}`);
            responseHeaders.set(key, value);
        }

        // Body size (u32 little-endian)
        const bodySize = resDataView.getUint32(resPos, true);
        resPos += 4;

        // Body
        const bodyBuffer = new Uint8Array(exports.memory.buffer, resOffset + resPos, bodySize);
        const responseText = decoder.decode(bodyBuffer);

        return new Response(responseText, { status: statusCode, headers: responseHeaders });
    }
};
