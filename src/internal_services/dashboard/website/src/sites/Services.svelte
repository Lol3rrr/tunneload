<script lang="ts">
	import { onMount } from "svelte";

	export let services: Array<Service> = [];

	onMount(async () => {
		const res = await fetch("/api/services");
		const content = await res.json() as {
			services: Array<Service>,
		};

		services = content.services;
	});
</script>

<content>
	<h1>
		Services
	</h1>
</content>

<style>
	h1 {
		color: #CCCCCC;
	}
</style>
