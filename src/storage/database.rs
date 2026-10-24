// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Storage {
    pub data: BTreeMap<String, serde_json::Value>,
    pub path: Option<PathBuf>,
}

impl Storage {
    pub fn new(path: PathBuf) -> Self {
        let data = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            BTreeMap::new()
        };
        Self {
            data,
            path: Some(path),
        }
    }

    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.data.get(key).cloned()
    }

    pub fn set(&mut self, key: &str, value: serde_json::Value) {
        self.data.insert(key.to_string(), value);
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = &self.path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(&self.data)?;
            fs::write(path, json)?;
        }
        Ok(())
    }
}
