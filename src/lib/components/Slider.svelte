<script lang="ts">
	import { count } from '#lib/format.ts';

	interface Props {
		label: string;
		value: number;
		min: number;
		max: number;
		step?: number;
		unit?: string;
		disabled?: boolean;
	}

	let { label, value = $bindable(), min, max, step = 1, unit, disabled }: Props = $props();

	const id = $props.id();
</script>

<div class="slider">
	<label for={id}>{label}</label>
	<output for={id}>{count(value)}{unit ? ` ${unit}` : ''}</output>
	<input {id} type="range" bind:value {min} {max} {step} {disabled} />
</div>

<style>
	.slider {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 0.15rem 1rem;
		align-items: baseline;
	}

	label {
		font-size: 0.875rem;
		font-weight: 500;
	}

	output {
		font-family: var(--mono);
		font-size: 0.875rem;
		font-variant-numeric: tabular-nums;
		color: var(--muted);
	}

	input {
		grid-column: 1 / -1;
		width: 100%;
		margin: 0;
	}
</style>
