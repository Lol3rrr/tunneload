<script lang="ts">
	import { onMount } from "svelte";

	import { load_acceptors } from "@src/api/acceptors";

	export let acceptors: Array<Acceptor> = [];


	onMount(async () => {
		acceptors = await load_acceptors();
	});

	import Acceptor from "@src/routes/acceptors/Acceptor.svelte";
</script>

<content>
	<h1>
		Acceptors
	</h1>

	<div class="acceptor_list">
		{#each acceptors as acceptor}
			<Acceptor entity="{acceptor}" />
		{/each}
	</div>
</content>

<style>
	h1 {
		color: var(--white);
	}

	.acceptor_list {
		display: grid;
		grid-auto-flow: row;
		grid-template-columns: 1fr 1fr 1fr 1fr;
		justify-items: center;
		max-width: 90%;
		margin: auto;
	}
</style>
