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

	export let selected_middleware: Action | undefined = undefined;

	export function handle_click(index: number) {
		const middleware = middlewares[index];
		return () => {
			selected_middleware = middleware;
		};
	}

	import Popup from "@src/components/Popup.svelte";
	import Middleware from "@src/routes/middlewares/Middleware.svelte";
</script>

<content>
	<h1>
		Middlewares
	</h1>

	<CustomTable header="{middlewares_table_headers}" content="{middlewares_table}" row_click="{handle_click}" />

	<Popup display="{selected_middleware != undefined}">
		<Middleware middleware="{selected_middleware}" />
	</Popup>
</content>

<style>
	h1 {
		color: var(--white);
	}
</style>
