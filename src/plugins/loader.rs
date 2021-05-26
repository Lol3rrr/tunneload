use super::ActionPlugin;

mod actions;

/// A Configurator, which is responsible for loading all the
/// Plugins into Tunneload
pub struct Loader {
    path: String,
}

impl Loader {
    /// Creates a new Plugin-Loader with the given Path-String
    /// as the source of Plugins to load
    pub fn new(path: String) -> Self {
        Self { path }
    }

    /// Attempts to load all the Action-Plugins from the configured Path
    pub fn load_action_plugins(&self) -> Vec<ActionPlugin> {
        actions::load_actions(&self.path)
    }
}
