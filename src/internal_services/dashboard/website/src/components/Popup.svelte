<script lang="ts">
	export let display: boolean = false;

	function hide_background() {
		display = false;
	}
</script>

<div>
	{#if display}
		<div class="backdrop" on:click="{hide_background}"></div>
		<slot class="content"></slot>
	{/if}
</div>

<style>
	.backdrop {
		position: absolute;
		top: 0px;
		left: 0px;
		width: 100vw;
		height: 100vh;

		background-color: rgba(0, 0, 0);
		opacity: 0.2;
	}
</style>
