import { instantiate } from "asyncify-wasm";
import mod from "./uzumibi_on_cloudflare_spike_queue.wasm";

const wasmModule = mod;

export default {
	async queue(batch, env, ctx) {
		const decoder = new TextDecoder();
		const encoder = new TextEncoder();

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
