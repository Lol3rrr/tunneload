use tunneload::{
    configurator::{parser::GeneralConfigurator, ConfigItem, ConfigList},
    rules,
};

pub async fn minimal(g_conf: &GeneralConfigurator) {
    let middlewares = ConfigList::new();
    let services = ConfigList::new();

    // Load the Rules, without a cert_queue to stop it from marking the Rule-TLS as Generate
    let rule_list = g_conf.load_rules(&middlewares, &services, None).await;

    let minimal_rule = rule_list
        .iter()
        .find(|r| r.name() == "testing-rule-minimal")
        .expect("The Rule should have been loaded");
    assert_eq!(
        &rules::Matcher::Domain("example.com".to_owned()),
        minimal_rule.matcher()
    );
    assert_eq!(1, minimal_rule.priority());
    assert_eq!(
        std::sync::Arc::new(rules::Service::new("testing-service-minimal", Vec::new())),
        minimal_rule.service()
    );
    assert_eq!(&rules::RuleTLS::None, minimal_rule.tls());
}
