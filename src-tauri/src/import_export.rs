use crate::error::AppError;
use crate::error::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportReport {
    pub imported: usize,
    pub broken_links: Vec<String>,
}

pub fn import_vault(src: &str, dest: &str) -> AppResult<ImportReport> {
    let src = PathBuf::from(src);
    let dest = PathBuf::from(dest);
    if !src.is_dir() {
        return Err(AppError::Invalid("source not a directory".into()));
    }
    std::fs::create_dir_all(&dest).map_err(AppError::Io)?;
    let mut imported = 0usize;
    let mut broken = Vec::new();
    for entry in walkdir::WalkDir::new(&src) {
        let entry = entry.map_err(|e| AppError::Other(e.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(&src)
            .map_err(|e| AppError::Other(e.to_string()))?;
        let target = dest.join(rel);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
        std::fs::copy(entry.path(), &target).map_err(AppError::Io)?;
        imported += 1;
        if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let parsed = crate::markdown::parse(&content);
                // check links resolve within dest later; collect targets for report
                for l in &parsed.links {
                    if l.target.ends_with(".md") {
                        let candidate = dest.join(&l.target);
                        if !candidate.exists() {
                            broken.push(l.target.clone());
                        }
                    }
                }
            }
        }
    }
    broken.sort();
    broken.dedup();
    Ok(ImportReport {
        imported,
        broken_links: broken,
    })
}

pub fn backup(state: &State<AppState>, dest: &str) -> AppResult<()> {
    let root = {
        let g = state.inner.lock();
        g.vault_root.clone().ok_or(AppError::NoVault)?
    };
    let dest = PathBuf::from(dest);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(AppError::Io)?;
    }
    let file = std::fs::File::create(&dest).map_err(AppError::Io)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for entry in walkdir::WalkDir::new(&root) {
        let entry = entry.map_err(|e| AppError::Other(e.to_string()))?;
        let rel = entry
            .path()
            .strip_prefix(&root)
            .map_err(|e| AppError::Other(e.to_string()))?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if rel_str.is_empty() {
            continue;
        }
        if entry.file_type().is_dir() {
            zip.add_directory(rel_str, opts)
                .map_err(|e| AppError::Other(e.to_string()))?;
        } else {
            zip.start_file(rel_str, opts)
                .map_err(|e| AppError::Other(e.to_string()))?;
            let data = std::fs::read(entry.path()).map_err(AppError::Io)?;
            use std::io::Write;
            zip.write_all(&data)
                .map_err(|e| AppError::Other(e.to_string()))?;
        }
    }
    zip.finish().map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

pub fn restore_backup(src: &str, dest: &str) -> AppResult<()> {
    let src = PathBuf::from(src);
    let dest = PathBuf::from(dest);
    std::fs::create_dir_all(&dest).map_err(AppError::Io)?;
    let file = std::fs::File::open(&src).map_err(AppError::Io)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| AppError::Other(e.to_string()))?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::Other(e.to_string()))?;
        let outpath = dest.join(entry.name());
        if entry.is_dir() {
            std::fs::create_dir_all(&outpath).map_err(AppError::Io)?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p).map_err(AppError::Io)?;
            }
            let mut outfile = std::fs::File::create(&outpath).map_err(AppError::Io)?;
            use std::io::copy;
            copy(&mut entry, &mut outfile).map_err(AppError::Io)?;
        }
    }
    Ok(())
}

/// Write a file the user explicitly chose a destination for.
pub fn write_file(dest: &str, content: &str) -> AppResult<()> {
    let path = PathBuf::from(dest);
    if path.as_os_str().is_empty() {
        return Err(AppError::Invalid("no destination was chosen".into()));
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
    }
    std::fs::write(&path, content).map_err(AppError::Io)
}

