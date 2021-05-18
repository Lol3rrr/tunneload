<script lang="ts">
	import { onMount } from "svelte";

	onMount(async () => {
		const res = await fetch("/api/acceptors");
		const content = await res.json();

		console.log(content);
	});
</script>

<content>
	<h1>
		Acceptors
	</h1>
</content>

<style>
	h1 {
		color: #CCCCCC;
	}
</style>
