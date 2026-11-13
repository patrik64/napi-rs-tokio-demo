<script lang="ts">
	import { resolve } from '$app/paths';
	import { ms } from '#lib/format.ts';
	import type { Pulse } from '#lib/types.ts';

	/** Number of tokio workers, so the meters can render before the first pulse. */
	let { workers }: { workers: number } = $props();

	/** Readings kept for the strip: 30 seconds at four a second. */
	const HISTORY = 120;
	/** A delay this long is drawn as a problem. */
	const LATE_MS = 50;
	/** A worker that only woke for a moment still gets this much of its meter, in percent. */
	const IDLE_SLIVER = 14;
	/** Pulses arrive every 250 ms; this much silence means the server is stuck. */
	const STALLED_AFTER_MS = 700;

	let pulses = $state.raw<Pulse[]>([]);
	let connected = $state(false);
	let lastPulseAt = $state(0);
	let now = $state(0);

	const latest = $derived(pulses.at(-1));
	const load = $derived(latest?.workerLoad ?? Array<number>(workers).fill(0));
	const silentMs = $derived(connected && latest ? now - lastPulseAt : 0);
	const stalled = $derived(silentMs > STALLED_AFTER_MS);

	$effect(() => {
		const source = new EventSource(resolve('/api/pulse'));
		source.onopen = () => (connected = true);
		source.onerror = () => (connected = false);
		source.onmessage = (message) => {
			pulses = [...pulses.slice(1 - HISTORY), JSON.parse(message.data)];
			lastPulseAt = now = performance.now();
		};

		// A blocked server sends nothing at all, so silence has to be noticed here.
		const clock = setInterval(() => (now = performance.now()), 100);

		return () => {
			source.close();
			clearInterval(clock);
		};
	});

	/** Log scale, so 1 ms is visible and 2 s still fits. */
	function barHeight(lagMs: number) {
		return Math.max(8, Math.min(100, (Math.log10(1 + lagMs) / Math.log10(2000)) * 100));
	}
</script>

<aside aria-label="Live runtime monitor">
	<div class="inner">
		<div class="group loop">
			<span class="label"><i class="dot node"></i>Node event loop</span>
			<div class="strip" class:stalled aria-hidden="true">
				{#each pulses as pulse, index (index)}
					<i class:late={pulse.loopLagMs > LATE_MS} style:height="{barHeight(pulse.loopLagMs)}%"></i>
				{/each}
			</div>
			<span class="value" class:bad={stalled || (latest?.loopLagMs ?? 0) > LATE_MS}>
				{#if stalled}
					blocked {(silentMs / 1000).toFixed(1)} s
				{:else if latest}
					{ms(latest.loopLagMs)}
				{:else}
					connecting
				{/if}
			</span>
		</div>

		<div class="group">
			<span class="label"><i class="dot rust"></i>tokio workers</span>
			<div class="meters">
				{#each load as share, worker (worker)}
					<i title="worker {worker}: {Math.round(share * 100)}% busy">
						<b style:height="{share > 0 ? Math.max(IDLE_SLIVER, share * 100) : 0}%"></b>
					</i>
				{/each}
			</div>
		</div>

		<div class="group">
			<span class="label">tasks alive</span>
			<span class="value">{latest?.aliveTasks ?? 0}</span>
		</div>

		<div class="group queued">
			<span class="label">queued</span>
			<span class="value">{latest?.queueDepth ?? 0}</span>
		</div>
	</div>
</aside>

<style>
	aside {
		position: sticky;
		top: 0;
		z-index: 10;
		border-block: 1px solid var(--border);
		background: color-mix(in srgb, var(--bg) 88%, transparent);
		backdrop-filter: blur(10px);
	}

	.inner {
		display: flex;
		align-items: center;
		gap: 0.5rem clamp(1rem, 3vw, 2rem);
		max-width: var(--page);
		margin-inline: auto;
		padding: 0.55rem var(--gutter);
	}

	.group {
		display: grid;
		grid-template-columns: auto auto;
		gap: 0 0.6rem;
		align-items: center;
	}

	.loop {
		flex: 1;
		min-width: 0;
		grid-template-columns: minmax(0, 1fr) auto;
	}

	.label {
		grid-column: 1 / -1;
		display: flex;
		align-items: center;
		gap: 0.35rem;
		font-size: 0.7rem;
		color: var(--muted);
		white-space: nowrap;
	}

	.dot {
		width: 0.45rem;
		height: 0.45rem;
		border-radius: 50%;
	}

	.dot.node {
		background: var(--node);
	}

	.dot.rust {
		background: var(--rust);
	}

	.value {
		font-family: var(--mono);
		font-size: 0.85rem;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
		line-height: 1.6rem;
		white-space: nowrap;
	}

	.loop .value {
		min-width: 6.5rem;
		text-align: right;
	}

	.value.bad {
		color: var(--bad);
	}

	.strip {
		display: flex;
		align-items: flex-end;
		justify-content: flex-end;
		gap: 2px;
		height: 1.6rem;
		overflow: hidden;
	}

	.strip i {
		flex: none;
		width: 3px;
		border-radius: 1px;
		background: var(--node);
	}

	.strip i.late {
		background: var(--bad);
	}

	.strip.stalled {
		opacity: 0.35;
	}

	.meters {
		grid-column: 1 / -1;
		display: flex;
		gap: 3px;
		height: 1.6rem;
	}

	.meters i {
		display: flex;
		align-items: flex-end;
		width: 9px;
		border-radius: 2px;
		background: var(--sunken);
		box-shadow: inset 0 0 0 1px var(--border);
		overflow: hidden;
	}

	.meters b {
		width: 100%;
		background: var(--rust);
		transition: height 200ms ease-out;
	}

	@media (max-width: 34rem) {
		.queued {
			display: none;
		}
	}
</style>
