use crate::{
    cli,
    configurator::{
        self,
        kubernetes::{
            general::{KubernetesEvents, KubernetesLoader, KubernetesParser},
            ingress::{IngressEvents, IngressLoader, IngressParser},
            traefik_bindings::{self, TraefikEvents, TraefikLoader, TraefikParser},
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

        let client = rt.block_on(kube::Client::try_default()).unwrap();

        for k8s_namespace in config.namespaces.iter() {
            tracing::info!(
                "Enabling Kubernetes-Configurator for Namespace: {}",
                k8s_namespace
            );

            let k8s_loader = KubernetesLoader::new(client.clone(), k8s_namespace.to_owned());
            let k8s_events = KubernetesEvents::new(client.clone(), k8s_namespace.to_owned());
            let k8s_parser = KubernetesParser::new();

            config_builder = config_builder.general_configurator(GeneralConfigurator::new(
                format!("K8S-{}", k8s_namespace),
                k8s_loader,
                k8s_events,
                k8s_parser,
            ));
        }

        if config.traefik {
            kube_dashboard.enable_traefik();

            for traefik_namespace in config.traefik_namespaces.iter() {
                let g_conf = traefik_bindings::setup_general_configurator(
                    client.clone(),
                    &traefik_namespace,
                );

                config_builder = config_builder.general_configurator(g_conf);
            }
        }
        if config.ingress {
            kube_dashboard.enable_ingress();

            let priority = config.ingress_priority.unwrap_or(100);

            for ingress_namespace in config.ingress_namespaces.iter() {
                tracing::info!(
                    "Enabling Ingress-Kubernetes-Configurator for namespace: {}",
                    ingress_namespace
                );

                let ingress_loader =
                    IngressLoader::new(client.clone(), ingress_namespace.to_owned());
                let ingress_events =
                    IngressEvents::new(client.clone(), ingress_namespace.to_owned());
                let ingress_parser = IngressParser::new(priority);

                config_builder = config_builder.general_configurator(GeneralConfigurator::new(
                    format!("Ingress-{}", ingress_namespace),
                    ingress_loader,
                    ingress_events,
                    ingress_parser,
                ));
            }
        }

        dashboard_configurators.push(Box::new(kube_dashboard));
    }

    config_builder
}
