<script lang="ts">
	export let rule: Rule = undefined;

	import Matcher from "./Matcher.svelte";
</script>

<div class="rule">
	<h2>{rule.name}</h2>

	<hr>

	<h3><u>Priority</u>: {rule.priority}</h3>
	
	<h3><u>Matcher</u></h3>
	<Matcher matcher="{rule.matcher}" />

	<h3><u>Middlewares</u></h3>
	<ul>
		{#each rule.middlewares as middleware}
			<li>{middleware.name}</li>
		{/each}
	</ul>

	<h3><u>Service</u>: {rule.service.name}</h3>

	<h3><u>TLS</u>: {rule.tls}</h3>
</div>

<style>
	* {
		color: var(--white);
	}

	.rule {
		position: absolute;
		display: inline-block;

		top: 10vh;
		margin: auto;
		padding: 0px 1vw 2vh 1vw;
		transform: translateX(-50%);

		box-shadow: black 5px 5px 10px;
		border-radius: 6px;
		background-color: #525557;
	}

	ul {
		display: inline-block;
		left: 0;
		text-align: left;
		margin-top: 0px;
	}
</style>
