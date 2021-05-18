<script lang="ts">
	import Navbar from "./Navbar.svelte";

	import { Router, Route } from "svelte-routing";

	import Dashboard from "./routes/Dashboard.svelte";
	import Rules from "./routes/Rules.svelte";
	import Services from "./routes/Services.svelte";
	import Middlewares from "./routes/Middlewares.svelte";
	import Acceptors from "./routes/Acceptors.svelte";

	export let url = "";
</script>

<main>
	<Router url="{url}">
		<Navbar />

		<Route path="/">
			<Dashboard />
		</Route>
		<Route path="/rules">
			<Rules />
		</Route>
		<Route path="/services">
			<Services />
		</Route>
		<Route path="/middlewares">
			<Middlewares />
		</Route>
		<Route path="/acceptors">
			<Acceptors />
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
