<script lang="ts">
	import { onMount } from "svelte";

	export let action_plugins: Array<ActionPlugin> = [];
	export let action_table_headers = ["Name"];
	export let action_table: Array<Array<String>> = [];

	import { load_action_plugins } from "@src/api/action_plugins";

	onMount(async () => {	
		action_plugins = await load_action_plugins();
		generate_table_content();
	});

	import CustomTable from "./../components/table.svelte";

	function generate_table_content() {
		let result = [];

		action_plugins.forEach((tmp_plugin) => {
			let row = [
				tmp_plugin.name,
			];
			result.push(row);
		});

		action_table = result;
	}
</script>

<content>
	<h1>
		Action-Plugins
	</h1>
	<CustomTable header="{action_table_headers}" content="{action_table}" />
</content>

<style>
	h1 {
		color: var(--white);
	}
</style>
