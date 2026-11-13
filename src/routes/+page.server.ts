import { runtimeInfo } from '@napi-tokio/native';
import { VERSION } from '@sveltejs/kit';
import { sources } from '#lib/server/sources.ts';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async () => {
	const { napiVersion, tokioVersion, workers } = runtimeInfo();

	return {
		versions: {
			'napi-rs': napiVersion,
			tokio: tokioVersion,
			SvelteKit: VERSION,
			Node: process.versions.node
		},
		workers,
		sources: await sources
	};
};
