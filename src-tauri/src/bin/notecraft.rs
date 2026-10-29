//! `notecraft-cli` — a local command line interface to the same encrypted
//! vaults the desktop app uses.
//!
//! Every command that touches notes must unlock a profile with the account's
//! username and password, exactly like the app:
//!
//! ```text
//! notecraft-cli list --user ada --password 'correct horse'
//! NOTECRAFT_PASSWORD=... notecraft-cli search "todo" --json
//! ```
//!
//! `--json` prints a single JSON document so other tools and agents can parse
//! results without scraping text.

use notecraft_lib::{secure, vault};
use std::io::{Read, Write};
use std::path::PathBuf;

const USAGE: &str = "\
notecraft-cli — use NoteCraft vaults from a terminal

USAGE:
  notecraft-cli <command> [options]

COMMANDS:
  profiles                       List local profiles (no password needed)
  list [folder]                  List notes in the profile vault
  read <path>                    Print a note
  write <path>                   Replace a note (--content TEXT, --file PATH, or --stdin)
  new <title>                    Create a note and print its path
  append <path>                  Append text to a note
  search <query>                 Search note text and paths
  tasks                          List open tasks with their source note
  delete <path>                  Move a note to the vault trash

AUTH:
  --user NAME --password PASS    Or NOTECRAFT_USER / NOTECRAFT_PASSWORD
  A profile must already exist; create it in the desktop app.

OPTIONS:
  --json                         Machine readable output
  --folder DIR                   Limit list/search to a folder
  --history                      Accepted for compatibility; no-op
  -h, --help                     Show this help
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print!("{USAGE}");
        std::process::exit(if args.is_empty() { 2 } else { 0 });
    }

    match run(&args) {
        Ok(()) => {}
        Err(message) => {
            let _ = writeln!(std::io::stderr(), "error: {message}");
            std::process::exit(1);
        }
    }
}

#[derive(Default)]
struct Options {
    positionals: Vec<String>,
    user: Option<String>,
    password: Option<String>,
    content: Option<String>,
    file: Option<String>,
    stdin: bool,
    json: bool,
    folder: Option<String>,
}

fn parse(args: &[String]) -> Result<(String, Options), String> {
    let mut command = String::new();
    let mut options = Options::default();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--json" => options.json = true,
            "--stdin" => options.stdin = true,
            "--user" | "--username" => {
                index += 1;
                options.user = args.get(index).cloned();
            }
            "--password" | "--pass" => {
                index += 1;
                options.password = args.get(index).cloned();
            }
            "--content" => {
                index += 1;
                options.content = args.get(index).cloned();
            }
            "--file" => {
                index += 1;
                options.file = args.get(index).cloned();
            }
            "--folder" => {
                index += 1;
                options.folder = args.get(index).cloned();
            }
            other if other.starts_with("--user=") => {
                options.user = Some(other["--user=".len()..].to_string());
            }
            other if other.starts_with("--password=") => {
                options.password = Some(other["--password=".len()..].to_string());
            }
            other if other.starts_with('-') && other != "-" => {
                return Err(format!("unknown option {other}"));
            }
            other => {
                if command.is_empty() {
                    command = other.to_string();
                } else {
                    options.positionals.push(other.to_string());
                }
            }
        }
        index += 1;
    }
    if command.is_empty() {
        return Err("missing command".into());
    }
    Ok((command, options))
}

/// Resolve the profile and install its key for this process.
fn session(options: &Options) -> Result<secure::ProfileSummary, String> {
    let user = options
        .user
        .clone()
        .or_else(|| std::env::var("NOTECRAFT_USER").ok())
        .ok_or_else(|| "missing --user (or NOTECRAFT_USER)".to_string())?;
    let password = options
        .password
        .clone()
        .or_else(|| std::env::var("NOTECRAFT_PASSWORD").ok())
        .ok_or_else(|| "missing --password (or NOTECRAFT_PASSWORD)".to_string())?;
    let profile = secure::unlock(&user, &password).map_err(|error| error.to_string())?;
    Ok(profile)
}

fn vault_root(profile: &secure::ProfileSummary) -> PathBuf {
    PathBuf::from(&profile.vault_path)
}

fn print_json(value: serde_json::Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?;
    println!("{text}");
    Ok(())
}

