<script lang="ts">
	import { onMount } from "svelte";
	
	import { load_rules } from "@src/api/rules";
	import { load_services } from "@src/api/services";
	import { load_middlewares } from "@src/api/middlewares";

	export let rules: Array<Rule> = [];
	export let services: Array<Service> = [];
	export let middlewares: Array<Middleware> = [];

	onMount(async () => {
		rules = await load_rules();
		services = await load_services();
		middlewares = await load_middlewares();
	});
</script>

<content>
	<h1>
		Dashboard
	</h1>
	<h2>Acceptors</h2>
	<h2>Routes</h2>
	<div>
		<span>{rules.length} Rules</span>
	</div>
	<h2>Services</h2>
	<div>
		<span>{services.length} Services</span>
	</div>
	<h2>Middlewares</h2>
	<div>
		<span>{middlewares.length} Middlewares</span>
	</div>
</content>

<style>
	h1, h2, span {
		color: var(--white);
	}
</style>
