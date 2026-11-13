<script lang="ts">
	import { resolve } from '$app/paths';
	import { post } from '#lib/api.ts';
	import { Call } from '#lib/call.svelte.ts';
	import Slider from '#lib/components/Slider.svelte';
	import { ms } from '#lib/format.ts';
	import { PROBE_TARGETS } from '#lib/probe-targets.ts';
	import type { ProbeResponse } from '#lib/types.ts';

	let selected = $state(PROBE_TARGETS.map((target) => target.address));
	let timeoutMs = $state(1500);

	const call = new Call<ProbeResponse>();
	const response = $derived(call.result);
	const oneByOneMs = $derived(
		response?.results.reduce((sum, result) => sum + result.totalMs, 0) ?? 0
	);

	const run = () =>
		call.run(() =>
			post<ProbeResponse>(resolve('/api/probe'), { targets: selected, timeoutMs })
		);
</script>

<fieldset disabled={call.pending}>
	<legend>Hosts to connect to</legend>
	{#each PROBE_TARGETS as target (target.address)}
		<label>
			<input type="checkbox" bind:group={selected} value={target.address} />
			<code>{target.address}</code>
			{#if target.note}<span class="note">{target.note}</span>{/if}
		</label>
	{/each}
</fieldset>

<Slider
	label="Timeout for each"
	bind:value={timeoutMs}
	min={250}
	max={3000}
	step={250}
	unit="ms"
	disabled={call.pending}
/>

<div class="actions">
	<button class="primary" onclick={run} disabled={call.pending || selected.length === 0}>
		{call.pending ? 'Connecting…' : 'Probe all at once'}
	</button>
	{#if call.error}<p class="error">{call.error}</p>{/if}
</div>

{#if response}
	<dl class="stats">
		<div>
			<dt>Took</dt>
			<dd>{ms(response.wallMs)}</dd>
		</div>
		<div>
			<dt>One at a time would take</dt>
			<dd>{ms(oneByOneMs)}</dd>
		</div>
	</dl>

	<ul class="results">
		{#each response.results as result (result.target)}
			<li class:failed={result.error}>
				<code>{result.target}</code>
				<span class="timing">
					{#if result.resolveMs !== undefined}lookup {ms(result.resolveMs)}{/if}
					{#if result.connectMs !== undefined}· connect {ms(result.connectMs)}{/if}
				</span>
				<span class="outcome">{result.error ?? `Connected to ${result.address}`}</span>
			</li>
		{/each}
	</ul>
{/if}

<style>
	fieldset {
		display: grid;
		gap: 0.45rem;
		margin: 0;
		padding: 0;
		border: 0;
		font-size: 0.875rem;
	}

	legend {
		padding: 0;
		margin-bottom: 0.5rem;
		font-weight: 500;
	}

	label {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 0.1rem 0.5rem;
	}

	.results {
		display: grid;
		padding: 0;
		list-style: none;
		font-size: 0.85rem;
	}

	.results li {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		align-items: baseline;
		gap: 0.1rem 1rem;
		padding-block: 0.55rem;
		border-top: 1px solid var(--border);
	}

	.results li:last-child {
		padding-bottom: 0;
	}

	/* Match the plain host names in the checklist above. */
	.results code {
		padding: 0;
		border: 0;
		background: none;
	}

	.timing {
		font-family: var(--mono);
		font-size: 0.78rem;
		font-variant-numeric: tabular-nums;
		color: var(--muted);
	}

	.outcome {
		flex-basis: 100%;
		overflow-wrap: anywhere;
		color: var(--good);
	}

	.failed .outcome {
		color: var(--bad);
	}
</style>
