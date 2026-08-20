export class RequestTooLargeError extends Error {
    constructor(actualBytes, maxBytes) {
        super(`Encoded request size ${actualBytes} exceeds the configured maximum of ${maxBytes} bytes`);
        this.name = "RequestTooLargeError";
        this.actualBytes = actualBytes;
        this.maxBytes = maxBytes;
    }
}

function readError(exports, result, operation) {
    const errorOffset = Number((result >> 32n) & 0xFFFFFFFFn);
    const buffer = new Uint8Array(exports.memory.buffer, errorOffset);
    let message = "";
    for (let i = 0; buffer[i] !== 0; i++) {
        message += String.fromCharCode(buffer[i]);
    }
    return new Error(`Failed to ${operation}: ${message}`);
}

export async function writeRequestToWasm(exports, request) {
    const encoder = new TextEncoder();
    const url = new URL(request.url);
    const methodBytes = encoder.encode(request.method);
    const pathBytes = encoder.encode(url.pathname);
    const queryBytes = encoder.encode(url.searchParams.toString());
    const bodyBytes = request.body
        ? new Uint8Array(await request.arrayBuffer())
        : new Uint8Array(0);

    const headers = [];
    request.headers.forEach((value, key) => {
        if (key.toLowerCase() !== "cf-connecting-ip"
            && key.toLowerCase() !== "cf-ray"
            && !key.toLowerCase().startsWith("x-")) {
            headers.push({
                key: encoder.encode(key),
                value: encoder.encode(value),
            });
        }
    });

    const headersSize = headers.reduce(
        (size, header) => size + 2 + header.key.length + 2 + header.value.length,
        0,
    );
    const requiredSize = 6
        + 2 + pathBytes.length
        + 2 + queryBytes.length
        + 2 + headersSize
        + 4 + bodyBytes.length;
    const maxBytes = Number(await exports.uzumibi_http_max_bytes());

    if (requiredSize > maxBytes) {
        throw new RequestTooLargeError(requiredSize, maxBytes);
    }

    const result = await exports.uzumibi_initialize_request(requiredSize);
    const offset = Number(result & 0xFFFFFFFFn);
    if (offset === 0) {
        throw readError(exports, result, "initialize request");
    }

    const requestBuffer = new Uint8Array(exports.memory.buffer, offset, requiredSize);
    const dataView = new DataView(exports.memory.buffer, offset, requiredSize);
    let pos = 0;

    requestBuffer.fill(0, pos, pos + 6);
    requestBuffer.set(methodBytes.slice(0, 6), pos);
    pos += 6;

    dataView.setUint16(pos, pathBytes.length, true);
    pos += 2;
    requestBuffer.set(pathBytes, pos);
    pos += pathBytes.length;

    dataView.setUint16(pos, queryBytes.length, true);
    pos += 2;
    requestBuffer.set(queryBytes, pos);
    pos += queryBytes.length;

    dataView.setUint16(pos, headers.length, true);
    pos += 2;
    for (const header of headers) {
        dataView.setUint16(pos, header.key.length, true);
        pos += 2;
        requestBuffer.set(header.key, pos);
        pos += header.key.length;
        dataView.setUint16(pos, header.value.length, true);
        pos += 2;
        requestBuffer.set(header.value, pos);
        pos += header.value.length;
    }

    dataView.setUint32(pos, bodyBytes.length, true);
    pos += 4;
    requestBuffer.set(bodyBytes, pos);

    return requiredSize;
}
