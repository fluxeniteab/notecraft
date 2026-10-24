use crate::error::AppError;
use crate::state::AppState;
use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Nonce};
use serde::{Deserialize, Serialize};
use tauri::State;

pub fn set_password(state: &State<AppState>, password: &str) -> AppResult<()> {
    // Derive key via argon2, store salt+verification marker. We store a verifier file, not the password.
    use argon2::{Argon2, PasswordHasher};
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt).map_err(|e| AppError::Other(e.to_string()))?;
    let dir = crate::settings::data_dir().join("security");
    std::fs::create_dir_all(&dir).map_err(AppError::Io)?;
    std::fs::write(dir.join("verifier.txt"), hash.to_string()).map_err(AppError::Io)?;
    std::fs::write(dir.join("salt.txt"), salt.to_string()).map_err(AppError::Io)?;
    let _ = state;
    Ok(())
}

#[allow(dead_code)]
pub fn encrypt_bytes(password: &str, plaintext: &[u8]) -> AppResult<Vec<u8>> {
    use argon2::{Argon2, PasswordHasher};
    use argon2::password_hash::SaltString;
    // deterministic-ish: use a fixed salt file or generate; here generate salt prepended
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let argon2 = Argon2::default();
    let phc = argon2.hash_password(password.as_bytes(), &salt).map_err(|e| AppError::Other(e.to_string()))?;
    // use first 32 bytes of hash output? argon2 hash is PHC string; instead derive via argon2 key derivation
    let mut key = [0u8; 32];
    Argon2::default().hash_password_into(password.as_bytes(), salt.as_str().as_bytes(), &mut key).map_err(|e| AppError::Other(e.to_string()))?;
    let cipher = ChaCha20Poly1305::new_from_slice(&key).map_err(|e| AppError::Other(e.to_string()))?;
    use argon2::password_hash::rand_core::RngCore;
    let mut nonce_bytes = [0u8; 12];
    argon2::password_hash::rand_core::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher.encrypt(nonce, plaintext).map_err(|e| AppError::Other(e.to_string()))?;
    let mut out = salt.as_str().as_bytes().to_vec();
    out.push(b'\n');
    out.extend_from_slice(nonce);
    out.extend_from_slice(&ct);
    let _ = phc;
    Ok(out)
}
