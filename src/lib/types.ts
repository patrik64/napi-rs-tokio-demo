// Shapes of the JSON the endpoints return. The Rust-side types come straight
// from the `index.d.ts` that napi-rs generates; these imports are type-only,
// so the addon itself never ends up in the browser bundle.
import type { PrimeCount, ProbeResult } from '@napi-tokio/native';

export type {
	FanOutReport,
	ParallelReport,
	Stint,
	StreamEvent,
	TaskSpan
} from '@napi-tokio/native';

export type DelayResponse =
	| { outcome: 'resolved'; elapsedMs: number }
	| { outcome: 'rejected'; elapsedMs: number; message: string; code: string };

export type PrimesVariant = 'sync' | 'worker' | 'blocking';

export interface PrimesResponse extends PrimeCount {
	/** Longest the Node event loop was held up while the count ran, in ms. */
	loopLagMs: number;
}

export interface ProbeResponse {
	results: ProbeResult[];
	wallMs: number;
}

/** One frame of the live monitor. */
export interface Pulse {
	/** Longest the Node event loop was held up since the previous pulse, in ms. */
	loopLagMs: number;
	aliveTasks: number;
	queueDepth: number;
	/** How loaded each tokio worker was over the last interval, 0 to 1. */
	workerLoad: number[];
}

/** A source file rendered to highlighted HTML on the server. */
export interface SourceFile {
	path: string;
	html: string;
}
