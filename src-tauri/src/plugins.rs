use crate::error::AppError;
use crate::error::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub enabled: bool,
    pub permissions: Vec<String>,
}

pub fn list(state: &State<AppState>) -> AppResult<Vec<PluginInfo>> {
    let g = state.inner.lock();
    let mut out = vec![];
    if let Some(conn) = g.db.as_ref() {
        let mut stmt = conn
            .prepare("SELECT id,manifest,enabled FROM plugins")
            .map_err(AppError::Sqlite)?;
        let rows = stmt
            .query_map([], |r| {
                let id: String = r.get(0)?;
                let m: String = r.get(1)?;
                let enabled: i64 = r.get(2)?;
                let v: Value = serde_json::from_str(&m).unwrap_or(Value::Null);
                Ok(PluginInfo {
                    id: id.clone(),
                    name: v
                        .get("name")
                        .and_then(|x| x.as_str())
                        .unwrap_or(&id)
                        .to_string(),
                    version: v
                        .get("version")
                        .and_then(|x| x.as_str())
                        .unwrap_or("0.0.0")
                        .to_string(),
                    author: v
                        .get("author")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: v
                        .get("description")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string(),
                    enabled: enabled != 0,
                    permissions: v
                        .get("permissions")
                        .and_then(|x| x.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default(),
                })
            })
            .map_err(AppError::Sqlite)?;
        for r in rows {
            if let Ok(p) = r {
                out.push(p);
            }
        }
    }
    Ok(out)
}

pub fn enable(state: &State<AppState>, id: &str, enabled: bool) -> AppResult<()> {
    let g = state.inner.lock();
    if let Some(conn) = g.db.as_ref() {
        conn.execute(
            "UPDATE plugins SET enabled=?1 WHERE id=?2",
            rusqlite::params![if enabled { 1 } else { 0 }, id],
        )
        .map_err(AppError::Sqlite)?;
    }
    Ok(())
}

pub fn uninstall(state: &State<AppState>, id: &str) -> AppResult<()> {
    {
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            conn.execute("DELETE FROM plugins WHERE id=?1", rusqlite::params![id])
                .map_err(AppError::Sqlite)?;
        }
    }
    let dir = plugins_dir().join(sanitize_id(id));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(AppError::Io)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------
// Installable plugin packages
//
// A package is a zip the user drops on the settings page. It must contain a
// `plugin.json` manifest and a JavaScript entry point; `style.css` and
// `icon.png` are optional. Everything is extracted under the app data folder.
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub permissions: Vec<String>,
    pub main: Option<String>,
    /// Alias some packages use instead of `main`.
    pub entry: Option<String>,
    pub style: Option<String>,
    pub icon: Option<String>,
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: "0.0.1".into(),
            description: String::new(),
            author: String::new(),
            permissions: Vec::new(),
            main: None,
            entry: None,
            style: None,
            icon: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginPackage {
    pub id: String,
    pub entry: String,
    pub code: String,
    pub style: Option<String>,
    pub icon: Option<String>,
}

fn plugins_dir() -> std::path::PathBuf {
    crate::secure::app_dir().join("plugins")
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .take(64)
        .collect()
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && !id.starts_with('.')
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// Find a file in the archive by exact path, then by name, so packages made
/// with a wrapping folder (the common zip layout) still install.
fn find_file(archive: &mut zip::ZipArchive<std::fs::File>, wanted: &str) -> Option<String> {
    let wanted_slash = format!("/{wanted}");
    for index in 0..archive.len() {
        let entry = archive.by_index(index).ok()?;
        if entry.name().eq_ignore_ascii_case(wanted) {
            return Some(entry.name().to_string());
        }
    }
    for index in 0..archive.len() {
        let entry = archive.by_index(index).ok()?;
        let name = entry.name().to_ascii_lowercase();
        if name.ends_with(&wanted_slash.to_ascii_lowercase()) {
            return Some(entry.name().to_string());
        }
    }
    None
}

fn file_ends_with(archive: &mut zip::ZipArchive<std::fs::File>, suffix: &str) -> Option<String> {
    for index in 0..archive.len() {
        let entry = archive.by_index(index).ok()?;
        if entry.name().to_ascii_lowercase().ends_with(suffix) {
            return Some(entry.name().to_string());
        }
    }
    None
}

/// Install a plugin from a `.zip` the user picked or dropped onto the window.
pub fn import_zip(path: &str, state: &State<AppState>) -> AppResult<PluginInfo> {
    let file = std::fs::File::open(path).map_err(AppError::Io)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| AppError::Invalid(e.to_string()))?;

    let manifest_name = find_file(&mut archive, "plugin.json")
        .or_else(|| find_file(&mut archive, "manifest.json"))
        .ok_or_else(|| {
            AppError::Invalid("the zip has no plugin.json (or manifest.json) at its root".into())
        })?;

    let mut manifest_text = String::new();
    {
        use std::io::Read;
        let mut entry = archive.by_name(&manifest_name).map_err(AppError::Io)?;
        entry
            .read_to_string(&mut manifest_text)
            .map_err(AppError::Io)?;
    }
    let mut manifest: PluginManifest = serde_json::from_str(&manifest_text)
        .map_err(|e| AppError::Invalid(format!("plugin.json is invalid: {e}")))?;
    if manifest.id.trim().is_empty() {
        return Err(AppError::Invalid("plugin.json needs an id".into()));
    }
    manifest.id = manifest.id.trim().to_string();
    if !valid_id(&manifest.id) {
        return Err(AppError::Invalid(
            "plugin ids may only use letters, digits, dashes and underscores".into(),
        ));
    }
    if manifest.name.trim().is_empty() {
        manifest.name = manifest.id.clone();
    }
    if manifest.version.trim().is_empty() {
        manifest.version = "0.0.1".into();
    }

    let declared_entry = manifest.main.clone().or_else(|| manifest.entry.clone());
    let entry_name = match declared_entry {
        Some(main) => find_file(&mut archive, &main).ok_or_else(|| {
            AppError::Invalid(format!("plugin.json points at a missing file: {main}"))
        })?,
        None => find_file(&mut archive, "index.js")
            .or_else(|| find_file(&mut archive, "dist/index.js"))
            .or_else(|| file_ends_with(&mut archive, ".js"))
            .or_else(|| file_ends_with(&mut archive, ".ts"))
            .ok_or_else(|| {
                AppError::Invalid("the zip has no JavaScript entry point (index.js)".into())
            })?,
    };
    let lower_entry = entry_name.to_ascii_lowercase();
    let mut entry_name = entry_name;
    if lower_entry.ends_with(".ts") {
        // TypeScript sources are kept in the package, but the runtime loads the
        // compiled JavaScript beside them.
        match find_file(
            &mut archive,
            &format!("{}.js", entry_name.trim_end_matches(".ts")),
        ) {
            Some(compiled) => entry_name = compiled,
            None => {
                return Err(AppError::Invalid(
                    "TypeScript plugins must ship the compiled .js entry point".into(),
                ))
            }
        }
    }
    manifest.main = Some(entry_name);

    let dir = plugins_dir().join(&manifest.id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(AppError::Io)?;
    }
    std::fs::create_dir_all(&dir).map_err(AppError::Io)?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(AppError::Io)?;
        let rel = match entry.enclosed_name() {
            Some(rel) => rel.to_path_buf(),
            None => continue,
        };
        let target = dir.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&target).map_err(AppError::Io)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
        let mut out = std::fs::File::create(&target).map_err(AppError::Io)?;
        std::io::copy(&mut entry, &mut out).map_err(AppError::Io)?;
    }

    let manifest_json =
        serde_json::to_string(&manifest).map_err(|e| AppError::Other(e.to_string()))?;
    {
        let g = state.inner.lock();
        if let Some(conn) = g.db.as_ref() {
            conn.execute(
                "INSERT INTO plugins(id,manifest,enabled,installed_at) VALUES(?1,?2,1,?3) \
                 ON CONFLICT(id) DO UPDATE SET manifest=excluded.manifest, enabled=1",
                rusqlite::params![manifest.id, manifest_json, chrono::Utc::now().to_rfc3339()],
            )
            .map_err(AppError::Sqlite)?;
        }
    }

    Ok(PluginInfo {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        author: manifest.author.clone(),
        description: manifest.description.clone(),
        enabled: true,
        permissions: manifest.permissions.clone(),
    })
}

