pub mod tests;

pub fn get_namespace() -> String {
    std::env::var("K8S_TEST_NS").unwrap_or("testing".to_owned())
}

mod kubernetes;

pub fn cmp_vec_contents<T>(a: &[T], b: &[T]) -> bool
where
    T: PartialEq,
{
    a.iter()
        .map(|item| b.contains(item))
        .find(|is_contained| !is_contained)
        .is_none()
}

fn main() {
    let mut args = std::env::args();

    let dry_run = args.find(|arg| arg == "--dry-run").is_some();

    setup_tracing();

    for case in inventory::iter::<tests::E2ETest> {
        if dry_run {
            println!(" - Dry-Run {}", case.name());
            continue;
        }

        let success = case.run_test();

        if !success {
            panic!("Test {:?} has failed", case.name());
        }
    }
}

fn setup_tracing() {
    let tracing_directive_str =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "tunneload=debug".to_owned());
    let tracing_sub = tracing_subscriber::FmtSubscriber::builder()
        .with_level(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing_directive_str.parse().unwrap()),
        )
        .finish();
    tracing::subscriber::set_global_default(tracing_sub)
        .expect("Setting initial Tracing-Subscriber");
}
