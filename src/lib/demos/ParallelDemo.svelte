<script lang="ts">
	import { resolve } from '$app/paths';
	import { post } from '#lib/api.ts';
	import { Call } from '#lib/call.svelte.ts';
	import Slider from '#lib/components/Slider.svelte';
	import { count, ms } from '#lib/format.ts';
	import type { ParallelReport } from '#lib/types.ts';

	/** Number of tokio workers. Each task wants a thread, so this is the natural batch size. */
	let { workers }: { workers: number } = $props();

	/** Each task counts the primes below this, on its own. */
	const LIMIT = 10_000_000;

	// svelte-ignore state_referenced_locally (the worker count is only the slider's starting point)
	let tasks = $state(workers);
	let yieldBetweenBatches = $state(true);

	const call = new Call<ParallelReport>();
	const report = $derived(call.result);
	const duration = (stint: { startedMs: number; finishedMs: number }) =>
		stint.finishedMs - stint.startedMs;

	/** One lane per thread that did any of the work, in thread order. */
	const lanes = $derived(
		[...Map.groupBy(report?.stints ?? [], (stint) => stint.thread)].sort(([a], [b]) =>
			a.localeCompare(b, undefined, { numeric: true })
		)
	);

	/** The same stints from each task's point of view. */
	const byTask = $derived(
		[...Map.groupBy(report?.stints ?? [], (stint) => stint.task)]
			.sort(([a], [b]) => a - b)
			.map(([task, stints]) => ({
				task,
				stints: stints.length,
				ranMs: stints.reduce((sum, stint) => sum + duration(stint), 0),
				threads: [...new Set(stints.map((stint) => stint.thread))]
			}))
	);
	const cpuMs = $derived(byTask.reduce((sum, task) => sum + task.ranMs, 0));

	/** The task under the pointer: its stints stay lit while the rest dim. */
	let followed = $state<number>();
	const following = $derived(byTask.find(({ task }) => task === followed));

	const percent = (valueMs: number) => (report ? (valueMs / report.wallMs) * 100 : 0);

	const run = () =>
		call.run(() =>
			post<ParallelReport>(resolve('/api/parallel'), { tasks, limit: LIMIT, yieldBetweenBatches })
		);
</script>

<div class="controls">
	<Slider
		label="Tasks"
		bind:value={tasks}
		min={1}
		max={Math.min(32, workers * 2)}
		disabled={call.pending}
	/>

	<label class="toggle">
		<input type="checkbox" bind:checked={yieldBetweenBatches} disabled={call.pending} />
		Let other tasks in between batches
	</label>
</div>

<div class="actions">
	<button class="primary" onclick={run} disabled={call.pending}>
		{call.pending ? 'Counting on every worker…' : 'Spawn tasks'}
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
			<dt>CPU time used</dt>
			<dd>{ms(cpuMs)}</dd>
		</div>
		<div>
			<dt>Workers used</dt>
			<dd>{lanes.length} of {workers}</dd>
		</div>
	</dl>

	<figure>
		<div class="lanes" role="presentation" onpointerleave={() => (followed = undefined)}>
			{#each lanes as [thread, stints] (thread)}
				<span class="thread">{thread}</span>
				<div class="lane">
					{#each stints as stint (stint.startedMs)}
						<!-- Pointing is a shortcut; the table below carries the same details. -->
						<i
							role="presentation"
							class:dimmed={followed !== undefined && stint.task !== followed}
							style:left="{percent(stint.startedMs)}%"
							style:width="calc({percent(duration(stint))}% - 2px)"
							onpointerenter={() => (followed = stint.task)}
						></i>
					{/each}
				</div>
			{/each}
			<span></span>
			<div class="axis" aria-hidden="true">
				<span>0 ms</span>
				<span>{ms(report.wallMs)}</span>
			</div>
		</div>

		<figcaption class="note">
			{#if following}
				Task {following.task} ran for {ms(following.ranMs)} in
				{following.stints === 1 ? 'one stint' : `${following.stints} stints`} on
				{following.threads.join(', ')}.
			{:else}
				One row per worker thread; each bar is a task running on it. Point at a bar to follow
				that task. Every task found {count(report.primes)} primes.
			{/if}
		</figcaption>
	</figure>

	<details>
		<summary>Per-task breakdown</summary>
		<table>
			<thead>
				<tr>
					<th scope="col">Task</th>
					<th scope="col">Ran for</th>
					<th scope="col">Stints</th>
					<th scope="col">Threads</th>
				</tr>
			</thead>
			<tbody>
				{#each byTask as { task, ranMs, stints, threads } (task)}
					<tr>
						<th scope="row">{task}</th>
						<td>{ms(ranMs)}</td>
						<td>{stints}</td>
						<td>{threads.join(', ')}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</details>
{:else}
	<p class="note">
		Each task counts the primes below {count(LIMIT / 1_000_000)} million, about a second of solid
		computing. Watch the worker meters in the monitor while they run.
	</p>
{/if}

<style>
	.toggle {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.875rem;
		font-weight: 500;
	}

	figure {
		margin: 0;
	}

	.lanes {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr);
		gap: 4px 0.6rem;
		align-items: center;
	}

	.thread {
		font-family: var(--mono);
		font-size: 0.7rem;
		color: var(--muted);
	}

	.lane {
		position: relative;
		height: 14px;
		border-radius: 3px;
		background: var(--sunken);
	}

	.lane i {
		position: absolute;
		inset-block: 0;
		min-width: 2px;
		border-radius: 3px;
		background: var(--rust);
		transition: opacity 120ms;
	}

	.lane i.dimmed {
		opacity: 0.25;
	}

	.lanes .axis {
		margin-top: 0.2rem;
	}

	figcaption {
		margin-top: 0.75rem;
		/* Room for two lines, so following a task doesn't make the page jump. */
		min-height: 2.75rem;
	}

	summary {
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
	}

	table {
		width: 100%;
		margin-top: 0.6rem;
		border-collapse: collapse;
		font-size: 0.8rem;
	}

	th,
	td {
		padding: 0.35rem 0.75rem 0.35rem 0;
		border-top: 1px solid var(--border);
		text-align: left;
		vertical-align: baseline;
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}

	/* Only the list of threads is long enough to need wrapping. */
	td:last-child {
		padding-right: 0;
		white-space: normal;
	}

	thead th {
		border-top: 0;
		font-weight: 500;
		color: var(--muted);
	}

	tbody th,
	td {
		font-family: var(--mono);
		font-weight: 400;
	}
</style>
