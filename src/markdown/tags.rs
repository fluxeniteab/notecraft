// Copyright (C) 2026 Fluxenite. Licensed under GPL-3.0-only.
use regex::Regex;
use std::collections::BTreeSet;

pub fn extract_inline_tags(text: &str) -> Vec<String> {
    // #tag (not in code fences ideally, but simple)
    let re = Regex::new(r"(?m)(?:^|[^&\w])#([A-Za-z0-9_\-/]+)").unwrap();
    let mut set = BTreeSet::new();
    for cap in re.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            set.insert(m.as_str().to_string());
        }
    }
    set.into_iter().collect()
}

pub fn merge_tags(fm_tags: &[String], inline: &[String]) -> Vec<String> {
    let mut set = BTreeSet::new();
    for t in fm_tags {
        set.insert(t.clone());
    }
    for t in inline {
        set.insert(t.clone());
    }
    set.into_iter().collect()
}
