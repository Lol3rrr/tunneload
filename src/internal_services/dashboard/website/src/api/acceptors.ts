export async function load_acceptors() {
	const res = await fetch("/api/acceptors");
	const content = await res.json() as {
		acceptors: Array<Acceptor>,
	};

	return content.acceptors;
}
