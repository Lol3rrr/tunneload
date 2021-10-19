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
