<script lang="ts">
	export let matcher: Matcher = undefined;
</script>

<div class="matcher">
	<h3>{matcher.type}</h3>

	<div class="sub_comp">
		{#if matcher.type == "And" || matcher.type == "Or"}
			{#each matcher.c as sub_matcher}
				<svelte:self matcher="{sub_matcher}" />
			{/each}
		{:else}
			<span>{matcher.c}</span>
		{/if}
	</div>
</div>

<style>
	h3::before {
		--block_size: 14px;
		--size: 5px;

		display: inline-block;
		width: var(--size);
		height: var(--size);
		border-radius: var(--size);
		margin-bottom: calc((var(--block_size) - var(--size)) / 2);
		margin-right: 5px;

		content: " ";
		background-color: var(--white);
	}
	h3, span {
		text-align: left;
		margin-top: 5px;
		margin-bottom: 5px;
	}

	.matcher {
		display: inline-flex;
		flex-direction: column;
	}

	.sub_comp {
		display: inline-flex;
		flex-direction: column;
		padding-left: 20px;
	}
</style>
