use notecraft::vault::operations::*;
use notecraft::vault::tree::{build_tree, is_markdown};
use tempfile::tempdir;

#[test]
fn vault_open_and_tree() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("a.md"), "# hi").unwrap();
    std::fs::create_dir(root.join("sub")).unwrap();
    std::fs::write(root.join("sub").join("b.markdown"), "x").unwrap();
    let tree = build_tree(root).unwrap();
    assert!(tree.is_dir);
    // count markdown files in tree
    fn count(n: &notecraft::vault::tree::FileNode) -> usize {
        if n.is_dir {
            n.children.iter().map(count).sum()
        } else {
            1
        }
    }
    assert_eq!(count(&tree), 2);
}

#[test]
fn create_note_and_collision() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let p = create_note(root, root, "note").unwrap();
    assert!(p.exists());
    assert!(is_markdown(&p));
    // collision
    let r = create_note(root, root, "note");
    assert!(r.is_err());
}

#[test]
fn rename_and_delete() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let p = create_note(root, root, "old.md").unwrap();
    let newp = rename_path(root, &p, "new.md").unwrap();
    assert!(newp.exists());
    assert!(!p.exists());
    delete_path(root, &newp).unwrap();
    assert!(!newp.exists());
}

#[test]
fn path_traversal_blocked() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let outside = dir.path().parent().unwrap().join("evil.md");
    let r = ensure_inside(root, &outside);
    assert!(r.is_err());
}

#[test]
fn utf8_read_write() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let p = root.join("u.md");
    std::fs::write(&p, "Héllo 世界").unwrap();
    let s = read_file_strict(&p).unwrap();
    assert_eq!(s, "Héllo 世界");
}

#[test]
fn safe_write_conflict() {
    use notecraft::vault::safe_write::{atomic_write, file_meta};
    let dir = tempdir().unwrap();
    let root = dir.path();
    let p = root.join("c.md");
    std::fs::write(&p, "v1").unwrap();
    let meta = file_meta(&p).unwrap();
    // external modify
    std::fs::write(&p, "external").unwrap();
    let r = atomic_write(&p, "v2", Some(meta));
    assert!(r.is_err());
}
