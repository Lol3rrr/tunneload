<script lang="ts">
	import { onMount } from "svelte";

	export let middlewares: Array<Middleware> = [];

	onMount(async () => {
		const res = await fetch("/api/middlewares");
		const content = await res.json() as {
			middlewares: Array<Middleware>,
		};

		middlewares = content.middlewares;
	});
</script>

<content>
	<h1>
		Middlewares
	</h1>
	<div class="middleware_container">
		{#each middlewares as middleware}
			<div class="middleware">
				<h3>{middleware.name}</h3>
			</div>
		{/each}
	</div>
</content>

<style>
	h1 {
		color: #CCCCCC;
	}

	.middleware_container {
		width: 80%;
		margin: 0% 10%;

		display: grid;
	}

	.middleware {
		display: inline-block;
		background-color: #cccccc;
	}
</style>
