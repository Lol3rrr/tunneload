<script lang="ts">
	export let rules: Array<Rule> = [];

	import { onMount } from "svelte";

	onMount(async () => {
		const res = await fetch("/api/rules");
		const content = await res.json() as {
			rules: Array<Rule>,
		};

		rules = content.rules
	});
</script>

<content>
	<h1>
		Rules
	</h1>
	<div class="rule_container">
		{#each rules as rule}
			<div class="rule">
				<h3>{rule.name}</h3>
				<h4>Priority</h4>
				<p>{rule.priority}</p>
				<h4>Middlewares</h4>
				<div>
					{#each rule.middlewares as middleware}
						<p>{middleware.name}</p>
					{/each}
				</div>
				<h4>Service</h4>
				<p>{rule.service.name}</p>
				<h4>TLS</h4>
				<p>{rule.tls}</p>
			</div>
		{/each}
	</div>
</content>

<style>
	h1 {
		color: #CCCCCC;
	}

	.rule_container {
		width: 80%;
		margin: 0% 10%;

		display: grid;
	}

	.rule {
		display: inline-block;
		background-color: #cccccc;
	}

	.rule > h4 {
		text-align: left;
	}
	.rule > p {
		font-size: 14px;
	}
</style>
