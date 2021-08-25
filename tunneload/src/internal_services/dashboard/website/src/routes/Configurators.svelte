<script lang="ts">
	export let configurators: Array<Configurator> = [];

	import { onMount } from "svelte";

	import { load_configurators } from "@src/api/configurators";

	onMount(async () => {
		configurators = await load_configurators();
	});

	import Configurator from "@src/routes/configurators/Configurator.svelte";
</script>

<content>
	<h1>
		Configurators
	</h1>

	<div class="configurator_list">
		{#each configurators as configurator}
			<Configurator entity="{configurator}" />
		{/each}
	</div>
</content>

<style>
	h1 {
		color: var(--white);
	}

	.configurator_list {
		display: grid;
		grid-auto-flow: row;
		grid-template-columns: 1fr 1fr 1fr 1fr;
		justify-items: center;
		max-width: 90%;
		margin: auto;
	}
</style>
