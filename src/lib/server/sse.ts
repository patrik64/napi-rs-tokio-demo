/**
 * Turns a push-style source into a Server-Sent Events response.
 *
 * `subscribe` is given `send` and `close`, and returns a cleanup function that
 * runs when the client goes away. Calls to `send` after that are ignored, so
 * a source that is still winding down can't write to a closed stream.
 */
export function eventStream<T>(
	subscribe: (send: (data: T) => void, close: () => void) => () => void
): Response {
	const encoder = new TextEncoder();
	let open = true;
	let cleanup = () => {};

	const body = new ReadableStream<Uint8Array>({
		start(controller) {
			cleanup = subscribe(
				(data) => {
					if (open) controller.enqueue(encoder.encode(`data: ${JSON.stringify(data)}\n\n`));
				},
				() => {
					if (open) controller.close();
					open = false;
				}
			);
		},
		cancel() {
			open = false;
			cleanup();
		}
	});

	return new Response(body, {
		headers: {
			'content-type': 'text/event-stream',
			'cache-control': 'no-cache'
		}
	});
}
