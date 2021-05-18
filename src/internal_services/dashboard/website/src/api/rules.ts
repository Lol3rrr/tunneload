export async function load_rules() {
	const res = await fetch("/api/rules");
	const content = await res.json() as {
		rules: Array<Rule>,
	};

	return content.rules;
}
