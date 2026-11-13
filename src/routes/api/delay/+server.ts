import { delay } from '@napi-tokio/native';
import { int, readJson } from '#lib/server/input.ts';
import type { DelayResponse } from '#lib/types.ts';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request }) => {
	const body = await readJson(request);
	const workMs = int(body.workMs, 0, 5000, 1000);
	const deadlineMs = body.deadlineMs == null ? null : int(body.deadlineMs, 0, 5000, 1000);

	const started = performance.now();
	let result: DelayResponse;
	try {
		// `delay` is a Rust `async fn`. On this side it is simply a promise.
		result = { outcome: 'resolved', elapsedMs: await delay(workMs, deadlineMs) };
	} catch (error) {
		// A Rust `Err` rejects with a JS `Error`; `code` carries the napi status.
		const { message, code = 'GenericFailure' } = error as Error & { code?: string };
		result = { outcome: 'rejected', elapsedMs: performance.now() - started, message, code };
	}

	return Response.json(result);
};
