interface Matcher {
	PathPrefix: String,
};

interface Action {
	type: String,
	c: any,
};

interface Middleware {
	name: String,
	action: Action,
};

interface Service {
	name: String,
	addresses: Array<any>,
	internal: Boolean,
};

interface TLS {

};

interface Rule {
	name: String,
	priority: Number,
	matcher: Matcher,
	middlewares: Array<Middleware>,
	service: Service,
	tls: TLS | undefined,
};

interface Configurator {
	type: String,
	content: any,
}
interface Acceptor {
	type: String,
	content: any,
}
