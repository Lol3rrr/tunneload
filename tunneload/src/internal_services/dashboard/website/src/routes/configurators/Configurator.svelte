<script lang="ts">
	export let entity: Configurator = undefined;
</script>

<configurator>
	<h3>{entity.type}</h3>

	{#if entity.type == "Kubernetes"}
		<h4>Traefik: {entity.content.traefik}</h4>
		<h4>Ingress: {entity.content.ingress}</h4>
	{:else if entity.type == "File"}
		<h4>Path: "{entity.content.path}"</h4>
	{/if}
</configurator>

<style>
	configurator {
		display: inline-block;

		padding: 5px 10px;
		border-radius: 5px;
		background-color: var(--silver);

		transition: 0.2s;
	}
	configurator:hover {
		transform: scale(1.05, 1.05);
	}
	h3 {
		margin-top: 5px;
		margin-bottom: 5px;
	}
</style>
