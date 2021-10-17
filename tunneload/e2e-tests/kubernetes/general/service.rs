use general_traits::ConfigItem;
use tunneload::configurator::parser::GeneralConfigurator;

pub async fn load(conf: &GeneralConfigurator) {
    let result = conf.load_services().await;

    let is_contained = result
        .iter()
        .find(|s| s.name() == "test-service@testing")
        .is_some();

    assert_eq!(true, is_contained);
}
