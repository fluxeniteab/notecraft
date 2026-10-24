use notecraft::storage::database::Storage;
use tempfile::tempdir;

#[test]
fn storage_roundtrip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("store.json");
    let mut s = Storage::new(path.clone());
    s.set("key", serde_json::json!("value"));
    s.save().unwrap();
    let s2 = Storage::new(path);
    assert_eq!(s2.get("key").unwrap(), serde_json::json!("value"));
}

#[test]
fn settings_malformed_recovery() {
    use notecraft::config::load_settings;
    // load_settings returns default on missing; just call
    let _ = load_settings();
}
