<script lang="ts">
	import { onMount } from "svelte";

	export let services: Array<Service> = [];
	export let services_table_headers = ["Name"];
	export let services_table: Array<Array<String>> = [];

	import { load_services } from "@src/api/services";

	onMount(async () => {	
		services = await load_services();
		generate_table_content();
	});

	import CustomTable from "./../components/table.svelte";

	function generate_table_content() {
		let result = [];

		services.forEach((tmp_service) => {
			let row = [
				tmp_service.name,
			];
			result.push(row);
		});

		services_table = result;
	}
</script>

<content>
	<h1>
		Services
	</h1>
	<CustomTable header="{services_table_headers}" content="{services_table}" />
</content>

<style>
	h1 {
		color: var(--white);
	}
</style>
