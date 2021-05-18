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
	<div class="service_container">
		{#each services as service}
			<div class="service">
				<h3>{service.name}</h3>
			</div>
		{/each}
	</div>
</content>

<style>
	h1 {
		color: #CCCCCC;
	}

	.service_container {
		width: 80%;
		margin: 0% 10%;

		display: flex;
	}

	.service {
		margin: 5px;
		padding: 2px 6px;
		border-radius: 8px;
		background-color: #cccccc;
	}
</style>
