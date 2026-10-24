use notecraft::markdown::frontmatter::parse_frontmatter;
use notecraft::markdown::tags::extract_inline_tags;

#[test]
fn frontmatter_parse() {
    let input = "---\ntitle: Hello\ntags: [a, b]\n---\nBody text #tag";
    let parsed = parse_frontmatter(input);
    assert!(parsed.frontmatter.is_some());
    let fm = parsed.frontmatter.unwrap();
    assert_eq!(fm.title.as_deref(), Some("Hello"));
    assert_eq!(fm.tags, vec!["a".to_string(), "b".to_string()]);
    assert!(parsed.body.contains("Body text"));
}

#[test]
fn frontmatter_malformed() {
    let input = "---\n: bad\n---\nBody";
    let parsed = parse_frontmatter(input);
    // should not panic; frontmatter may be None
    assert!(parsed.body.contains("Body"));
}

#[test]
fn inline_tags() {
    let tags = extract_inline_tags("Hello #rust and #note-taking");
    assert!(tags.contains(&"rust".to_string()));
    assert!(tags.contains(&"note-taking".to_string()));
}

#[test]
fn no_frontmatter() {
    let input = "Just text";
    let parsed = parse_frontmatter(input);
    assert!(parsed.frontmatter.is_none());
    assert_eq!(parsed.body, "Just text");
}
