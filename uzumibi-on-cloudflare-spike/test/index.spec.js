import { env, createExecutionContext, waitOnExecutionContext, SELF } from 'cloudflare:test';
import { describe, it, expect } from 'vitest';
import worker from '../src';

describe('Uzumibi worker', () => {
	it('responds with Uzumibi (unit style)', async () => {
		const request = new Request('http://example.com');
		// Create an empty context to pass to `worker.fetch()`.
		const ctx = createExecutionContext();
		const response = await worker.fetch(request, env, ctx);
		// Wait for all `Promise`s passed to `ctx.waitUntil()` to settle before running test assertions
		await waitOnExecutionContext(ctx);
		expect(await response.text()).toMatchInlineSnapshot(`
			""Uzumibi" is a Japanese term that refers
			to live embers buried under a layer of ash
			to keep the fire from going out.
			"
		`);
	});

	it('responds with Uzumibi (integration style)', async () => {
		const response = await SELF.fetch('http://example.com');
		expect(await response.text()).toMatchInlineSnapshot(`
			""Uzumibi" is a Japanese term that refers
			to live embers buried under a layer of ash
			to keep the fire from going out.
			"
		`);
	});
});
