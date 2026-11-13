import { countPrimesBlocking, countPrimesOnWorker, countPrimesSync } from '@napi-tokio/native';
import { error } from '@sveltejs/kit';
import { int, readJson } from '#lib/server/input.ts';
import { watchLoopLag } from '#lib/server/loop-lag.ts';
import type { PrimesResponse, PrimesVariant } from '#lib/types.ts';
import type { RequestHandler } from './$types';

// The same Rust function, exported three ways.
const variants = {
	sync: countPrimesSync,
	worker: countPrimesOnWorker,
	blocking: countPrimesBlocking
};

export const POST: RequestHandler = async ({ request }) => {
	const body = await readJson(request);
	const limit = int(body.limit, 1_000, 20_000_000, 5_000_000);
	const variant = body.variant as PrimesVariant;
	if (!Object.hasOwn(variants, variant)) error(400, 'Unknown variant');

	// While the count runs, measure whether Node itself can still do anything.
	const lag = watchLoopLag();
	try {
		const count = await variants[variant](limit);
		return Response.json({ ...count, loopLagMs: lag.take() } satisfies PrimesResponse);
	} finally {
		lag.stop();
	}
};
