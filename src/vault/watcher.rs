// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;

pub struct VaultWatcher {
    _watcher: RecommendedWatcher,
    pub rx: mpsc::Receiver<notify::Result<notify::Event>>,
}

impl VaultWatcher {
    pub fn new(root: &Path) -> crate::error::Result<Self> {
        let (tx, rx) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        )
        .map_err(|e| crate::error::NoteCraftError::Other(e.to_string()))?;
        watcher
            .watch(root, RecursiveMode::Recursive)
            .map_err(|e| crate::error::NoteCraftError::Other(e.to_string()))?;
        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }
}
