use crate::{cli, configurator, internal_services::DashboardEntityList};

/// Handles all the Setup for the Files-Configurator
pub fn setup(
    config: &cli::Options,
    mut config_builder: configurator::ManagerBuilder,
    dashboard_configurators: &mut DashboardEntityList,
) -> configurator::ManagerBuilder {
    if let Some(path) = config.file.clone() {
        log::info!("Enabling File-Configurator");

        let (file_manager, file_configurator) = configurator::files::new(path);
        config_builder = config_builder.configurator(file_manager);

        dashboard_configurators.push(Box::new(file_configurator));
    }

    config_builder
}
