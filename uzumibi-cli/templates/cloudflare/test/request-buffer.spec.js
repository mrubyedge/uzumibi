import { describe, expect, it } from "vitest";
import {
    RequestTooLargeError,
    writeRequestToWasm,
} from "../src/request-buffer.js";

function createExports(maxBytes) {
    const memory = new WebAssembly.Memory({ initial: 4 });
    let allocatedSize = 0;

    return {
        exports: {
            memory,
            uzumibi_http_max_bytes: () => maxBytes,
            uzumibi_initialize_request: (size) => {
                allocatedSize = size;
                return 1024n;
            },
        },
        allocatedSize: () => allocatedSize,
    };
}

describe("Cloudflare request buffer", () => {
    it("allocates the encoded request size instead of the configured maximum", async () => {
        const wasm = createExports(131072);
        const request = new Request("https://example.com/items?q=uzumibi", {
            method: "POST",
            headers: { "content-type": "text/plain" },
            body: "hello",
        });

        const encodedSize = await writeRequestToWasm(wasm.exports, request);

        expect(wasm.allocatedSize()).toBe(encodedSize);
        expect(encodedSize).toBeLessThan(131072);
    });

    it("accepts a request larger than the old 64 KiB limit", async () => {
        const wasm = createExports(131072);
        const request = new Request("https://example.com/upload", {
            method: "POST",
            body: new Uint8Array(70000),
        });

        const encodedSize = await writeRequestToWasm(wasm.exports, request);

        expect(encodedSize).toBeGreaterThan(65536);
        expect(wasm.allocatedSize()).toBe(encodedSize);
    });

    it("rejects a request that exceeds the configured maximum", async () => {
        const wasm = createExports(65536);
        const request = new Request("https://example.com/upload", {
            method: "POST",
            body: new Uint8Array(70000),
        });

        await expect(writeRequestToWasm(wasm.exports, request)).rejects.toBeInstanceOf(
            RequestTooLargeError,
        );
        expect(wasm.allocatedSize()).toBe(0);
    });
});
