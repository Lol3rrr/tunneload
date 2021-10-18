use std::thread::Thread;

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

mod util_tests {
    use super::*;

    #[test]
    fn cmp_vecs_equal() {
        let a = vec![0, 1, 2, 3, 4];
        let b = vec![4, 3, 2, 1, 0];

        assert!(cmp_vec_contents(&a, &b));
    }
    #[test]
    fn cmp_vecs_not_equal() {
        let a = vec![0, 1, 2, 3];
        let b = vec![4, 3, 2, 1];

        assert!(!cmp_vec_contents(&a, &b));
    }
}
