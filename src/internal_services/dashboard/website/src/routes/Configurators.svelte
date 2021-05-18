<script lang="ts">
	import { onMount } from "svelte";

	import { load_configurators } from "@src/api/configurators";

	onMount(async () => {
		await load_configurators();
	});
</script>

<content>
	<h1>
		Configurators
	</h1>
</content>

<style>
	h1 {
		color: #CCCCCC;
	}
</style>
