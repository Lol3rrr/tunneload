use crate::{
    cli,
    configurator::{
        self,
        kubernetes::{self, ingress, traefik_bindings},
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
            let g_conf =
                kubernetes::general::setup_general_configurator(client.clone(), k8s_namespace);

            config_builder = config_builder.general_configurator(g_conf);
        }

        if config.traefik {
            kube_dashboard.enable_traefik();

            for traefik_namespace in config.traefik_namespaces.iter() {
                let g_conf = traefik_bindings::setup_general_configurator(
                    client.clone(),
                    traefik_namespace,
                );

                config_builder = config_builder.general_configurator(g_conf);
            }
        }
        if config.ingress {
            kube_dashboard.enable_ingress();

            let priority = config.ingress_priority.unwrap_or(100);

            for ingress_namespace in config.ingress_namespaces.iter() {
                let g_conf = ingress::setup_general_configurator(
                    client.clone(),
                    ingress_namespace,
                    priority,
                );

                config_builder = config_builder.general_configurator(g_conf);
            }
        }

        dashboard_configurators.push(Box::new(kube_dashboard));
    }

    config_builder
}
