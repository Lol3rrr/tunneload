<script lang="ts">
	export let configurators: Array<Configurator> = [];
	export let configurators_table_headers = ["Type"];
	export let configurators_table: Array<Array<String>> = [];

	import { onMount } from "svelte";

	import { load_configurators } from "@src/api/configurators";
	import CustomTable from "@src/components/table.svelte";

	onMount(async () => {
		configurators = await load_configurators();
		generate_table_content();
	});

	function generate_table_content() {
		let result = [];

		configurators.forEach((tmp_configurator) => {
			let row = [
				tmp_configurator.type,
			];
			result.push(row);
		});

		configurators_table = result;
	}
</script>

<content>
	<h1>
		Configurators
	</h1>
	<CustomTable header="{configurators_table_headers}" content="{configurators_table}" />
</content>

<style>
	h1 {
		color: var(--white);
	}
</style>
