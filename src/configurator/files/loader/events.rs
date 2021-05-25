use std::{path::PathBuf, time::Duration};

use notify::{DebouncedEvent, Watcher};

pub struct CustomWatcher {
    _watcher: notify::RecommendedWatcher,
    rx: std::sync::mpsc::Receiver<DebouncedEvent>,
}

impl CustomWatcher {
    pub fn new(path: String) -> Option<Self> {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = match notify::watcher(tx, Duration::from_secs(1)) {
            Ok(w) => w,
            Err(e) => {
                log::error!("Creating Middleware-File-Watcher: {:?}", e);
                return None;
            }
        };

        if let Err(e) = watcher.watch(path, notify::RecursiveMode::Recursive) {
            log::error!("Starting Watcher for Path: {:?}", e);
            return None;
        }

        Some(Self {
            _watcher: watcher,
            rx,
        })
    }
}

impl Iterator for CustomWatcher {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        match self.rx.recv() {
            Ok(event) => match event {
                DebouncedEvent::Write(path) | DebouncedEvent::Create(path) => Some(path),
                DebouncedEvent::NoticeWrite(_) | DebouncedEvent::NoticeRemove(_) => self.next(),
                _ => {
                    log::info!("Unexpected Event: {:?}", event);
                    self.next()
                }
            },
            Err(e) => {
                log::error!("Error receiving File-Event: {:?}", e);
                None
            }
        }
    }
}
