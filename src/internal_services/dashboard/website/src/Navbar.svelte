<script>
	import { Link } from "svelte-routing";
</script>

<div class="navbar-wrapper">
	<div class="navbar-item">
		<a href="https://github.com/Lol3rrr/tunneload" target="_blank">Tunneload</a>
	</div>
	<div class="navbar-item">
		<Link to="/">Dashboard</Link>
	</div>
	<div class="navbar-item">
		<Link to="/acceptors" class="link">Acceptors</Link>
	</div>
	<div class="navbar-item">
		<Link to="/configurators" class="link">Configurators</Link>
	</div>
	<div class="navbar-item">
		<Link to="/rules" class="link">Rules</Link>
	</div>
	<div class="navbar-item">
		<Link to="/services" class="link">Services</Link>
	</div>
	<div class="navbar-item">
		<Link to="/middlewares" class="link">Middlewares</Link>
	</div>
</div>

<style>
	.navbar-wrapper {
		width: 100%;
		max-height: 5vh;
		display: inline-flex;

		background-color: #4E6E7A;
	}
	.navbar-item {
		max-height: 5vh;
		padding: 0.2vh 0.5vw;

		transition: transform 0.2s;
	}
	.navbar-item:hover {
		transform: scale(1.1, 1.1);
	}
	:global(a) {
		display: inline-block;
		font-size: 22px;

		color: var(--white);
		background-color: inherit;
		border: none;
		text-decoration: none;
	}
	:global(a:hover) {
		text-decoration: none;
	}
</style>
