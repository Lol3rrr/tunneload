<script lang="ts">
	import { onMount } from "svelte";

	export let plugins: Array<ActionPlugin> = [];
	export let table_headers = ["Name"];
	export let table: Array<Array<String>> = [];

	import { load_plugins } from "@src/api/plugins";

	onMount(async () => {	
		plugins = await load_plugins();
		generate_table_content();
	});

	import CustomTable from "./../components/table.svelte";

	function generate_table_content() {
		let result = [];

		plugins.forEach((tmp_plugin) => {
			let row = [
				tmp_plugin.name,
			];
			result.push(row);
		});

		table = result;
	}
</script>

<content>
	<h1>
		Action-Plugins
	</h1>
	<CustomTable header="{table_headers}" content="{table}" />
</content>

<style>
	h1 {
		color: var(--white);
	}
</style>
