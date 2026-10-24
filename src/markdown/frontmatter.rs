// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Frontmatter {
    pub title: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub created: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub frontmatter: Option<Frontmatter>,
    pub body: String,
    pub raw_frontmatter: Option<String>,
}

pub fn parse_frontmatter(input: &str) -> ParsedDocument {
    // detect opening ---
    if !input.starts_with("---") {
        return ParsedDocument {
            frontmatter: None,
            body: input.to_string(),
            raw_frontmatter: None,
        };
    }
    // find closing ---
    let _rest = &input[3..];
    // skip first newline
    // Actually handle: content after first --- line
    // Re-evaluate robustly:
    let lines: Vec<&str> = input.lines().collect();
    if lines.is_empty() || lines[0].trim() != "---" {
        return ParsedDocument {
            frontmatter: None,
            body: input.to_string(),
            raw_frontmatter: None,
        };
    }
    let mut end_idx = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" || line.trim() == "..." {
            end_idx = Some(i);
            break;
        }
    }
    let Some(end) = end_idx else {
        return ParsedDocument {
            frontmatter: None,
            body: input.to_string(),
            raw_frontmatter: None,
        };
    };
    let fm_text = lines[1..end].join("\n");
    let body = lines[end + 1..].join("\n");
    // preserve leading newline? body as joined; if original had trailing newline handling, keep simple
    let fm: Option<Frontmatter> = serde_yaml::from_str(&fm_text).ok();
    ParsedDocument {
        frontmatter: fm,
        body,
        raw_frontmatter: Some(fm_text),
    }
}

pub fn serialize_frontmatter(fm: &Frontmatter) -> Result<String> {
    let s = serde_yaml::to_string(fm)?;
    Ok(s)
}

pub fn reconstruct(fm: &Option<Frontmatter>, body: &str) -> String {
    match fm {
        Some(f) => {
            // Use serde_yaml to preserve fields
            match serde_yaml::to_string(f) {
                Ok(s) => format!("---\n{}---\n{}", s, body),
                Err(_) => body.to_string(),
            }
        }
        None => body.to_string(),
    }
}
