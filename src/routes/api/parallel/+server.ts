import { countPrimesInParallel } from '@napi-tokio/native';
import { int, readJson } from '#lib/server/input.ts';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request }) => {
	const body = await readJson(request);
	const tasks = int(body.tasks, 1, 32, 8);
	const limit = int(body.limit, 1_000, 12_000_000, 10_000_000);
	const yieldBetweenBatches = body.yieldBetweenBatches !== false;

	// Every tokio worker may be flat out for seconds. To Node it is one pending promise.
	return Response.json(await countPrimesInParallel(tasks, limit, yieldBetweenBatches));
};
