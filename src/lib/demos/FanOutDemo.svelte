<script lang="ts">
	import { resolve } from '$app/paths';
	import { post } from '#lib/api.ts';
	import { Call } from '#lib/call.svelte.ts';
	import Slider from '#lib/components/Slider.svelte';
	import { ms } from '#lib/format.ts';
	import type { FanOutReport } from '#lib/types.ts';

	/** Each task sleeps for a random time in this range. */
	const MIN_MS = 150;
	const MAX_MS = 600;

	let tasks = $state(16);
	let maxConcurrency = $state(4);

	const call = new Call<FanOutReport>();
	const report = $derived(call.result);
	const threads = $derived(new Set(report?.spans.map((span) => span.thread)).size);

	const run = () =>
		call.run(() =>
			post<FanOutReport>(resolve('/api/fan-out'), {
				tasks,
				maxConcurrency,
				minMs: MIN_MS,
				maxMs: MAX_MS
			})
		);
</script>

<div class="controls">
	<Slider label="Tasks" bind:value={tasks} min={1} max={32} disabled={call.pending} />
	<Slider
		label="Running at once, at most"
		bind:value={maxConcurrency}
		min={1}
		max={16}
		disabled={call.pending}
	/>
</div>

<div class="actions">
	<button class="primary" onclick={run} disabled={call.pending}>
		{call.pending ? 'Waiting for all tasks…' : 'Spawn tasks'}
	</button>
	{#if call.error}<p class="error">{call.error}</p>{/if}
</div>

{#if report}
	<dl class="stats">
		<div>
			<dt>Took</dt>
			<dd>{ms(report.wallMs)}</dd>
		</div>
		<div>
			<dt>One at a time would take</dt>
			<dd>{ms(report.workMs)}</dd>
		</div>
		<div>
			<dt>Threads used</dt>
			<dd>{threads}</dd>
		</div>
	</dl>

	<figure>
		<ol>
			{#each report.spans as span (span.id)}
				<li
					title="task {span.id}: waited {ms(span.startedMs)}, ran {ms(
						span.finishedMs - span.startedMs
					)} on {span.thread}"
				>
					<i class="waiting" style:width="{(span.startedMs / report.wallMs) * 100}%"></i>
					<i
						class="running"
						style:left="{(span.startedMs / report.wallMs) * 100}%"
						style:width="{((span.finishedMs - span.startedMs) / report.wallMs) * 100}%"
					></i>
				</li>
			{/each}
		</ol>
		<div class="axis" aria-hidden="true">
			<span>0 ms</span>
			<span>{ms(report.wallMs)}</span>
		</div>
		<figcaption class="note">
			One row per task. The thin line is time spent waiting for a permit; the bar is the task
			running.
		</figcaption>
	</figure>
{:else}
	<p class="note">
		Each task sleeps for a random {MIN_MS} to {MAX_MS} ms. Run it to see when each one got its turn.
	</p>
{/if}

<style>
	figure {
		margin: 0;
	}

	ol {
		display: grid;
		gap: 3px;
		padding: 0;
		list-style: none;
	}

	li {
		position: relative;
		height: 7px;
	}

	i {
		position: absolute;
		top: 0;
		height: 100%;
	}

	.waiting {
		left: 0;
		top: 3px;
		height: 1px;
		background: var(--border);
	}

	.running {
		min-width: 2px;
		border-radius: 2px;
		background: var(--rust);
	}

	figcaption {
		margin-top: 0.75rem;
	}
</style>
