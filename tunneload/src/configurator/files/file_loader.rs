use async_trait::async_trait;

use crate::configurator::parser::{Loader, RawMiddlewareConfig, RawRuleConfig};

mod middlewares;
mod rules;

/// The Loader for the File-Configuration
pub struct FileLoader {
    path: String,
}

impl FileLoader {
    /// Creates a new Instance of the Loader that loads the Configuration
    /// from the provided Path
    pub fn new(path: String) -> Self {
        Self { path }
    }

    fn load<T, F>(path: String, parse: &F) -> Vec<T>
    where
        F: Fn(Vec<u8>) -> Option<Vec<T>>,
    {
        let mut result = Vec::new();

        let metadata = match std::fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Loading Metadata: {:?}", e);
                return Vec::new();
            }
        };
        if metadata.is_file() {
            let content = match std::fs::read(&path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Reading File: {:?}", e);
                    return result;
                }
            };

            match parse(content) {
                Some(conf) => result.extend(conf),
                None => {
                    tracing::error!("Could not Load-File: {:?}", path);
                }
            };
        } else {
            for entry in std::fs::read_dir(&path).unwrap() {
                let raw_path = entry.unwrap().path();
                let entry_path = raw_path.to_str().unwrap();
                let tmp = Self::load(entry_path.to_owned(), parse);
                result.extend(tmp);
            }
        }

        result
    }
}

#[async_trait]
impl Loader for FileLoader {
    async fn middlewares(&self) -> Vec<RawMiddlewareConfig> {
        Self::load(self.path.clone(), &|content: Vec<u8>| {
            middlewares::load_file(content)
        })
    }

    async fn rules(&self) -> Vec<RawRuleConfig> {
        Self::load(self.path.clone(), &|content: Vec<u8>| {
            rules::load_file(content)
        })
    }
}
