interface Matcher {
		PathPrefix: String,
	};

	interface Middleware {
		name: String
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
