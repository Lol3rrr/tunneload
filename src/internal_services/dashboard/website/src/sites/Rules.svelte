<script lang="ts">
	export let rules: Array<Rule> = [];

	import { onMount } from "svelte";

	onMount(async () => {
		const res = await fetch("/api/rules");
		const content = await res.json() as {
			rules: Array<Rule>,
		};

		rules = content.rules;	
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
				<div>
					<h4>Priority: {rule.priority}</h4>
				</div>
				<div>
					<h4>Matcher: </h4>
					<span>"Some Matcher"</span>
				</div>
				<div>
					<h4>Middlewares: </h4>
					<div>
						{#each rule.middlewares as middleware}
							<p>{middleware.name}</p>
						{/each}
					</div>
				</div>
				<div>
					<h4>Service: </h4>
					<span>{rule.service.name}</span>
				</div>
				<div>
					<h4>TLS: </h4>
					<span>{rule.tls}</span>
				</div>
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

		display: flex;
		flex-direction: row;
	}

	.rule {
		width: 25%;
		display: inline-block;
		background-color: #cccccc;

		margin: 5px;
		padding: 15px 8px;
		border-radius: 8px;
	}

	h3 {
		margin-top: 5px;
	}
	h4 {
		text-align: left;
		margin-top: 5px;
		margin-bottom: 0px;
	}
	p {
		font-size: 14px;
	}
</style>
