<script lang="ts">
	import type { SourceFile } from '#lib/types.ts';

	let { files }: { files: SourceFile[] } = $props();

	let active = $state(0);
	const id = $props.id();

	/** `src/routes/api/delay/+server.ts` is only recognizable with its folder. */
	function tabName(path: string) {
		const parts = path.split('/');
		return parts.slice(path.endsWith('+server.ts') ? -2 : -1).join('/');
	}
</script>

<div class="code">
	<div class="tabs" role="tablist" aria-label="Source files">
		{#each files as file, index (file.path)}
			<button
				role="tab"
				id="{id}-tab-{index}"
				aria-selected={index === active}
				aria-controls="{id}-panel"
				title={file.path}
				onclick={() => (active = index)}
			>
				{tabName(file.path)}
			</button>
		{/each}
	</div>

	<div class="panel" role="tabpanel" id="{id}-panel" aria-labelledby="{id}-tab-{active}">
		<!-- Highlighted on the server from this project's own files. -->
		{@html files[active].html}
	</div>
</div>

<style>
	.code {
		min-width: 0;
		border: 1px solid var(--border);
		border-radius: var(--radius);
		background: var(--surface);
		overflow: hidden;
	}

	.tabs {
		display: flex;
		background: var(--sunken);
		border-bottom: 1px solid var(--border);
		overflow-x: auto;
	}

	button {
		padding: 0.6rem 0.9rem;
		border: 0;
		border-radius: 0;
		background: none;
		color: var(--muted);
		font-family: var(--mono);
		font-size: 0.78rem;
		font-weight: 500;
	}

	button:hover {
		color: var(--text);
	}

	button[aria-selected='true'] {
		color: var(--text);
		box-shadow: inset 0 -2px 0 var(--rust);
	}

	button:focus-visible {
		outline-offset: -2px;
	}

	.panel {
		max-height: 31rem;
		overflow: auto;
	}
</style>
