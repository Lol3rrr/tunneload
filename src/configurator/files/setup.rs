use crate::{
    cli,
    configurator::{
        self,
        files::{FileEvents, FileLoader, FileParser},
        parser::GeneralConfigurator,
    },
    internal_services::DashboardEntityList,
};

/// Handles all the Setup for the Files-Configurator
pub fn setup(
    config: &cli::Options,
    mut config_builder: configurator::ManagerBuilder,
    dashboard_configurators: &mut DashboardEntityList,
) -> configurator::ManagerBuilder {
    if let Some(path) = config.file.clone() {
        log::info!("Enabling File-Configurator");

        let (file_manager, file_configurator) = configurator::files::new(path.clone());
        config_builder = config_builder.configurator(file_manager);

        let file_loader = FileLoader::new(path.clone());
        let file_events = FileEvents::new(path);
        let file_parser = FileParser::new();
        config_builder = config_builder.general_configurator(GeneralConfigurator::new(
            file_loader,
            file_events,
            file_parser,
        ));

        dashboard_configurators.push(Box::new(file_configurator));
    }

    config_builder
}
