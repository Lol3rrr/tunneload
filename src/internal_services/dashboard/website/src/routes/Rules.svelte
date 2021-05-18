<script lang="ts">
	export let rules: Array<Rule> = [];
	export let rules_table_headers = ["Name", "Priority", "Service", "TLS"];
	export let rules_table: Array<Array<String>> = [];

	import { onMount } from "svelte";

	import { load_rules } from "@src/api/rules";

	onMount(async () => {
		rules = await load_rules();
		generate_table_content();
	});

	import CustomTable from "./../components/table.svelte";

	function generate_table_content() {
		let result = [];

		rules.forEach((tmp_rule) => {
			let tls = tmp_rule.tls != undefined ? "Enabled" : "Disabled";
			let row = [
				tmp_rule.name,
				tmp_rule.priority.toString(),
				tmp_rule.service.name,
				tls,
			];
			result.push(row);
		});

		rules_table = result;
	}

	export function handle_click(index: number) {
		const rule = rules[index];
		return () => {
			console.log(rule);
		};
	}
</script>

<content>
	<h1>
		Rules
	</h1>
	<CustomTable header="{rules_table_headers}" content="{rules_table}" row_click="{handle_click}" />
</content>

<style>
	h1 {
		color: var(--white);
	}
</style>
