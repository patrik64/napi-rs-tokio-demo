import { probe } from '@napi-tokio/native';
import { error } from '@sveltejs/kit';
import { PROBE_TARGETS } from '#lib/probe-targets.ts';
import { int, readJson } from '#lib/server/input.ts';
import type { ProbeResponse } from '#lib/types.ts';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request }) => {
	const body = await readJson(request);
	const timeoutMs = int(body.timeoutMs, 100, 5000, 1500);

	// Dial only hosts from our own list, never whatever the request names.
	const requested = Array.isArray(body.targets) ? body.targets : [];
	const targets = PROBE_TARGETS.map((target) => target.address).filter((address) =>
		requested.includes(address)
	);
	if (targets.length === 0) error(400, 'Pick at least one target');

	const started = performance.now();
	const results = await probe(targets, timeoutMs);
	const wallMs = performance.now() - started;

	return Response.json({ results, wallMs } satisfies ProbeResponse);
};
