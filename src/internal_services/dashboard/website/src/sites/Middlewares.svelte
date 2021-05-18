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
</content>

<style>
	h1 {
		color: #CCCCCC;
	}
</style>
