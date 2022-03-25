use crate::internal_services::DashboardEntity;

use serde_json::json;

/// The Dashboard-Entity for the File-Configurator
pub struct FileConfigurator {
    path: String,
}

impl FileConfigurator {
    /// Creates a new Empty Entity
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl DashboardEntity for FileConfigurator {
    fn get_type(&self) -> &str {
        "File"
    }
    // This is needed here because of the Macro
    #[allow(clippy::disallowed_methods)]
    fn get_content(&self) -> serde_json::Value {
        json!({
            "path": self.path,
        })
    }
}
