<script lang="ts">
	export let middleware: Middleware = undefined;

	import Action from "./Action.svelte";
</script>

<div class="middleware">
	<h2>{middleware.name}</h2>

	<hr>

	<Action action="{middleware.action}" />
</div>

<style>
	h2 {
		color: var(--white);
	}
	.middleware {
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
</style>
