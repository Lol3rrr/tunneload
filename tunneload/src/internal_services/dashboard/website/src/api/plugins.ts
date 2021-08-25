export async function load_plugins() {
	const res = await fetch("/api/plugins");
	const content = await res.json() as {
		plugins: Array<ActionPlugin>,
	};
	
	return content.plugins;
}
