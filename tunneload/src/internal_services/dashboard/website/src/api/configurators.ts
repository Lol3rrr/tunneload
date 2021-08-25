export async function load_configurators() {
	const res = await fetch("/api/configurators");
	const content = await res.json() as {
		configurators: Array<Configurator>,
	};

	return content.configurators;
}
