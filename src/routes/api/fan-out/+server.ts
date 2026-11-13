import { fanOut } from '@napi-tokio/native';
import { int, readJson } from '#lib/server/input.ts';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request }) => {
	const body = await readJson(request);
	const tasks = int(body.tasks, 1, 64, 16);
	const maxConcurrency = int(body.maxConcurrency, 1, 64, 4);
	const minMs = int(body.minMs, 0, 2000, 150);
	const maxMs = int(body.maxMs, minMs, 2000, 600);

	// One call, one promise. All the spawning and joining happens inside tokio.
	return Response.json(await fanOut(tasks, maxConcurrency, minMs, maxMs));
};
