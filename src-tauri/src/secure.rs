//! Encrypted local profiles.
//!
//! Every profile is a separate local user. A profile owns:
//!
//! ```text
//! %APPDATA%/NoteCraft/profiles/<id>/vault/    notes, encrypted at rest
//! %APPDATA%/NoteCraft/profiles/<id>/notecraft.db
//! %APPDATA%/NoteCraft/profiles/<id>/index/    derived search index
//! ```
//!
//! The password never touches disk. `profiles.json` stores a random salt and a
//! verifier blob that can only be decrypted with the key derived from the
//! password, so a wrong password fails closed before any note is read.
//!
//! Note files are stored in a small self-describing container:
//!
//! ```text
//! "NCV1" | 12-byte nonce | ChaCha20-Poly1305 ciphertext (+ tag)
//! ```
//!
//! Files that do not start with the magic bytes are treated as legacy
//! plaintext and are encrypted in place the first time a profile is unlocked.

use crate::error::{AppError, AppResult};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// Container magic for encrypted note files.
pub const MAGIC: &[u8; 4] = b"NCV1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
/// Minimum accepted password length for new profiles and CLI unlocks.
pub const MIN_PASSWORD_LEN: usize = 8;

/// Session key for the unlocked profile. `None` while locked, which means all
/// note reads/writes fail instead of silently writing plaintext.
static SESSION_KEY: RwLock<Option<[u8; KEY_LEN]>> = RwLock::new(None);
/// Id of the unlocked profile, used to keep derived data (search index) inside
/// that profile's own directory.
static ACTIVE_PROFILE: RwLock<Option<String>> = RwLock::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSummary {
    pub id: String,
    pub username: String,
    pub created_at: String,
    /// Absolute path of the encrypted vault directory.
    pub vault_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfileRecord {
    id: String,
    username: String,
    created_at: String,
    salt: String,
    verifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Registry {
    version: u32,
    #[serde(default)]
    profiles: Vec<ProfileRecord>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            version: 1,
            profiles: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------- key material

fn kdf(password: &str, salt: &[u8]) -> AppResult<[u8; KEY_LEN]> {
    use argon2::{Algorithm, Argon2, Params, Version};
    let params = Params::new(19 * 1024, 2, 1, Some(KEY_LEN))
        .map_err(|error| AppError::Other(format!("kdf parameters: {error}")))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|error| AppError::Other(format!("key derivation failed: {error}")))?;
    Ok(key)
}

fn cipher(key: &[u8; KEY_LEN]) -> AppResult<ChaCha20Poly1305> {
    ChaCha20Poly1305::new_from_slice(key).map_err(|_| AppError::Other("invalid key length".into()))
}

/// Encrypt a payload into the `NCV1` container format.
pub fn seal(key: &[u8; KEY_LEN], plaintext: &[u8]) -> AppResult<Vec<u8>> {
    let cipher = cipher(key)?;
    let nonce_bytes = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|error| AppError::Other(format!("encryption failed: {error}")))?;
    let mut out = Vec::with_capacity(MAGIC.len() + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(nonce);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt an `NCV1` container.
pub fn open(key: &[u8; KEY_LEN], blob: &[u8]) -> AppResult<Vec<u8>> {
    if !is_container(blob) {
        return Err(AppError::Invalid("not an encrypted note".into()));
    }
    let cipher = cipher(key)?;
    let nonce_start = MAGIC.len();
    let nonce = Nonce::from_slice(&blob[nonce_start..nonce_start + NONCE_LEN]);
    let body = &blob[nonce_start + NONCE_LEN..];
    cipher
        .decrypt(nonce, body)
        .map_err(|_| AppError::Permission("wrong password or corrupted note".into()))
}

/// True when a file payload uses the encrypted container format.
pub fn is_container(blob: &[u8]) -> bool {
    blob.len() > MAGIC.len() + NONCE_LEN && &blob[..MAGIC.len()] == MAGIC
}

/// Install (or clear) the session key for the unlocked profile.
pub fn set_session_key(key: Option<[u8; KEY_LEN]>) {
    let mut guard = SESSION_KEY.write().unwrap_or_else(|e| e.into_inner());
    *guard = key;
}

pub fn session_key() -> Option<[u8; KEY_LEN]> {
    *SESSION_KEY.read().unwrap_or_else(|e| e.into_inner())
}

pub fn is_unlocked() -> bool {
    session_key().is_some()
}

/// Remember which profile owns the current session key.
pub fn set_active_profile(id: Option<String>) {
    let mut guard = ACTIVE_PROFILE.write().unwrap_or_else(|e| e.into_inner());
    *guard = id;
}

pub fn active_profile() -> Option<String> {
    ACTIVE_PROFILE
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

/// True when the payload can be read right now: plaintext always, containers
/// only while a profile is unlocked.
pub fn can_read(blob: &[u8]) -> bool {
    !is_container(blob) || is_unlocked()
}

// ------------------------------------------------------------------- file io

/// Read a note file and decrypt it with the session key when needed.
pub fn read_note_file(path: &Path) -> AppResult<Vec<u8>> {
    let blob = std::fs::read(path).map_err(AppError::Io)?;
    if is_container(&blob) {
        let key = session_key().ok_or_else(|| AppError::Permission("profile is locked".into()))?;
        open(&key, &blob)
    } else {
        Ok(blob)
    }
}

/// Read a note file as UTF-8 text.
pub fn read_note_text(path: &Path) -> AppResult<String> {
    let bytes = read_note_file(path)?;
    String::from_utf8(bytes).map_err(|_| AppError::Invalid("note is not valid UTF-8".into()))
}

/// Write a note file atomically, encrypting it when a profile is unlocked.
pub fn write_note_file(path: &Path, content: &[u8]) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(AppError::Io)?;
    }
    let payload = match session_key() {
        Some(key) => seal(&key, content)?,
        None => content.to_vec(),
    };
    let tmp = path.with_extension("notecraft-tmp");
    {
        let mut file = std::fs::File::create(&tmp).map_err(AppError::Io)?;
        file.write_all(&payload).map_err(AppError::Io)?;
        let _ = file.sync_all();
    }
    std::fs::rename(&tmp, path).map_err(AppError::Io)?;
    Ok(())
}

/// Encrypt every legacy plaintext markdown file under `root`, in place.
/// Returns how many files were migrated.
pub fn encrypt_plaintext_notes(root: &Path) -> AppResult<usize> {
    let Some(key) = session_key() else {
        return Ok(0);
    };
    let mut migrated = 0;
    for path in crate::vault::walk_markdown(root, &[]) {
        let Ok(blob) = std::fs::read(&path) else {
            continue;
        };
        if is_container(&blob) {
            continue;
        }
        let sealed = seal(&key, &blob)?;
        let tmp = path.with_extension("notecraft-tmp");
        if std::fs::write(&tmp, &sealed).is_ok() && std::fs::rename(&tmp, &path).is_ok() {
            migrated += 1;
        }
    }
    Ok(migrated)
}

// ------------------------------------------------------------------- profiles

/// Root data directory shared by the app, the CLI and the updater.
pub fn app_dir() -> PathBuf {
    crate::settings::data_dir()
}

pub fn profiles_dir() -> PathBuf {
    app_dir().join("profiles")
}

pub fn registry_path() -> PathBuf {
    app_dir().join("profiles.json")
}

pub fn profile_dir(id: &str) -> PathBuf {
    profiles_dir().join(id)
}

pub fn profile_vault_dir(id: &str) -> PathBuf {
    profile_dir(id).join("vault")
}

pub fn profile_db_path(id: &str) -> PathBuf {
    profile_dir(id).join("notecraft.db")
}

fn load_registry() -> Registry {
    match std::fs::read_to_string(registry_path()) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Registry::default(),
    }
}

fn save_registry(registry: &Registry) -> AppResult<()> {
    std::fs::create_dir_all(app_dir()).map_err(AppError::Io)?;
    let tmp = app_dir().join("profiles.json.tmp");
    let text = serde_json::to_string_pretty(registry).map_err(AppError::Json)?;
    std::fs::write(&tmp, text).map_err(AppError::Io)?;
    std::fs::rename(&tmp, registry_path()).map_err(AppError::Io)?;
    Ok(())
}

fn summary(record: &ProfileRecord) -> ProfileSummary {
    ProfileSummary {
        id: record.id.clone(),
        username: record.username.clone(),
        created_at: record.created_at.clone(),
        vault_path: profile_vault_dir(&record.id).to_string_lossy().to_string(),
    }
}

pub fn list_profiles() -> Vec<ProfileSummary> {
    load_registry().profiles.iter().map(summary).collect()
}

pub fn find_profile(username: &str) -> Option<ProfileSummary> {
    let wanted = normalize_username(username);
    load_registry()
        .profiles
        .iter()
        .find(|record| normalize_username(&record.username) == wanted)
        .map(summary)
}

pub fn normalize_username(username: &str) -> String {
    username.trim().to_lowercase()
}

fn validate_new_profile(username: &str, password: &str) -> AppResult<String> {
    let name = username.trim().to_string();
    if name.len() < 3 || name.len() > 32 {
        return Err(AppError::Invalid(
            "username must be between 3 and 32 characters".into(),
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ' ')
    {
        return Err(AppError::Invalid(
            "username may only use letters, numbers, spaces, dots, dashes and underscores".into(),
        ));
    }
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(AppError::Invalid(format!(
            "password must be at least {MIN_PASSWORD_LEN} characters"
        )));
    }
    Ok(name)
}

/// Create a profile and its encrypted vault. Does not unlock the session.
pub fn create_profile(username: &str, password: &str) -> AppResult<ProfileSummary> {
    let name = validate_new_profile(username, password)?;
    let mut registry = load_registry();
    if registry
        .profiles
        .iter()
        .any(|record| normalize_username(&record.username) == normalize_username(&name))
    {
        return Err(AppError::Conflict(format!(
            "a profile named \"{name}\" already exists"
        )));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let mut salt_bytes = [0u8; SALT_LEN];
    {
        use chacha20poly1305::aead::rand_core::RngCore;
        OsRng.fill_bytes(&mut salt_bytes);
    }
    let key = kdf(password, &salt_bytes)?;
    let verifier = B64.encode(seal(&key, format!("notecraft-profile:{id}").as_bytes())?);

    let vault = profile_vault_dir(&id);
    std::fs::create_dir_all(&vault).map_err(AppError::Io)?;
    std::fs::create_dir_all(profile_dir(&id).join("index")).map_err(AppError::Io)?;

    let record = ProfileRecord {
        id: id.clone(),
        username: name,
        created_at: chrono::Utc::now().to_rfc3339(),
        salt: B64.encode(salt_bytes),
        verifier,
    };
    registry.profiles.push(record.clone());
    save_registry(&registry)?;

    // Seed a tutorial note that is written through the same encrypted path.
    let previous = session_key();
    set_session_key(Some(key));
    let seeded = crate::templates::tutorial_note();
    let _ = write_note_file(&vault.join("Tutorial.md"), seeded.as_bytes());
    set_session_key(previous);

    Ok(summary(&record))
}

/// Verify a password and install the session key.
pub fn unlock(username: &str, password: &str) -> AppResult<ProfileSummary> {
    let registry = load_registry();
    let wanted = normalize_username(username);
    let record = registry
        .profiles
        .iter()
        .find(|record| normalize_username(&record.username) == wanted)
        .ok_or_else(|| AppError::NotFound(format!("no profile named \"{username}\"")))?;

    let salt = B64
        .decode(record.salt.as_bytes())
        .map_err(|_| AppError::Other("profile salt is corrupted".into()))?;
    let verifier = B64
        .decode(record.verifier.as_bytes())
        .map_err(|_| AppError::Other("profile verifier is corrupted".into()))?;

    let key = kdf(password, &salt)?;
    let expected = format!("notecraft-profile:{}", record.id);
    match open(&key, &verifier) {
        Ok(plain) if plain == expected.as_bytes() => {
            set_session_key(Some(key));
            set_active_profile(Some(record.id.clone()));
            std::fs::create_dir_all(profile_dir(&record.id).join("index")).ok();
            Ok(summary(record))
        }
        _ => Err(AppError::Permission("wrong password".into())),
    }
}

/// Drop the session key. Note reads fail until the next unlock.
pub fn lock() {
    set_session_key(None);
    set_active_profile(None);
}

/// Delete a profile after verifying the password.
pub fn delete_profile(username: &str, password: &str) -> AppResult<()> {
    let summary = unlock(username, password)?;
    lock();
    let mut registry = load_registry();
    registry.profiles.retain(|record| record.id != summary.id);
    save_registry(&registry)?;
    let dir = profile_dir(&summary.id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(AppError::Io)?;
    }
    Ok(())
}

/// Change a profile password and re-encrypt the vault with the new key.
pub fn change_password(username: &str, current: &str, next: &str) -> AppResult<()> {
    if next.chars().count() < MIN_PASSWORD_LEN {
        return Err(AppError::Invalid(format!(
            "password must be at least {MIN_PASSWORD_LEN} characters"
        )));
    }
    let summary = unlock(username, current)?;

    let mut registry = load_registry();
    let index = registry
        .profiles
        .iter()
        .position(|record| record.id == summary.id)
        .ok_or_else(|| AppError::NotFound("profile disappeared".into()))?;

    let mut salt_bytes = [0u8; SALT_LEN];
    {
        use chacha20poly1305::aead::rand_core::RngCore;
        OsRng.fill_bytes(&mut salt_bytes);
    }
    let new_key = kdf(next, &salt_bytes)?;
    let old_key = session_key().ok_or_else(|| AppError::Permission("profile is locked".into()))?;

    // Re-seal every note from plaintext under the new key before saving the
    // new salt, so an interrupted change still opens with the old password.
    let vault = profile_vault_dir(&summary.id);
    let mut resealed: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    for path in crate::vault::walk_markdown(&vault, &[]) {
        let Ok(blob) = std::fs::read(&path) else {
            continue;
        };
        let plaintext = if is_container(&blob) {
            open(&old_key, &blob)?
        } else {
            blob
        };
        resealed.push((path, seal(&new_key, &plaintext)?));
    }
    for (path, sealed) in resealed {
        let tmp = path.with_extension("notecraft-tmp");
        std::fs::write(&tmp, sealed).map_err(AppError::Io)?;
        std::fs::rename(&tmp, &path).map_err(AppError::Io)?;
    }

    registry.profiles[index].salt = B64.encode(salt_bytes);
    registry.profiles[index].verifier = B64.encode(seal(
        &new_key,
        format!("notecraft-profile:{}", summary.id).as_bytes(),
    )?);
    save_registry(&registry)?;
    set_session_key(Some(new_key));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_roundtrip() {
        let key = [7u8; KEY_LEN];
        let sealed = seal(&key, b"# hello").unwrap();
        assert!(is_container(&sealed));
        assert_eq!(open(&key, &sealed).unwrap(), b"# hello");
    }

    #[test]
    fn wrong_key_is_rejected() {
        let sealed = seal(&[1u8; KEY_LEN], b"secret").unwrap();
        assert!(open(&[2u8; KEY_LEN], &sealed).is_err());
    }

    #[test]
    fn tampered_ciphertext_is_rejected() {
        let key = [3u8; KEY_LEN];
        let mut sealed = seal(&key, b"secret").unwrap();
        let last = sealed.len() - 1;
        sealed[last] ^= 0xFF;
        assert!(open(&key, &sealed).is_err());
    }

    #[test]
    fn plaintext_is_always_readable() {
        assert!(!is_container(b"# plain markdown"));
        assert!(can_read(b"# plain markdown"));
    }

    #[test]
    fn usernames_are_case_insensitive() {
        assert_eq!(normalize_username(" Ada "), "ada");
    }

    #[test]
    fn short_passwords_are_rejected() {
        assert!(validate_new_profile("ada", "short").is_err());
        assert!(validate_new_profile("ada", "long-enough").is_ok());
    }
}
