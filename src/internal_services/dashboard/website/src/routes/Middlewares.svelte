<script lang="ts">
	import { onMount } from "svelte";

	export let middlewares: Array<Middleware> = [];
	export let middlewares_table_headers = ["Name", "Middleware"];
	export let middlewares_table: Array<Array<String>> = [];

	import { load_middlewares } from "@src/api/middlewares";

	onMount(async () => {
		middlewares = await load_middlewares();
		generate_table_content();
	});

	import CustomTable from "./../components/table.svelte";

	function generate_table_content() {
		let result = [];

		middlewares.forEach((tmp_middleware) => {
			let row = [
				tmp_middleware.name,
				tmp_middleware.action.type,
			];
			result.push(row);
		});

		middlewares_table = result;
	}
</script>

<content>
	<h1>
		Middlewares
	</h1>

	<CustomTable header="{middlewares_table_headers}" content="{middlewares_table}" />
</content>

<style>
	h1 {
		color: var(--white);
	}
</style>
