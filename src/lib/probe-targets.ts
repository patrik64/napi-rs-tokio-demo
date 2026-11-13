export interface ProbeTarget {
	address: string;
	note?: string;
}

/**
 * The only hosts the probe demo will ever dial. The endpoint checks requests
 * against this list so it can't be used to scan arbitrary hosts and ports.
 */
export const PROBE_TARGETS: ProbeTarget[] = [
	{ address: 'svelte.dev:443' },
	{ address: 'napi.rs:443' },
	{ address: 'tokio.rs:443' },
	{ address: 'crates.io:443' },
	{ address: 'registry.npmjs.org:443' },
	{ address: '10.255.255.1:81', note: 'unroutable, so it runs into the timeout' },
	{ address: 'no-such-host.invalid:443', note: 'never resolves' }
];
