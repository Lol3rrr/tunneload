<script lang="ts">
	import Navbar from "./Navbar.svelte";

	import { Router, Route } from "svelte-routing";

	import Dashboard from "./Dashboard.svelte";
	import Routes from "./Routes.svelte";
	import Services from "./Services.svelte";
	import Middlewares from "./Middlewares.svelte";

	export let url = "";
</script>

<main>
	<Router url="{url}">
		<Navbar />

		<Route path="/">
			<Dashboard />
		</Route>
		<Route path="/routes">
			<Routes />
		</Route>
		<Route path="/services">
			<Services />
		</Route>
		<Route path="/middlewares">
			<Middlewares />
		</Route>
	</Router>
</main>

<style>
	main {
		text-align: center;
		margin: 0 auto;
		width: 100%;
		height: 100%;
	}
</style>
