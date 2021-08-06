pub fn get_namespace() -> String {
    std::env::var("K8S_TEST_NS").unwrap_or("testing".to_owned())
}

mod kubernetes;

#[cfg(feature = "e2e")]
mod e2e {
    use super::*;

    #[test]
    fn kubernetes() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(kubernetes::run());
    }
}
