<script lang="ts">
	export let entity: Acceptor = undefined;
</script>

<acceptor>
	<h3>{entity.type}</h3>

	{#if entity.type == "Webserver"}
		<h4>Port: {entity.content.port}</h4>
	{:else if entity.type == "Tunneler"}
		<h4>Port: {entity.content.external_port}</h4>
	{/if}
</acceptor>

<style>
	acceptor {
		display: inline-block;

		margin: auto;
		padding: 5px 10px;
		border-radius: 5px;
		background-color: var(--silver);

		transition: 0.2s;
	}
	acceptor:hover {
		transform: scale(1.05, 1.05);
	}
	h3 {
		margin-top: 5px;
		margin-bottom: 5px;
	}
</style>
