import { DurableObject } from "cloudflare:workers";
import { instantiate } from "asyncify-wasm";
import mod from "./uzumibi_on_cloudflare_spike_queue.wasm";

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
	async queue(batch, env, ctx) {
		const decoder = new TextDecoder();
		const encoder = new TextEncoder();

		// Durable Object stub (if binding exists)
		const doStub = env.UZUMIBI_KV_DATA
			? env.UZUMIBI_KV_DATA.getByName("default")
			: null;

		// Current message being processed (set per iteration)
		const getMessage = (id) => {
			const message = batch.messages.find((m) => m.id === id);
			if (!message) throw new Error(`Message not found for id: ${id}`);
			return message;
		};

		const importObject = {
			env: {
				debug_console_log: (ptr, size) => {
					const memory = exports.memory;
					const buffer = new Uint8Array(memory.buffer, ptr, size);
					console.log(`[debug]: ${decoder.decode(buffer)}`);
					return 0;
				},

				// Fetch.fetch(url, method, body) -> packed Uzumibi::Response
				// Dummy implementation: queue consumers don't have access to fetch directly,
				// but enable-external is enabled when queue feature is active.
				uzumibi_cf_fetch: async (
					urlPtr, urlSize,
					methodPtr, methodSize,
					bodyPtr, bodySize,
					resultPtr, resultMaxSize,
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

					const response = await fetch(url, fetchOptions);
					const responseBody = await response.text();

					const respHeaders = [];
					response.headers.forEach((value, key) => {
						respHeaders.push({ key, value });
					});

					const resultView = new DataView(memory.buffer, resultPtr, resultMaxSize);
					const resultBuffer = new Uint8Array(memory.buffer, resultPtr, resultMaxSize);
					let pos = 0;

					resultView.setUint16(pos, response.status, true);
					pos += 2;

					resultView.setUint16(pos, respHeaders.length, true);
					pos += 2;

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

					const bodyBytes = encoder.encode(responseBody);
					resultView.setUint32(pos, bodyBytes.length, true);
					pos += 4;

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
					if (value === null) return -1;

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

				uzumibi_cf_message_ack: async (idPtr, idSize) => {
					const id = decoder.decode(new Uint8Array(exports.memory.buffer, idPtr, idSize));
					getMessage(id).ack();
					return 0;
				},

				uzumibi_cf_message_retry: async (idPtr, idSize, delaySeconds) => {
					const id = decoder.decode(new Uint8Array(exports.memory.buffer, idPtr, idSize));
					getMessage(id).retry({ delaySeconds });
					return 0;
				},
			},
		};

		const instance = await instantiate(wasmModule, importObject);
		const exports = instance.exports;

		for (const message of batch.messages) {
			const idBytes = encoder.encode(message.id);
			const timestampBytes = encoder.encode(
				message.timestamp.toISOString(),
			);
			const bodyBytes = encoder.encode(
				typeof message.body === "string"
					? message.body
					: JSON.stringify(message.body),
			);
			const attempts = message.attempts;

			// Pack message data:
			//   u16 LE id_size, id bytes,
			//   u16 LE timestamp_size, timestamp bytes,
			//   u32 LE body_size, body bytes,
			//   u32 LE attempts
			const totalSize =
				2 +
				idBytes.length +
				2 +
				timestampBytes.length +
				4 +
				bodyBytes.length +
				4;

			const msgResult =
				await exports.uzumibi_initialize_message(totalSize);
			const msgOffset = Number(msgResult & 0xffffffffn);
			if (msgOffset === 0) {
				const errOffset = Number(
					(msgResult >> 32n) & 0xffffffffn,
				);
				const buffer = new Uint8Array(
					exports.memory.buffer,
					errOffset,
				);
				let errStr = "";
				for (let i = 0; buffer[i] !== 0; i++) {
					errStr += String.fromCharCode(buffer[i]);
				}
				throw new Error(
					`Failed to initialize message: ${errStr}`,
				);
			}

			const msgBuffer = new Uint8Array(
				exports.memory.buffer,
				msgOffset,
				totalSize,
			);
			const dataView = new DataView(
				exports.memory.buffer,
				msgOffset,
			);
			let pos = 0;

			// id
			dataView.setUint16(pos, idBytes.length, true);
			pos += 2;
			msgBuffer.set(idBytes, pos);
			pos += idBytes.length;

			// timestamp
			dataView.setUint16(pos, timestampBytes.length, true);
			pos += 2;
			msgBuffer.set(timestampBytes, pos);
			pos += timestampBytes.length;

			// body
			dataView.setUint32(pos, bodyBytes.length, true);
			pos += 4;
			msgBuffer.set(bodyBytes, pos);
			pos += bodyBytes.length;

			// attempts
			dataView.setUint32(pos, attempts, true);

			const result = await exports.uzumibi_start_message();
			if (result !== 0) {
				const buffer = new Uint8Array(
					exports.memory.buffer,
					result,
				);
				let errStr = "";
				for (let i = 0; buffer[i] !== 0; i++) {
					errStr += String.fromCharCode(buffer[i]);
				}
				throw new Error(
					`Failed to process message: ${errStr}`,
				);
			}
		}
	},
};
