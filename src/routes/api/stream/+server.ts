import { startStream } from '@napi-tokio/native';
import { int } from '#lib/server/input.ts';
import { eventStream } from '#lib/server/sse.ts';
import type { StreamEvent } from '#lib/types.ts';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = ({ url }) => {
	const options = {
		producers: int(url.searchParams.get('producers'), 1, 4, 3),
		samplesPerProducer: int(url.searchParams.get('samples'), 1, 200, 40),
		intervalMs: int(url.searchParams.get('intervalMs'), 20, 1000, 80)
	};

	return eventStream<StreamEvent>((send, close) => {
		// Rust calls this function from its tokio tasks, once per event.
		const stream = startStream(options, (event) => {
			send(event);
			if (event.type === 'end') close();
		});

		// If the browser disconnects first, cancel the Rust tasks.
		return () => stream.stop();
	});
};
