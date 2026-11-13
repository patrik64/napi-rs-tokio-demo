<script lang="ts">
	import { resolve } from '$app/paths';
	import { post } from '#lib/api.ts';
	import { Call } from '#lib/call.svelte.ts';
	import Slider from '#lib/components/Slider.svelte';
	import { ms } from '#lib/format.ts';
	import type { DelayResponse } from '#lib/types.ts';

	let workMs = $state(1200);
	let hasDeadline = $state(true);
	let deadlineMs = $state(2000);

	const request = $derived({ workMs, deadlineMs: hasDeadline ? deadlineMs : null });

	const call = new Call<DelayResponse>();
	/** The request the current call or result belongs to. */
	let sent = $state.raw<{ workMs: number; deadlineMs: number | null }>();
	let runningForMs = $state(0);

	// Moving a slider afterwards makes the last outcome describe something else, so hide it.
	const upToDate = $derived(
		sent?.workMs === request.workMs && sent.deadlineMs === request.deadlineMs
	);
	const outcome = $derived(upToDate && !call.pending ? call.result : undefined);

	// The track is drawn to scale: its full width is a little more than the longest duration on it.
	const scaleMs = $derived(Math.max(request.workMs, request.deadlineMs ?? 0, 1) * 1.12);
	const percent = (valueMs: number) => `${Math.min(100, (valueMs / scaleMs) * 100)}%`;
	const filledMs = $derived(call.pending ? runningForMs : (outcome?.elapsedMs ?? 0));

	async function run() {
		sent = request;
		const started = performance.now();
		const clock = setInterval(() => (runningForMs = performance.now() - started), 30);
		runningForMs = 0;
		await call.run(() => post<DelayResponse>(resolve('/api/delay'), sent));
		clearInterval(clock);
	}
</script>

<div class="controls">
	<Slider
		label="Work takes"
		bind:value={workMs}
		min={100}
		max={3000}
		step={100}
		unit="ms"
		disabled={call.pending}
	/>

	<label class="toggle">
		<input type="checkbox" bind:checked={hasDeadline} disabled={call.pending} />
		Give up at a deadline
	</label>

	{#if hasDeadline}
		<Slider
			label="Deadline"
			bind:value={deadlineMs}
			min={100}
			max={3000}
			step={100}
			unit="ms"
			disabled={call.pending}
		/>
	{/if}
</div>

<div class="track" aria-hidden="true">
	<div class="work" style:width={percent(request.workMs)}></div>
	<div class="fill {outcome?.outcome ?? ''}" style:width={percent(filledMs)}></div>
	{#if request.deadlineMs !== null}
		<div class="deadline" style:left={percent(request.deadlineMs)}></div>
	{/if}
</div>

<div class="actions">
	<button class="primary" onclick={run} disabled={call.pending}>
		{call.pending ? 'Awaiting…' : 'Call delay()'}
	</button>

	<p class="note" aria-live="polite">
		{#if call.pending}
			The promise is pending.
		{:else if call.error}
			<span class="error">{call.error}</span>
		{:else if outcome?.outcome === 'resolved'}
			<strong class="good">Resolved</strong> after {ms(outcome.elapsedMs)}.
		{:else if outcome?.outcome === 'rejected'}
			<strong class="bad">Rejected</strong> after {ms(outcome.elapsedMs)}:
			“{outcome.message}” <code>{outcome.code}</code>
		{:else}
			The shaded bar is the work; the line is the deadline.
		{/if}
	</p>
</div>

<style>
	.toggle {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.875rem;
		font-weight: 500;
	}

	.track {
		position: relative;
		height: 1.75rem;
		border-radius: 6px;
		background: var(--sunken);
		box-shadow: inset 0 0 0 1px var(--border);
	}

	.work,
	.fill {
		position: absolute;
		inset-block: 0;
		left: 0;
		border-radius: 6px;
	}

	.work {
		background: color-mix(in srgb, var(--rust) 16%, transparent);
	}

	.fill {
		background: var(--rust);
	}

	.fill.resolved {
		background: var(--good);
	}

	.fill.rejected {
		background: var(--bad);
	}

	.deadline {
		position: absolute;
		inset-block: -0.3rem;
		width: 2px;
		margin-left: -1px;
		background: var(--text);
	}

	.note {
		flex: 1;
		min-width: 12rem;
	}

	.good {
		color: var(--good);
	}

	.bad {
		color: var(--bad);
	}
</style>
