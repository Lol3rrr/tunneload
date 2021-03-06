use criterion::{black_box, criterion_group, criterion_main, Criterion};
use stream_httparse::{Headers, Request};
use tunneload::{
    plugins::{ActionPluginInstance, Plugin},
    rules::Action,
};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Create Request", |b| {
        b.iter(|| {
            let request = Request::new(
                "HTTP/1.1",
                stream_httparse::Method::GET,
                "/test/api",
                Headers::new(),
                &[],
            );
            black_box(request);
        })
    });

    c.bench_function("Action - StripPrefix", |b| {
        let action = Action::RemovePrefix("/test".to_string());

        b.iter(|| {
            let mut request = Request::new(
                "HTTP/1.1",
                stream_httparse::Method::GET,
                "/test/api",
                Headers::new(),
                &[],
            );

            action.apply_req(&mut request).unwrap();
        })
    });

    c.bench_function("Plugin - StripPrefix", |b| {
        let data = std::fs::read("./tests/plugins/strip_prefix.wasm").unwrap();

        let plugin = Plugin::new("test_name".to_owned(), &data).unwrap();
        let instance: ActionPluginInstance = plugin.create_instance("/test".to_owned()).unwrap();

        b.iter(|| {
            let mut request = Request::new(
                "HTTP/1.1",
                stream_httparse::Method::GET,
                "/test/api",
                Headers::new(),
                &[],
            );

            instance.apply_req(&mut request).unwrap();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
