use async_trait::async_trait;

use crate::configurator::parser::{Loader, RawMiddlewareConfig, RawRuleConfig};

mod middlewares;
mod rules;

pub struct FileLoader {
    path: String,
}

impl FileLoader {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    fn load<T, F>(path: String, parse: &F) -> Vec<T>
    where
        F: Fn(&str) -> Option<Vec<T>>,
    {
        let mut result = Vec::new();

        let metadata = match std::fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                log::error!("Loading Metadata: {:?}", e);
                return Vec::new();
            }
        };
        if metadata.is_file() {
            match parse(&path) {
                Some(conf) => result.extend(conf),
                None => {
                    log::error!("Could not Load-File: {:?}", path);
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
        Self::load(self.path.clone(), &|path: &str| {
            middlewares::load_file(path)
        })
    }

    async fn rules(&self) -> Vec<RawRuleConfig> {
        Self::load(self.path.clone(), &|path: &str| rules::load_file(path))
    }
}
