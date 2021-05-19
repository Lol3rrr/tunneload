<script lang="ts">
	export let action: Action = undefined;
</script>

<div class="action">
	<h3>Type: {action.type}</h3>

	<div class="attributes">
		{#if action.type == "RemovePrefix"}
			<h3><u>Prefix</u>: "{action.c}"</h3>
		{:else if action.type == "Cors"}
			<div>
				<h3><u>Origins</u>:</h3>
				<ul>
					{#each action.c.origins as origin}
						<li>{origin}</li>
					{/each}
				</ul>
			</div>
			<div>
				<h3><u>Headers</u></h3>
				<ul>
					{#each action.c.headers as header}
						<li>{header}</li>
					{/each}
				</ul>
			</div>
			<div>
				<h3><u>Methods</u></h3>
				<ul>
					{#each action.c.methods as method}
						<li>{method}</li>
					{/each}
				</ul>
			</div>
			<div>
				<h3><u>Credentials</u>: {action.c.credentials}</h3>
			</div>
			<div>
				<h3><u>Max-Age</u>: {action.c.max_age}</h3>
			</div>
		{:else if action.type == "BasicAuth"}
			<h3><u>Users</u>:</h3>
			<ul>
				{#each Object.entries(action.c) as [user, user_pwd]}
					<li>"{user}"</li>
				{/each}
			</ul>
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
