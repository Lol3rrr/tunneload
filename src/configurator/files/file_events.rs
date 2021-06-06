use async_trait::async_trait;

use crate::configurator::parser::EventEmitter;

pub struct FileEvents {
    path: String,
}

impl FileEvents {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

#[async_trait]
impl EventEmitter for FileEvents {}
