import { runtimeMetrics } from '@napi-tokio/native';
import { watchLoopLag } from '#lib/server/loop-lag.ts';
import { eventStream } from '#lib/server/sse.ts';
import type { Pulse } from '#lib/types.ts';
import type { RequestHandler } from './$types';

const PULSE_MS = 250;

/** Feeds the monitor: one frame of Node and tokio vitals every 250 ms. */
export const GET: RequestHandler = () =>
	eventStream<Pulse>((send) => {
		const lag = watchLoopLag();
		let previous = { at: performance.now(), busyMs: runtimeMetrics().workerBusyMs };

		const timer = setInterval(() => {
			const metrics = runtimeMetrics();
			const at = performance.now();
			const elapsed = at - previous.at;

			send({
				loopLagMs: lag.take(),
				aliveTasks: metrics.aliveTasks,
				queueDepth: metrics.globalQueueDepth,
				// tokio reports cumulative busy time; the change since the last
				// frame, over the time that passed, is how loaded each worker was.
				// That total only moves when a worker pauses, so one that is in the
				// middle of a long task is counted as fully loaded instead.
				workerLoad: metrics.workerBusyMs.map((busyMs, worker) =>
					metrics.workerActive[worker]
						? 1
						: Math.min(1, (busyMs - previous.busyMs[worker]) / elapsed)
				)
			});
			previous = { at, busyMs: metrics.workerBusyMs };
		}, PULSE_MS);

		return () => {
			clearInterval(timer);
			lag.stop();
		};
	});
