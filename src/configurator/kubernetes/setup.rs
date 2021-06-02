use crate::{cli, configurator, internal_services::DashboardEntityList};

/// Handles all the Setup related to the Kubernetes-Configurator
pub fn setup(
    rt: &tokio::runtime::Runtime,
    config: &cli::KubernetesOpts,
    mut config_builder: configurator::ManagerBuilder,
    dashboard_configurators: &mut DashboardEntityList,
) -> configurator::ManagerBuilder {
    if config.is_enabled() {
        let mut kube_dashboard = configurator::kubernetes::KubernetesConfigurator::new();

        log::info!("Enabling Kubernetes-Configurator");
        let mut k8s_manager =
            rt.block_on(configurator::kubernetes::Loader::new("default".to_owned()));

        if config.traefik {
            log::info!("Enabling Traefik-Kubernetes-Configurator");
            k8s_manager.enable_traefik();
            kube_dashboard.enable_traefik();
        }
        if config.ingress {
            log::info!("Enabling Ingress-Kubernetes-Configurator");
            k8s_manager.enable_ingress();
            kube_dashboard.enable_ingress();

            // Checks if a new Priority has been set and if that is
            // the case, overwrites the old default one
            if let Some(n_priority) = config.ingress_priority {
                k8s_manager.set_ingress_priority(n_priority);
            }
        }
        config_builder = config_builder.configurator(k8s_manager);

        dashboard_configurators.push(Box::new(kube_dashboard));
    }

    config_builder
}
