/**
 * Measures how long Node's event loop is held up.
 *
 * A timer is asked to fire every `resolutionMs`. It can only fire when the
 * loop is free, so however late it runs is how long the loop was blocked.
 */
export function watchLoopLag(resolutionMs = 10) {
	let last = performance.now();
	let worst = 0;

	const sample = () => {
		const now = performance.now();
		worst = Math.max(worst, now - last - resolutionMs);
		last = now;
	};
	const timer = setInterval(sample, resolutionMs);

	return {
		/** Returns the worst delay since the previous call, in ms, and starts over. */
		take() {
			// Sample right now as well: straight after a synchronous call the
			// timer is overdue but has not had a chance to fire yet.
			sample();
			const result = worst;
			worst = 0;
			return result;
		},
		stop() {
			clearInterval(timer);
		}
	};
}