fn run(args: &[String]) -> Result<(), String> {
    let (command, options) = parse(args)?;

    if command == "profiles" {
        let profiles = secure::list_profiles();
        if options.json {
            let items: Vec<serde_json::Value> = profiles
                .iter()
                .map(|profile| {
                    serde_json::json!({
                        "id": profile.id,
                        "username": profile.username,
                        "created_at": profile.created_at,
                        "vault_path": profile.vault_path,
                    })
                })
                .collect();
            return print_json(serde_json::json!({ "profiles": items }));
        }
        if profiles.is_empty() {
            println!("No profiles yet. Create one in the NoteCraft desktop app.");
        }
        for profile in profiles {
            println!(
                "{:<24} {}  {}",
                profile.username, profile.created_at, profile.vault_path
            );
        }
        return Ok(());
    }

    let profile = session(&options)?;
    let root = vault_root(&profile);

    match command.as_str() {
        "list" => {
            let folder = options
                .positionals
                .first()
                .cloned()
                .or(options.folder.clone())
                .unwrap_or_default();
            let mut entries: Vec<vault::FileEntry> = vault::list_all_files(&root, &[], false)
                .into_iter()
                .filter(|entry| folder.is_empty() || entry.path.starts_with(&folder))
                .filter(|entry| {
                    entry.extension == "md"
                        || entry.extension == "markdown"
                        || entry.extension == "canvas"
                })
                .collect();
            entries.sort_by(|a, b| a.path.cmp(&b.path));
            if options.json {
                let items: Vec<serde_json::Value> = entries
                    .iter()
                    .map(|entry| {
                        serde_json::json!({
                            "path": entry.path,
                            "name": entry.name,
                            "size": entry.size,
                            "modified": entry.modified,
                        })
                    })
                    .collect();
                return print_json(serde_json::json!({
                    "profile": profile.username,
                    "root": profile.vault_path,
                    "notes": items,
                }));
            }
            for entry in entries {
                println!("{}", entry.path);
            }
            Ok(())
        }
        "read" => {
            let path = options
                .positionals
                .first()
                .ok_or_else(|| "usage: read <path>".to_string())?;
            let content = vault::read_file(&root, path).map_err(|error| error.to_string())?;
            if options.json {
                print_json(serde_json::json!({ "path": path, "content": content }))
            } else {
                print!("{content}");
                Ok(())
            }
        }
        "write" => {
            let path = options
                .positionals
                .first()
                .ok_or_else(|| "usage: write <path> --content TEXT | --stdin".to_string())?;
            let content = read_content(&options)?;
            vault::write_file_atomic(&root, path, &content).map_err(|error| error.to_string())?;
            if options.json {
                print_json(serde_json::json!({ "written": path, "bytes": content.len() }))
            } else {
                println!("wrote {path} ({} bytes)", content.len());
                Ok(())
            }
        }
        "new" => {
            let title = options
                .positionals
                .first()
                .ok_or_else(|| "usage: new <title>".to_string())?;
            let folder = options.folder.clone().unwrap_or_default();
            let rel = unique_note_path(&root, &folder, title);
            let heading = format!("# {title}\n\n");
            vault::create_file(&root, &rel, &heading).map_err(|error| error.to_string())?;
            if options.json {
                print_json(serde_json::json!({ "created": rel }))
            } else {
                println!("{rel}");
                Ok(())
            }
        }
        "append" => {
            let path = options
                .positionals
                .first()
                .ok_or_else(|| "usage: append <path> --content TEXT".to_string())?;
            let extra = read_content(&options)?;
            let existing = vault::read_file(&root, path).unwrap_or_default();
            let mut merged = existing;
            if !merged.is_empty() && !merged.ends_with('\n') {
                merged.push('\n');
            }
            merged.push_str(&extra);
            if !merged.ends_with('\n') {
                merged.push('\n');
            }
            vault::write_file_atomic(&root, path, &merged).map_err(|error| error.to_string())?;
            if options.json {
                print_json(serde_json::json!({ "appended": path }))
            } else {
                println!("appended to {path}");
                Ok(())
            }
        }
        "search" => {
            let query = options.positionals.join(" ").trim().to_string();
            if query.is_empty() {
                return Err("usage: search <query>".into());
            }
            let needle = query.to_lowercase();
            let folder = options.folder.clone().unwrap_or_default();
            let mut hits: Vec<serde_json::Value> = Vec::new();
            for path in vault::walk_markdown(&root, &[]) {
                let rel = vault::to_rel(&root, &path);
                if !folder.is_empty() && !rel.starts_with(&folder) {
                    continue;
                }
                let Ok(content) = secure::read_note_text(&path) else {
                    continue;
                };
                if rel.to_lowercase().contains(&needle) {
                    hits.push(serde_json::json!({ "path": rel, "line": 0, "text": rel }));
                }
                for (index, line) in content.lines().enumerate() {
                    if line.to_lowercase().contains(&needle) {
                        hits.push(serde_json::json!({
                            "path": rel,
                            "line": index + 1,
                            "text": line.trim(),
                        }));
                        if hits.len() >= 200 {
                            break;
                        }
                    }
                }
                if hits.len() >= 200 {
                    break;
                }
            }
            if options.json {
                print_json(serde_json::json!({ "query": query, "hits": hits }))
            } else {
                for hit in &hits {
                    println!(
                        "{}:{}: {}",
                        hit["path"].as_str().unwrap_or(""),
                        hit["line"].as_u64().unwrap_or(0),
                        hit["text"].as_str().unwrap_or("")
                    );
                }
                Ok(())
            }
        }
        "tasks" => {
            let mut items: Vec<serde_json::Value> = Vec::new();
            for path in vault::walk_markdown(&root, &[]) {
                let rel = vault::to_rel(&root, &path);
                let Ok(content) = secure::read_note_text(&path) else {
                    continue;
                };
                for (index, line) in content.lines().enumerate() {
                    let trimmed = line.trim_start();
                    let open = trimmed.starts_with("- [ ]") || trimmed.starts_with("* [ ]");
                    let done = trimmed.starts_with("- [x]") || trimmed.starts_with("* [x]");
                    if !open && !done {
                        continue;
                    }
                    items.push(serde_json::json!({
                        "path": rel,
                        "line": index + 1,
                        "done": done,
                        "text": trimmed[5..].trim(),
                    }));
                }
            }
            if options.json {
                print_json(serde_json::json!({ "tasks": items }))
            } else {
                for item in &items {
                    let marker = if item["done"].as_bool().unwrap_or(false) {
                        "x"
                    } else {
                        " "
                    };
                    println!(
                        "[{marker}] {} ({}:{})",
                        item["text"].as_str().unwrap_or(""),
                        item["path"].as_str().unwrap_or(""),
                        item["line"].as_u64().unwrap_or(0)
                    );
                }
                Ok(())
            }
        }
        "delete" => {
            let path = options
                .positionals
                .first()
                .ok_or_else(|| "usage: delete <path>".to_string())?;
            vault::delete_path(&root, path, true).map_err(|error| error.to_string())?;
            if options.json {
                print_json(serde_json::json!({ "trashed": path }))
            } else {
                println!("moved {path} to the vault trash");
                Ok(())
            }
        }
        other => Err(format!("unknown command {other}\n\n{USAGE}")),
    }
}

fn read_content(options: &Options) -> Result<String, String> {
    if let Some(path) = &options.file {
        return std::fs::read_to_string(path).map_err(|error| error.to_string());
    }
    if options.stdin || options.content.is_none() {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| error.to_string())?;
        return Ok(buffer);
    }
    Ok(options.content.clone().unwrap_or_default())
}

fn unique_note_path(root: &PathBuf, folder: &str, title: &str) -> String {
    let stem = title
        .chars()
        .map(|character| match character {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c => c,
        })
        .collect::<String>();
    let stem = stem.trim().trim_matches('.').to_string();
    let stem = if stem.is_empty() {
        "Untitled".to_string()
    } else {
        stem
    };
    let base = if folder.is_empty() {
        stem.clone()
    } else {
        format!("{}/{stem}", folder.trim_matches('/'))
    };
    let mut candidate = format!("{base}.md");
    let mut counter = 2;
    while root.join(&candidate).exists() {
        candidate = format!("{base} {counter}.md");
        counter += 1;
    }
    candidate
}
