<script lang="ts">
	import { resolve } from '$app/paths';
	import { post } from '#lib/api.ts';
	import { Call } from '#lib/call.svelte.ts';
	import { count, ms } from '#lib/format.ts';
	import type { PrimesResponse, PrimesVariant } from '#lib/types.ts';

	const sizes = [
		{ limit: 2_000_000, label: '2 million' },
		{ limit: 5_000_000, label: '5 million' },
		{ limit: 12_000_000, label: '12 million' }
	];
	let limit = $state(5_000_000);

	/** Above this, a person using the server would notice the pause. */
	const NOTICEABLE_MS = 50;

	const variants: { id: PrimesVariant; name: string; rust: string; call: Call<PrimesResponse> }[] =
		[
			{ id: 'sync', name: 'Synchronous export', rust: 'fn', call: new Call() },
			{ id: 'worker', name: 'Async export', rust: 'async fn', call: new Call() },
			{ id: 'blocking', name: 'Async export, blocking pool', rust: 'spawn_blocking', call: new Call() }
		];

	const run = (variant: (typeof variants)[number]) =>
		variant.call.run(() =>
			post<PrimesResponse>(resolve('/api/primes'), { variant: variant.id, limit })
		);
</script>

<fieldset>
	<legend>Count the primes below</legend>
	{#each sizes as size (size.limit)}
		<label>
			<input type="radio" bind:group={limit} value={size.limit} />
			{size.label}
		</label>
	{/each}
</fieldset>

<ul>
	{#each variants as variant (variant.id)}
		{@const { pending, result, error } = variant.call}
		<li>
			<div class="what">
				<strong>{variant.name}</strong>
				<code>{variant.rust}</code>
			</div>

			<button class="primary" onclick={() => run(variant)} disabled={pending}>
				{pending ? 'Counting…' : 'Run'}
			</button>

			<p class="note" aria-live="polite">
				{#if error}
					<span class="error">{error}</span>
				{:else if result}
					{count(result.primes)} primes in {ms(result.elapsedMs)} on <code>{result.thread}</code>.
					{#if result.loopLagMs > NOTICEABLE_MS}
						<strong class="bad">Node was frozen for {ms(result.loopLagMs)}.</strong>
					{:else}
						<strong class="good">Node stayed responsive.</strong>
					{/if}
				{:else if pending}
					Running…
				{:else}
					Not run yet.
				{/if}
			</p>
		</li>
	{/each}
</ul>

<style>
	fieldset {
		display: flex;
		flex-wrap: wrap;
		gap: 0.4rem 1.1rem;
		margin: 0;
		padding: 0;
		border: 0;
		font-size: 0.875rem;
	}

	legend {
		padding: 0;
		margin-bottom: 0.4rem;
		font-weight: 500;
	}

	label {
		display: flex;
		align-items: center;
		gap: 0.4rem;
	}

	ul {
		display: grid;
		padding: 0;
		list-style: none;
	}

	li {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 0.5rem 1rem;
		align-items: center;
		padding-block: 0.9rem;
		border-top: 1px solid var(--border);
	}

	li:last-child {
		padding-bottom: 0;
	}

	.what {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 0.25rem 0.6rem;
	}

	.what code {
		color: var(--rust);
	}

	.note {
		grid-column: 1 / -1;
	}

	.good {
		color: var(--good);
	}

	.bad {
		color: var(--bad);
	}
</style>
