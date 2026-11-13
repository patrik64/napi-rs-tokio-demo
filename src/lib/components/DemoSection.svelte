<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { SourceFile } from '#lib/types.ts';
	import CodeTabs from './CodeTabs.svelte';

	interface Props {
		id: string;
		/** Two-digit label shown beside the title, e.g. `01`. */
		index: string;
		title: string;
		summary: Snippet;
		sources: SourceFile[];
		/** The interactive part of the demo. */
		children: Snippet;
	}

	let { id, index, title, summary, sources, children }: Props = $props();
</script>

<section {id} aria-labelledby="{id}-title">
	<header>
		<span class="index" aria-hidden="true">{index}</span>
		<h2 id="{id}-title">{title}</h2>
		<p>{@render summary()}</p>
	</header>

	<div class="body">
		<div class="stage">{@render children()}</div>
		<CodeTabs files={sources} />
	</div>
</section>

<style>
	section {
		padding-block: clamp(2.5rem, 6vw, 4rem);
		border-top: 1px solid var(--border);
		/* Anchored sections stop with their top edge tucked under the sticky monitor. */
		scroll-margin-top: 3rem;
	}

	/* The monitor's own border already separates the first section from the hero. */
	section:first-child {
		border-top: 0;
	}

	header {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 0.4rem 0.9rem;
		align-items: baseline;
		margin-bottom: 1.5rem;
	}

	.index {
		font-family: var(--mono);
		font-size: 0.85rem;
		font-weight: 600;
		color: var(--rust);
	}

	h2 {
		font-size: clamp(1.35rem, 3vw, 1.7rem);
		line-height: 1.2;
		letter-spacing: -0.015em;
	}

	header p {
		grid-column: 2;
		max-width: 62ch;
		color: var(--muted);
	}

	.body {
		display: grid;
		gap: 1.25rem;
		align-items: start;
	}

	.stage {
		min-width: 0;
		display: grid;
		gap: 1.25rem;
		padding: 1.25rem;
		border: 1px solid var(--border);
		border-radius: var(--radius);
		background: var(--surface);
	}

	@media (min-width: 60rem) {
		.body {
			grid-template-columns: minmax(0, 5fr) minmax(0, 7fr);
		}
	}

	@media (max-width: 40rem) {
		header p {
			grid-column: 1 / -1;
		}
	}
</style>
