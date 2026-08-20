import mod from "./uzumibi_on_cloudflare_spike.wasm";
import { RequestTooLargeError, writeRequestToWasm } from "./request-buffer.js";

const importObject = {
	env: {
		debug_console_log: (ptr, size) => {
			const memory = exports.memory;
			let str = "";
			const buffer = new Uint8Array(memory.buffer);
			for (let i = ptr; i < ptr + size; i++) {
				str += String.fromCharCode(buffer[i]);
			}
			console.log(`[debug]: ${str}`);
			return 0;
		},
	},
};
const instance = await WebAssembly.instantiate(mod, importObject);
const exports = instance.exports;

export default {
	async fetch(request, env, ctx) {
		const path = new URL(request.url).pathname;
		if (path === "/favicon.ico") {
			return new Response(null, { status: 404 });
		}

		try {
			await writeRequestToWasm(exports, request);
		} catch (error) {
			if (error instanceof RequestTooLargeError) {
				return new Response(error.message, { status: 413 });
			}
			throw error;
		}

		const resResult = exports.uzumibi_start_request();
		const resOffset = Number(resResult & 0xFFFFFFFFn);
		if (resOffset === 0) {
			const errOffset = Number((resResult >> 32n) & 0xFFFFFFFFn);
			const decoder = new TextDecoder();
			let errStr = "";
			const buffer = new Uint8Array(exports.memory.buffer, errOffset);
			for (let i = 0; buffer[i] !== 0; i++) {
				errStr += String.fromCharCode(buffer[i]);
			}
			throw new Error(`Failed to start request: ${errStr}`);
		}

		// Unpack response
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
