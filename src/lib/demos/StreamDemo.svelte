<script lang="ts">
	import { resolve } from '$app/paths';
	import Slider from '#lib/components/Slider.svelte';
	import { count } from '#lib/format.ts';
	import type { StreamEvent } from '#lib/types.ts';

	/** Producer 0 ticks at this rate; each further producer is proportionally slower. */
	const INTERVAL_MS = 80;
	/** How much history the chart shows. */
	const WINDOW_MS = 8000;
	/** Chart size in SVG units, and the largest value a sample can take. */
	const WIDTH = 600;
	const HEIGHT = 150;
	const PEAK = 1.3;

	let producers = $state(3);
	let samples = $state(40);

	type Status = 'idle' | 'running' | 'ended' | 'stopped' | 'failed';
	let status = $state<Status>('idle');
	let received = $state(0);
	/** Per producer: how many samples arrived, and the recent ones as [atMs, value]. */
	let series = $state.raw<{ total: number; expected: number; recent: [number, number][] }[]>([]);
	let latestMs = $state(0);
	let source: EventSource | undefined;

	const running = $derived(status === 'running');
	const windowEndMs = $derived(Math.max(latestMs, WINDOW_MS));

	function line(points: [number, number][]) {
		return points
			.map(([atMs, value]) => {
				const x = WIDTH - ((windowEndMs - atMs) / WINDOW_MS) * WIDTH;
				const y = HEIGHT / 2 - (value / PEAK) * (HEIGHT / 2);
				return `${x.toFixed(1)},${y.toFixed(1)}`;
			})
			.join(' ');
	}

	function start() {
		const expected = samples;
		series = Array.from({ length: producers }, () => ({ total: 0, expected, recent: [] }));
		received = 0;
		latestMs = 0;
		status = 'running';

		const query = new URLSearchParams({
			producers: String(producers),
			samples: String(samples),
			intervalMs: String(INTERVAL_MS)
		});
		source = new EventSource(`${resolve('/api/stream')}?${query}`);

		source.onmessage = (message) => {
			const event: StreamEvent = JSON.parse(message.data);
			if (event.type === 'end') return finish('ended');

			received += 1;
			latestMs = event.atMs;
			series = series.map((one, producer) =>
				producer === event.producer
					? {
							...one,
							total: one.total + 1,
							recent: [
								...one.recent.filter(([atMs]) => atMs > event.atMs - WINDOW_MS),
								[event.atMs, event.value]
							]
						}
					: one
			);
		};

		// EventSource reconnects by default, which would start a second stream.
		source.onerror = () => finish('failed');
	}

	function finish(next: Status) {
		// Closing the connection is all it takes: the endpoint sees the
		// disconnect and cancels the Rust tasks.
		source?.close();
		source = undefined;
		status = next;
	}

	// Don't leave a stream running when the component goes away.
	$effect(() => () => source?.close());
</script>

<div class="controls">
	<Slider label="Producer tasks" bind:value={producers} min={1} max={4} disabled={running} />
	<Slider
		label="Samples from each"
		bind:value={samples}
		min={10}
		max={200}
		step={10}
		disabled={running}
	/>
</div>

<div class="actions">
	{#if running}
		<button onclick={() => finish('stopped')}>Stop</button>
	{:else}
		<button class="primary" onclick={start}>Start stream</button>
	{/if}

	<p class="note" aria-live="polite">
		{#if status === 'running'}
			{count(received)} events received.
		{:else if status === 'ended'}
			Ended on its own after {count(received)} events, once every producer had finished.
		{:else if status === 'stopped'}
			Stopped after {count(received)} events. The Rust tasks were cancelled.
		{:else if status === 'failed'}
			<span class="error">The connection dropped.</span>
		{:else}
			Nothing is running.
		{/if}
	</p>
</div>

<figure>
	<svg viewBox="0 0 {WIDTH} {HEIGHT}" preserveAspectRatio="none" role="img" aria-label="Live samples per producer">
		<line class="zero" x1="0" x2={WIDTH} y1={HEIGHT / 2} y2={HEIGHT / 2} />
		{#each series as { recent }, producer (producer)}
			<polyline points={line(recent)} style:stroke="var(--series-{producer})" />
		{/each}
	</svg>

	{#if series.length > 0}
		<figcaption>
			{#each series as { total, expected }, producer (producer)}
				<span>
					<i style:background="var(--series-{producer})"></i>
					every {INTERVAL_MS * (producer + 1)} ms
					<b>{total}/{expected}</b>
				</span>
			{/each}
		</figcaption>
	{/if}
</figure>

<style>
	figure {
		margin: 0;
	}

	svg {
		display: block;
		width: 100%;
		height: 9.5rem;
		border-radius: 6px;
		background: var(--sunken);
		box-shadow: inset 0 0 0 1px var(--border);
	}

	.zero {
		stroke: var(--border);
		vector-effect: non-scaling-stroke;
	}

	polyline {
		fill: none;
		stroke-width: 2;
		stroke-linejoin: round;
		stroke-linecap: round;
		vector-effect: non-scaling-stroke;
	}

	figcaption {
		display: flex;
		flex-wrap: wrap;
		gap: 0.3rem 1.1rem;
		margin-top: 0.6rem;
		font-size: 0.8rem;
		color: var(--muted);
	}

	figcaption span {
		display: flex;
		align-items: center;
		gap: 0.4rem;
	}

	figcaption i {
		width: 0.7rem;
		height: 3px;
		border-radius: 2px;
	}

	figcaption b {
		font-family: var(--mono);
		font-weight: 500;
		font-variant-numeric: tabular-nums;
		color: var(--text);
	}

	.note {
		flex: 1;
		min-width: 12rem;
	}
</style>
