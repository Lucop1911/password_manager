use aes_gcm::{
    Aes256Gcm, Key,
    aead::{Aead, Generate, KeyInit, Nonce},
};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use base64::Engine;
use rand::Rng;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct UserData {
    pub u: String,   // Username
    pub p_h: String, // Password verification hash (see `verify_password`)
    pub salt: String, // Salt for the verification hash
    pub key_salt: String, // Salt for deriving the AES encryption key
}

#[derive(Debug, Clone)]
pub struct PasswordEntry {
    pub id: i64,
    pub name: String,
    pub u: String,   // Username
    pub e_c: String, // Password crypt (base64 AES-256-GCM ciphertext)
    pub nonce: String,
}

/// Cryptographic random bytes from the OS CSPRNG.
pub fn random_bytes<const N: usize>() -> [u8; N] {
    rand::rng().random()
}

pub fn generate_salt() -> String {
    let salt: [u8; 16] = random_bytes();
    base64::engine::general_purpose::STANDARD.encode(salt)
}

/// Legacy password verification hash (SHA-256). Kept for accounts migrated
/// from the old `data.json` format. Do not use for new accounts.
pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::default();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let result = hasher.finalize();
    base64::engine::general_purpose::STANDARD.encode(result)
}

/// Memory-hard (Argon2id) password verification hash for new accounts.
const ARGON2_HASH_PREFIX: &str = "a2:";

pub fn hash_password_argon2(password: &str, salt: &str) -> String {
    let key = derive_key(password, salt);
    format!(
        "{}{}",
        ARGON2_HASH_PREFIX,
        base64::engine::general_purpose::STANDARD.encode(key)
    )
}

/// Verifies a password against a stored hash, supporting both the legacy
/// SHA-256 scheme (unprefixed) and the Argon2id scheme ("a2:" prefix).
pub fn verify_password(password: &str, salt: &str, stored_hash: &str) -> bool {
    if let Some(encoded) = stored_hash.strip_prefix(ARGON2_HASH_PREFIX) {
        let expected = derive_key(password, salt);
        base64::engine::general_purpose::STANDARD.encode(expected) == encoded
    } else {
        hash_password(password, salt) == stored_hash
    }
}

/// Derives a 32-byte key from the master password using Argon2id.
/// Used both for entry encryption and to protect the database keyfile.
pub fn derive_key(password: &str, salt: &str) -> [u8; 32] {
    let argon2 = Argon2::default();
    let salt_bytes = base64::engine::general_purpose::STANDARD
        .decode(salt)
        .unwrap_or_else(|_| salt.as_bytes().to_vec());

    let mut fixed_salt = [0u8; 16];
    if salt_bytes.len() >= 16 {
        fixed_salt.copy_from_slice(&salt_bytes[..16]);
    } else {
        fixed_salt[..salt_bytes.len()].copy_from_slice(&salt_bytes);
    }

    let salt_string = SaltString::encode_b64(&fixed_salt).unwrap();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .unwrap();
    let hash = password_hash.hash.unwrap();

    let mut key = [0u8; 32];
    let hash_bytes = hash.as_bytes();
    if hash_bytes.len() >= 32 {
        key.copy_from_slice(&hash_bytes[..32]);
    } else {
        key[..hash_bytes.len()].copy_from_slice(hash_bytes);
    }
    key
}

/// Authenticated encryption (AES-256-GCM): returns (ciphertext, nonce) as
/// base64. Each call uses a fresh random nonce, so repeated plaintexts
/// produce different ciphertexts.
pub fn encrypt_bytes(key_bytes: &[u8; 32], data: &[u8]) -> Result<(String, String), String> {
    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from(*key_bytes));
    let nonce = Nonce::<Aes256Gcm>::generate();
    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| format!("Encryption error: {:?}", e))?;

    let encrypted = base64::engine::general_purpose::STANDARD.encode(ciphertext);
    let nonce_b64 = base64::engine::general_purpose::STANDARD.encode(nonce);
    Ok((encrypted, nonce_b64))
}

pub fn decrypt_bytes(
    key_bytes: &[u8; 32],
    encrypted_b64: &str,
    nonce_b64: &str,
) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from(*key_bytes));

    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(encrypted_b64)
        .map_err(|e| format!("Base64 decode error: {:?}", e))?;
    let nonce_bytes = base64::engine::general_purpose::STANDARD
        .decode(nonce_b64)
        .map_err(|e| format!("Nonce decode error: {:?}", e))?;
    let nonce = Nonce::<Aes256Gcm>::try_from(nonce_bytes.as_slice())
        .map_err(|e| format!("Nonce decode error: {:?}", e))?;

    cipher
        .decrypt(&nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption error: {:?}", e))
}

pub fn encrypt_password(password: &str, key_bytes: &[u8; 32]) -> Result<(String, String), String> {
    encrypt_bytes(key_bytes, password.as_bytes())
}

pub fn decrypt_password(entry: &PasswordEntry, key_bytes: &[u8; 32]) -> Result<String, String> {
    let plaintext = decrypt_bytes(key_bytes, &entry.e_c, &entry.nonce)?;
    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 conversion error: {:?}", e))
}

pub fn confirm_notification() -> bool {
    let result = MessageDialog::new()
        .set_level(MessageLevel::Warning)
        .set_title("Confirm Deletion")
        .set_description("Are you sure you want to delete this password? This action cannot be undone.")
        .set_buttons(MessageButtons::YesNo)
        .show();

    result == MessageDialogResult::Yes
}