const EXPORT_CSS: &str = r##"
:root { color-scheme: light; }
* { box-sizing: border-box; }
body {
  margin: 0 auto; padding: 56px 40px 72px; max-width: 780px;
  font: 16px/1.7 ui-sans-serif, -apple-system, "Segoe UI", Roboto, sans-serif;
  color: #1c1917; background: #fffdf9;
}
h1, h2, h3, h4, h5, h6 { line-height: 1.25; margin: 1.6em 0 .5em; letter-spacing: -.01em; }
h1 { font-size: 2em; } h2 { font-size: 1.5em; } h3 { font-size: 1.2em; }
p, ul, ol, table { margin: .75em 0; }
ul, ol { padding-left: 1.4em; }
li { margin: .2em 0; }
li.nc-task { list-style: none; margin-left: -1.2em; }
li.nc-task input { margin-right: .5em; }
li.nc-task-done { color: #78716c; }
blockquote { margin: 1em 0; padding: .35em 1.1em; border-left: 3px solid #d6d3d1; color: #44403c; }
.nc-callout { margin: 1em 0; padding: .8em 1.1em; border: 1px solid #e7e5e4; border-radius: 10px; background: #faf9f7; }
.nc-callout-title { font-size: .72em; letter-spacing: .09em; text-transform: uppercase; font-weight: 700; color: #78716c; }
code { font-family: ui-monospace, "Cascadia Mono", Consolas, monospace; font-size: .9em; background: #f3efe9; padding: .12em .35em; border-radius: 4px; }
pre { padding: 14px 16px; background: #f6f2ec; border: 1px solid #e7e0d6; border-radius: 10px; overflow-x: auto; }
pre code { background: none; padding: 0; }
table { border-collapse: collapse; width: 100%; }
th, td { border: 1px solid #e2ddd6; padding: 7px 11px; text-align: left; font-size: .95em; }
th { background: #faf7f2; font-weight: 600; }
img { max-width: 100%; border-radius: 8px; }
hr { border: none; border-top: 1px solid #e2ddd6; margin: 1.8em 0; }
a { color: #6d28d9; }
a::after { content: " (" attr(href) ")"; font-size: .78em; color: #78716c; }
a[href^="#"]::after, a.nc-wikilink::after { content: ""; }
mark { background: #fde68a; }
.nc-frontmatter { margin: 0 0 1.6em; padding: .8em 1em; border: 1px solid #e7e5e4; border-radius: 10px; background: #faf9f7; font-size: .88em; }
.nc-frontmatter-title { font-size: .7em; text-transform: uppercase; letter-spacing: .09em; color: #78716c; margin-bottom: .35em; }
.nc-frontmatter dl { display: grid; grid-template-columns: minmax(90px, auto) 1fr; gap: .25em 1em; margin: 0; }
.nc-frontmatter dt { font-weight: 600; } .nc-frontmatter dd { margin: 0; }
.nc-mdx-block { margin: 1em 0; border: 1px solid #e7e5e4; border-radius: 10px; overflow: hidden; }
.nc-mdx-label { padding: .35em .8em; background: #f6f2ec; font-family: ui-monospace, Consolas, monospace; font-size: .78em; color: #57534e; }
.nc-mdx-source { margin: 0; border: none; border-radius: 0; }
.nc-footnotes { margin-top: 2.4em; border-top: 1px solid #e2ddd6; padding-top: .8em; font-size: .9em; color: #57534e; }
@media print { body { padding: 12mm; max-width: none; background: #fff; } a::after { content: ""; } }
"##;

/// A standalone, offline HTML document for one note. Used by the editor's
/// export menu and by the print-to-PDF path.
pub fn export_document_html(content: &str, rel: &str) -> String {
    let body = crate::markdown::render_document(content, rel);
    let name = rel.rsplit('/').next().unwrap_or(rel);
    let title = name
        .trim_end_matches(".md")
        .trim_end_matches(".markdown")
        .trim_end_matches(".mdx")
        .trim_end_matches(".json");
    let title_escaped = title
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>{title_escaped}</title><style>{EXPORT_CSS}</style></head>\
         <body>{body}</body></html>"
    )
}
