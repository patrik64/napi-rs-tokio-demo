<script lang="ts">
	import DemoSection from '#lib/components/DemoSection.svelte';
	import Monitor from '#lib/components/Monitor.svelte';
	import DelayDemo from '#lib/demos/DelayDemo.svelte';
	import FanOutDemo from '#lib/demos/FanOutDemo.svelte';
	import ParallelDemo from '#lib/demos/ParallelDemo.svelte';
	import PrimesDemo from '#lib/demos/PrimesDemo.svelte';
	import ProbeDemo from '#lib/demos/ProbeDemo.svelte';
	import StreamDemo from '#lib/demos/StreamDemo.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const contents = [
		['setup', 'Setup'],
		['delay', 'Promises'],
		['primes', 'Blocking work'],
		['fan-out', 'Concurrency'],
		['parallel', 'Parallelism'],
		['stream', 'Callbacks'],
		['probe', 'Network I/O'],
		['metrics', 'Metrics']
	];
</script>

<svelte:head>
	<title>napi-rs + tokio in SvelteKit</title>
	<meta
		name="description"
		content="Live demos of async Rust running on tokio inside a SvelteKit server, exported to JavaScript with napi-rs."
	/>
</svelte:head>

<header class="page hero">
	<p class="eyebrow">napi-rs with the <code>tokio_rt</code> feature</p>
	<h1>Rust futures, awaited from SvelteKit</h1>
	<p class="lede">
		napi-rs compiles Rust into a native Node addon. With its <code>tokio_rt</code> feature on, every
		<code>async fn</code> you export becomes a JavaScript function that returns a promise, and a multi-threaded
		tokio runtime inside the Node process does the work. Everything on this page is live: each button
		calls a SvelteKit endpoint on this server, and the endpoint calls into Rust.
	</p>

	<ul class="versions" aria-label="Versions in use">
		{#each Object.entries(data.versions) as [name, version] (name)}
			<li>{name} <b>{version}</b></li>
		{/each}
	</ul>

	<nav aria-label="On this page">
		{#each contents as [id, label] (id)}
			<a href="#{id}">{label}</a>
		{/each}
	</nav>
</header>

<Monitor workers={data.workers} />

<main class="page">
	<DemoSection id="setup" index="00" title="One feature flag" sources={data.sources.setup}>
		{#snippet summary()}
			Turn on <code>tokio_rt</code> for the <code>napi</code> crate and write ordinary async Rust.
			<code>napi build</code> compiles it and generates a loader plus TypeScript types for every export.
			Supplying your own runtime is optional; this project does it to number the threads.
		{/snippet}

		<h3>What happens on a call</h3>
		<ol class="path">
			<li>
				<b>Browser</b>
				A component sends a request to a SvelteKit endpoint.
			</li>
			<li class="node">
				<b>Node</b>
				The endpoint imports the Rust function like any other module and awaits it.
			</li>
			<li class="rust">
				<b>napi-rs</b>
				Converts the arguments, spawns the future on tokio and returns a promise straight away.
			</li>
			<li class="rust">
				<b>tokio</b>
				One of {data.workers} worker threads runs the future. These are tokio's own threads,
				separate from libuv's thread pool, and Node stays free to serve other requests.
			</li>
			<li class="node">
				<b>Node</b>
				When the future finishes, napi-rs settles the promise on the JS thread and the endpoint responds.
			</li>
		</ol>
	</DemoSection>

	<DemoSection id="delay" index="01" title="An async fn becomes a promise" sources={data.sources.delay}>
		{#snippet summary()}
			<code>delay</code> sleeps on a tokio timer, optionally under a deadline. Finishing in time resolves
			the promise. Missing the deadline returns <code>Err</code>, which rejects it.
		{/snippet}
		<DelayDemo />
	</DemoSection>

	<DemoSection id="primes" index="02" title="Blocking work, three ways" sources={data.sources.primes}>
		{#snippet summary()}
			One CPU-heavy function, exported three ways. What changes is which thread does the counting,
			and so what gets stuck waiting behind it. Keep an eye on the monitor above while each one runs.
		{/snippet}
		<PrimesDemo />
	</DemoSection>

	<DemoSection id="fan-out" index="03" title="Many tasks, one await" sources={data.sources.fanOut}>
		{#snippet summary()}
			A batch of tasks goes into a <code>JoinSet</code>, and a <code>Semaphore</code> caps how many run
			at once. The tasks only wait, so it takes very few threads to run them all.
		{/snippet}
		<FanOutDemo />
	</DemoSection>

	<DemoSection
		id="parallel"
		index="04"
		title="Work spreads across workers"
		sources={data.sources.parallel}
	>
		{#snippet summary()}
			These tasks compute instead of wait, so each one needs a thread. Idle workers steal queued
			tasks from busy ones, which spreads the batch over the whole runtime. A task only gives up
			its worker at an <code>.await</code>, so <code>yield_now</code> adds one between batches.
		{/snippet}
		<ParallelDemo workers={data.workers} />
	</DemoSection>

	<DemoSection id="stream" index="05" title="Rust calls back" sources={data.sources.stream}>
		{#snippet summary()}
			Producer tasks push samples into an <code>mpsc</code> channel. A forwarder hands each one to a
			JavaScript callback through a <code>ThreadsafeFunction</code>, and the endpoint relays them as
			server-sent events. Stopping early cancels the tasks with a <code>CancellationToken</code>.
		{/snippet}
		<StreamDemo />
	</DemoSection>

	<DemoSection id="probe" index="06" title="Network I/O with timeouts" sources={data.sources.probe}>
		{#snippet summary()}
			Looks up each host and opens a TCP connection to it, all at the same time, on tokio's
			non-blocking sockets. Each attempt gets its own timeout. This one needs internet access.
		{/snippet}
		<ProbeDemo />
	</DemoSection>

	<DemoSection id="metrics" index="07" title="Reading the runtime" sources={data.sources.metrics}>
		{#snippet summary()}
			The bar pinned to the top of the page is fed by a synchronous export that reads tokio's own
			metrics four times a second. The event-loop reading comes from Node: a timer that reports how
			late it fired.
		{/snippet}

		<h3>What the monitor shows</h3>
		<dl class="legend">
			<div>
				<dt>Node event loop</dt>
				<dd>
					One bar per reading. The taller the bar, the longer Node was unable to run anything; red
					means over 50 ms. If readings stop arriving, the loop is blocked right now.
				</dd>
			</div>
			<div>
				<dt>tokio workers</dt>
				<dd>
					One meter per worker thread, showing how much of the last 250 ms it spent running tasks.
					A worker in the middle of a long task shows as full, and one that only woke for a moment
					still shows a sliver.
				</dd>
			</div>
			<div>
				<dt>Tasks alive</dt>
				<dd>
					Tasks that have been spawned and have not finished. Each pending call from JavaScript
					counts as two: the future itself, and a small task napi-rs uses to watch it.
				</dd>
			</div>
			<div>
				<dt>Queued</dt>
				<dd>Tasks waiting in the queue shared by all workers.</dd>
			</div>
		</dl>
	</DemoSection>
</main>

<footer class="page">
	<p>
		Built with <a href="https://napi.rs">napi-rs</a>, <a href="https://tokio.rs">tokio</a> and
		<a href="https://svelte.dev/docs/kit">SvelteKit</a>. The code shown beside each demo is read from
		the files that are running.
	</p>
</footer>

<style>
	.page {
		max-width: var(--page);
		margin-inline: auto;
		padding-inline: var(--gutter);
	}

	.hero {
		display: grid;
		gap: 1.25rem;
		padding-block: clamp(2.5rem, 8vw, 5rem) clamp(2rem, 5vw, 3rem);
	}

	.eyebrow {
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--rust);
	}

	h1 {
		max-width: 16ch;
		font-size: clamp(2.1rem, 6vw, 3.6rem);
		line-height: 1.05;
		letter-spacing: -0.03em;
		text-wrap: balance;
	}

	.lede {
		max-width: 64ch;
		font-size: clamp(1rem, 2vw, 1.125rem);
		color: var(--muted);
	}

	.versions,
	nav {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		padding: 0;
		list-style: none;
	}

	.versions li {
		padding: 0.2rem 0.65rem;
		border: 1px solid var(--border);
		border-radius: 999px;
		background: var(--surface);
		font-family: var(--mono);
		font-size: 0.78rem;
		color: var(--muted);
	}

	.versions b {
		font-weight: 600;
		color: var(--text);
	}

	nav {
		gap: 0.4rem 1.25rem;
		font-size: 0.9rem;
	}

	h3 {
		font-size: 0.95rem;
	}

	/* The round trip of a call, as a vertical line of stops. */
	.path {
		display: grid;
		padding: 0;
		list-style: none;
	}

	.path li {
		position: relative;
		padding: 0 0 1.1rem 1.6rem;
		font-size: 0.9rem;
		color: var(--muted);
	}

	.path li:last-child {
		padding-bottom: 0;
	}

	.path li::before {
		content: '';
		position: absolute;
		left: 0;
		top: 0.4rem;
		width: 0.6rem;
		height: 0.6rem;
		border-radius: 50%;
		background: var(--muted);
	}

	.path li:not(:last-child)::after {
		content: '';
		position: absolute;
		left: calc(0.3rem - 0.5px);
		top: 1.2rem;
		bottom: -0.2rem;
		width: 1px;
		background: var(--border);
	}

	.path .node::before {
		background: var(--node);
	}

	.path .rust::before {
		background: var(--rust);
	}

	.path b {
		display: block;
		color: var(--text);
	}

	.legend {
		display: grid;
		gap: 0.9rem;
		font-size: 0.9rem;
	}

	.legend dt {
		font-weight: 600;
	}

	.legend dd {
		color: var(--muted);
	}

	footer {
		padding-block: 2rem 3rem;
		border-top: 1px solid var(--border);
		font-size: 0.875rem;
		color: var(--muted);
	}
</style>
