use notecraft::search::index::SearchIndex;
use tempfile::tempdir;

#[test]
fn search_index() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("a.md"), "hello world").unwrap();
    std::fs::write(root.join("b.md"), "nothing here").unwrap();
    let mut idx = SearchIndex::new();
    idx.rebuild(root);
    let res = idx.search("hello");
    assert_eq!(res.len(), 1);
    assert!(res[0].snippet.contains("hello"));
}

#[test]
fn search_unicode() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("u.md"), "Héllo 世界").unwrap();
    let mut idx = SearchIndex::new();
    idx.rebuild(root);
    let res = idx.search("世界");
    assert_eq!(res.len(), 1);
}
