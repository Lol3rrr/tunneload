use std::{pin::Pin, time::Duration};

use futures::Future;

#[macro_export]
macro_rules! current_source_dir {
    () => {{
        let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.pop();
        d.push(file!());
        d.pop();
        d
    }};
}

pub use current_source_dir;

pub struct E2ETest {
    name: String,
    setup: Option<Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Sync>>,
    test: Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Sync>,
    teardown: Option<Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Sync>>,
}

inventory::collect!(E2ETest);

impl E2ETest {
    pub fn with_setup_teardown<N, SetupFut, TestFut, TeardownFut>(
        name: N,
        setup: fn() -> SetupFut,
        test: fn() -> TestFut,
        teardown: fn() -> TeardownFut,
    ) -> Self
    where
        N: Into<String>,
        SetupFut: Future<Output = ()> + Send + 'static,
        TestFut: Future<Output = ()> + Send + 'static,
        TeardownFut: Future<Output = ()> + Send + 'static,
    {
        Self {
            name: name.into(),
            setup: Some(Box::new(move || Box::pin(setup()))),
            test: Box::new(move || Box::pin(test())),
            teardown: Some(Box::new(move || Box::pin(teardown()))),
        }
    }

    pub fn only_test<N, TestFut>(name: N, test: fn() -> TestFut) -> Self
    where
        N: Into<String>,
        TestFut: Future<Output = ()> + Send + 'static,
    {
        Self {
            name: name.into(),
            setup: None,
            test: Box::new(move || Box::pin(test())),
            teardown: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn run_test(&self) -> bool {
        println!(" - Running Test - {:?}... ", self.name);

        let setup = match &self.setup {
            Some(call) => call(),
            None => Box::pin(async move {}),
        };
        let test = (self.test)();
        let teardown = match &self.teardown {
            Some(call) => call(),
            None => Box::pin(async move {}),
        };

        let thread_builder = std::thread::Builder::new().name(self.name.clone());
        let handle = thread_builder
            .spawn(|| {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                runtime.block_on(setup);
                std::thread::sleep(Duration::from_millis(2000));
                runtime.block_on(test);
                runtime.block_on(teardown);
            })
            .unwrap();

        match handle.join() {
            Ok(_) => {
                println!("PASSED {:?}", self.name);
                true
            }
            Err(_) => {
                println!("FAILED {:?}", self.name);
                false
            }
        }
    }
}
