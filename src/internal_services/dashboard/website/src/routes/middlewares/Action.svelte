<script lang="ts">
	export let action: Action = undefined;
</script>

<div class="action">
	<h3>Type: {action.type}</h3>

	<div class="attributes">
		{#if action.type == "RemovePrefix"}
			<h3>Prefix: "{action.c}"</h3>
		{:else if action.type == "Cors"}
			<div>
				<h3>Origins:</h3>
				<ul>
					{#each action.c.origins as origin}
						<li>{origin}</li>
					{/each}
				</ul>
			</div>
			<div>
				<h3>Headers</h3>
				<ul>
					{#each action.c.headers as header}
						<li>{header}</li>
					{/each}
				</ul>
			</div>
			<div>
				<h3>Methods</h3>
				<ul>
					{#each action.c.methods as method}
						<li>{method}</li>
					{/each}
				</ul>
			</div>
			<div>
				<h3>Credentials: {action.c.credentials}</h3>
			</div>
			<div>
				<h3>Max-Age: {action.c.max_age}</h3>
			</div>
		{/if}
	</div>
</div>

<style>
	* {
		color: var(--white);
	}

	h3 {
		text-align: center;
		margin-bottom: 0.5vh;
	}

	.action {
		display: inline-block;
		margin: auto;
	}

	.attributes {
		display: inline-block;

		text-align: left;
	}

	ul {
		display: inline-block;
		left: 0;
		text-align: left;
	}
</style>
