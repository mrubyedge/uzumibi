import mod from "./$$PROJECT_NAME_UNDERSCORE$$.wasm";

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
		const reqOffset = exports.uzumibi_initialize_request(65536);
		const requestBuffer = new Uint8Array(exports.memory.buffer, reqOffset, 65536);
		const path = new URL(request.url).pathname;
		if (path === "/favicon.ico") {
			return new Response(null, { status: 404 });
		}

		let pos = 0;
		const encoder = new TextEncoder();
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

		// Headers
		const headers = [];
		request.headers.forEach((value, key) => {
			// 一般的なヘッダーのみ含める（必要に応じて調整）
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
		if (pos > 65536) {
			throw new Error("Request data exceeds allocated buffer size");
		}

		const resOffset = exports.uzumibi_start_request();

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
