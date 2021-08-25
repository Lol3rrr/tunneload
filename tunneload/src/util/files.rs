//! This module contains a couple of nice to haves to easier deal with Files
//! in certain Cases

pub mod events {
    //! This provides a custom Watcher to listen for any File related Events

    use std::{path::PathBuf, time::Duration};

    use notify::{DebouncedEvent, Watcher};

    /// The CustomWatcher that you can use to listen for File-Events in a
    /// nice and simple way
    pub struct CustomWatcher {
        _watcher: notify::RecommendedWatcher,
        rx: std::sync::mpsc::Receiver<DebouncedEvent>,
    }

    impl CustomWatcher {
        /// Creates a new Watcher that looks for all the Events in the given
        /// Path and all of its sub-directories
        pub fn new(path: String) -> Option<Self> {
            let (tx, rx) = std::sync::mpsc::channel();

            let mut watcher = match notify::watcher(tx, Duration::from_secs(1)) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!("Creating Middleware-File-Watcher: {:?}", e);
                    return None;
                }
            };

            if let Err(e) = watcher.watch(path, notify::RecursiveMode::Recursive) {
                tracing::error!("Starting Watcher for Path: {:?}", e);
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
                        tracing::info!("Unexpected Event: {:?}", event);
                        self.next()
                    }
                },
                Err(e) => {
                    tracing::error!("Error receiving File-Event: {:?}", e);
                    None
                }
            }
        }
    }
}