/// Load the files the webview needs to run an installed plugin.
pub fn package(state: &State<AppState>, id: &str) -> AppResult<PluginPackage> {
    let manifest: PluginManifest = {
        let g = state.inner.lock();
        let conn =
            g.db.as_ref()
                .ok_or_else(|| AppError::Invalid("the plugin database is not open".into()))?;
        let text: String = conn
            .query_row(
                "SELECT manifest FROM plugins WHERE id=?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .map_err(|_| AppError::NotFound(id.into()))?;
        serde_json::from_str(&text).unwrap_or_default()
    };

    let dir = plugins_dir().join(sanitize_id(id));
    let entry = manifest
        .main
        .clone()
        .filter(|value| dir.join(value).exists())
        .or_else(|| {
            ["index.js", "dist/index.js", "main.js"]
                .iter()
                .find(|name| dir.join(name).exists())
                .map(|name| name.to_string())
        })
        .ok_or_else(|| AppError::NotFound("the plugin has no entry file".into()))?;
    let code = std::fs::read_to_string(dir.join(&entry)).map_err(AppError::Io)?;

    let style = manifest
        .style
        .clone()
        .filter(|value| dir.join(value).exists())
        .or_else(|| {
            ["style.css", "styles.css", "dist/style.css"]
                .iter()
                .find(|name| dir.join(name).exists())
                .map(|name| name.to_string())
        })
        .and_then(|name| std::fs::read_to_string(dir.join(name)).ok());

    let icon = manifest
        .icon
        .clone()
        .filter(|value| dir.join(value).exists())
        .or_else(|| {
            ["icon.png", "logo.png", "icon.svg"]
                .iter()
                .find(|name| dir.join(name).exists())
                .map(|name| name.to_string())
        })
        .and_then(|name| {
            let bytes = std::fs::read(dir.join(&name)).ok()?;
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
            let mime = if name.to_ascii_lowercase().ends_with(".svg") {
                "image/svg+xml"
            } else {
                "image/png"
            };
            Some(format!("data:{mime};base64,{encoded}"))
        });

    Ok(PluginPackage {
        id: id.to_string(),
        entry,
        code,
        style,
        icon,
    })
}
