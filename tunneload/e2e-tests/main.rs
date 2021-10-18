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
    for case in inventory::iter::<tests::E2ETest> {
        let success = case.run_test();

        if !success {
            panic!("Test {:?} has failed", case.name());
        }
    }
}
