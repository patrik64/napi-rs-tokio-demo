// The page shows the code that is actually running. Vite's `?raw` imports
// inline each file as a string, so what you read can't drift from what runs.
import { createHighlighterCore } from 'shiki/core';
import { createJavaScriptRegexEngine } from 'shiki/engine/javascript';

import cargoToml from '../../../native/Cargo.toml?raw';
import typeDefs from '../../../native/index.d.ts?raw';
import cpuRs from '../../../native/src/cpu.rs?raw';
import delayRs from '../../../native/src/delay.rs?raw';
import fanOutRs from '../../../native/src/fan_out.rs?raw';
import libRs from '../../../native/src/lib.rs?raw';
import parallelRs from '../../../native/src/parallel.rs?raw';
import probeRs from '../../../native/src/probe.rs?raw';
import runtimeRs from '../../../native/src/runtime.rs?raw';
import streamRs from '../../../native/src/stream.rs?raw';
import delayEndpoint from '../../routes/api/delay/+server.ts?raw';
import fanOutEndpoint from '../../routes/api/fan-out/+server.ts?raw';
import parallelEndpoint from '../../routes/api/parallel/+server.ts?raw';
import primesEndpoint from '../../routes/api/primes/+server.ts?raw';
import probeEndpoint from '../../routes/api/probe/+server.ts?raw';
import pulseEndpoint from '../../routes/api/pulse/+server.ts?raw';
import streamEndpoint from '../../routes/api/stream/+server.ts?raw';
import type { SourceFile } from '../types.ts';
import loopLag from './loop-lag.ts?raw';
import sse from './sse.ts?raw';

const files = {
	setup: {
		'native/Cargo.toml': cargoToml,
		'native/src/lib.rs': libRs,
		'native/index.d.ts': typeDefs
	},
	delay: {
		'native/src/delay.rs': delayRs,
		'src/routes/api/delay/+server.ts': delayEndpoint
	},
	primes: {
		'native/src/cpu.rs': cpuRs,
		'src/routes/api/primes/+server.ts': primesEndpoint,
		'src/lib/server/loop-lag.ts': loopLag
	},
	fanOut: {
		'native/src/fan_out.rs': fanOutRs,
		'src/routes/api/fan-out/+server.ts': fanOutEndpoint
	},
	parallel: {
		'native/src/parallel.rs': parallelRs,
		'src/routes/api/parallel/+server.ts': parallelEndpoint
	},
	stream: {
		'native/src/stream.rs': streamRs,
		'src/routes/api/stream/+server.ts': streamEndpoint,
		'src/lib/server/sse.ts': sse
	},
	probe: {
		'native/src/probe.rs': probeRs,
		'src/routes/api/probe/+server.ts': probeEndpoint
	},
	metrics: {
		'native/src/runtime.rs': runtimeRs,
		'src/routes/api/pulse/+server.ts': pulseEndpoint
	}
} satisfies Record<string, Record<string, string>>;

export type Sources = Record<keyof typeof files, SourceFile[]>;

const languages: Record<string, string> = { rs: 'rust', ts: 'typescript', toml: 'toml' };

async function highlight(): Promise<Sources> {
	const highlighter = await createHighlighterCore({
		langs: [
			import('shiki/langs/rust.mjs'),
			import('shiki/langs/toml.mjs'),
			import('shiki/langs/typescript.mjs')
		],
		themes: [import('shiki/themes/github-light.mjs'), import('shiki/themes/github-dark.mjs')],
		engine: createJavaScriptRegexEngine()
	});

	const render = ([path, code]: [string, string]): SourceFile => ({
		path,
		html: highlighter.codeToHtml(code.trimEnd(), {
			lang: languages[path.split('.').pop()!],
			// Emit both palettes as CSS variables; app.css picks one per color scheme.
			themes: { light: 'github-light', dark: 'github-dark' },
			defaultColor: false
		})
	});

	return Object.fromEntries(
		Object.entries(files).map(([demo, sources]) => [demo, Object.entries(sources).map(render)])
	) as Sources;
}

/** Highlighted once per server start, then shared by every request. */
export const sources = highlight();
