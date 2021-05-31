export async function load_action_plugins() {
	const res = await fetch("/api/plugins/actions");
	const content = await res.json() as {
		plugins: Array<ActionPlugin>,
	};
	
	return content.plugins;
}
