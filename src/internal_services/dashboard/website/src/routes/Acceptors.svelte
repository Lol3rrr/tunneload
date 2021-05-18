<script lang="ts">
	import { onMount } from "svelte";

	import { load_acceptors } from "@src/api/acceptors";

	export let acceptors: Array<String> = [];
	export let acceptors_table_headers = ["Name"];
	export let acceptors_table: Array<Array<String>> = [];

	import CustomTable from "@src/components/table.svelte";

	onMount(async () => {
		acceptors = await load_acceptors();
		generate_table_content();
	});

	function generate_table_content() {
		let result = [];

		acceptors.forEach((tmp_acceptor) => {
			let row = [
				tmp_acceptor,
			];
			result.push(row);
		});

		acceptors_table = result;
	}
</script>

<content>
	<h1>
		Acceptors
	</h1>
	<CustomTable header="{acceptors_table_headers}" content="{acceptors_table}" />
</content>

<style>
	h1 {
		color: #CCCCCC;
	}
</style>
