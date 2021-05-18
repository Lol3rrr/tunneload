export async function load_configurators() {
	const res = await fetch("/api/configurators");
	const content = await res.json() as {
		acceptors: Array<String>,
	};

	console.log(content);
	return [];
}
