use general_traits::ConfigItem;
use tunneload::configurator::parser::GeneralConfigurator;

pub async fn load(conf: &GeneralConfigurator) {
    let result = conf.load_services().await;

    let is_contained = result
        .iter()
        .find(|s| s.name() == "test-service@testing")
        .is_some();

    if !is_contained {
        panic!(
            "Expected: {:?} to contain a service with the Name: {:?}",
            result, "test-service@testing"
        );
    }

    assert!(true);
}
