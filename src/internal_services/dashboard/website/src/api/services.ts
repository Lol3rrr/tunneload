export async function load_services() {
	const res = await fetch("/api/services");
	const content = await res.json() as {
		services: Array<Service>,
	};

	return content.services;
}
