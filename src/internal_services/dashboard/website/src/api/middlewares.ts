export async function load_middlewares() {
	const res = await fetch("/api/middlewares");
	const content = await res.json() as {
		middlewares: Array<Middleware>,
	};
	
	return content.middlewares;
}
