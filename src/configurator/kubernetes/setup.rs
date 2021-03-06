use crate::{
    cli,
    configurator::{
        self,
        kubernetes::{
            general::{KubernetesEvents, KubernetesLoader, KubernetesParser},
            ingress::{IngressEvents, IngressLoader, IngressParser},
            traefik_bindings::{TraefikEvents, TraefikLoader, TraefikParser},
        },
        parser::GeneralConfigurator,
    },
    internal_services::DashboardEntityList,
};

/// Handles all the Setup related to the Kubernetes-Configurator
#[tracing::instrument(skip(rt, config, config_builder, dashboard_configurators))]
pub fn setup(
    rt: &tokio::runtime::Runtime,
    config: &cli::KubernetesOpts,
    mut config_builder: configurator::ManagerBuilder,
    dashboard_configurators: &mut DashboardEntityList,
) -> configurator::ManagerBuilder {
    if config.is_enabled() {
        let mut kube_dashboard = configurator::kubernetes::KubernetesConfigurator::new();

        tracing::info!("Enabling Kubernetes-Configurator");
        let client = rt.block_on(kube::Client::try_default()).unwrap();

        let k8s_loader = KubernetesLoader::new(client.clone(), "default".to_owned());
        let k8s_events = KubernetesEvents::new(client.clone(), "default".to_owned());
        let k8s_parser = KubernetesParser::new();
        config_builder = config_builder
            .general_configurator(GeneralConfigurator::new(k8s_loader, k8s_events, k8s_parser));

        if config.traefik {
            tracing::info!("Enabling Traefik-Kubernetes-Configurator");
            kube_dashboard.enable_traefik();

            let traefik_loader = TraefikLoader::new(client.clone(), "default".to_owned());
            let traefik_events = TraefikEvents::new(client.clone(), "default".to_owned());
            let traefik_parser =
                TraefikParser::new(Some(client.clone()), Some("default".to_owned()));

            config_builder = config_builder.general_configurator(GeneralConfigurator::new(
                traefik_loader,
                traefik_events,
                traefik_parser,
            ));
        }
        if config.ingress {
            tracing::info!("Enabling Ingress-Kubernetes-Configurator");
            kube_dashboard.enable_ingress();

            let priority = config.ingress_priority.unwrap_or(100);

            let ingress_loader = IngressLoader::new(client.clone(), "default".to_owned());
            let ingress_events = IngressEvents::new(client, "default".to_owned());
            let ingress_parser = IngressParser::new(priority);
            config_builder = config_builder.general_configurator(GeneralConfigurator::new(
                ingress_loader,
                ingress_events,
                ingress_parser,
            ));
        }

        dashboard_configurators.push(Box::new(kube_dashboard));
    }

    config_builder
}
