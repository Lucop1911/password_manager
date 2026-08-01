use crate::helpers::db::paths::keyfile_path;
use crate::helpers::utils::{decrypt_bytes, derive_key, encrypt_bytes, random_bytes};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Plaintext sidecar file holding everything needed to unlock the SQLCipher
/// database: a random salt, the database key encrypted with a key derived
/// from the master password (envelope encryption) and the theme preference.
///
/// Existence of the keyfile means an account exists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyfile {
    pub salt: String,
    pub db_key_enc: String,
    pub db_key_nonce: String,
    pub dark_mode: bool,
}

impl Keyfile {
    pub fn create(master_password: &str, dark_mode: bool) -> Result<Self, String> {
        let salt = random_bytes::<16>();
        let db_key = random_bytes::<32>();
        let salt_b64 = base64::engine::general_purpose::STANDARD.encode(salt);
        let kek = derive_key(master_password, &salt_b64);
        let (db_key_enc, db_key_nonce) = encrypt_bytes(&kek, &db_key)?;

        Ok(Keyfile {
            salt: salt_b64,
            db_key_enc,
            db_key_nonce,
            dark_mode,
        })
    }

    /// Derives the KEK from the master password and decrypts the database key.
    /// Fails (AES-GCM authentication) when the master password is wrong.
    pub fn db_key(&self, master_password: &str) -> Result<[u8; 32], String> {
        let kek = derive_key(master_password, &self.salt);
        let bytes = decrypt_bytes(&kek, &self.db_key_enc, &self.db_key_nonce)?;
        bytes
            .try_into()
            .map_err(|_| "Invalid database key".to_string())
    }

    pub fn load() -> Result<Option<Self>, String> {
        Self::load_from(&keyfile_path())
    }

    pub fn load_from(path: &Path) -> Result<Option<Self>, String> {
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(path).map_err(|e| format!("Failed to read the keyfile: {}", e))?;
        serde_json::from_str(&raw)
            .map(Some)
            .map_err(|e| format!("Keyfile non valido: {}", e))
    }

    pub fn save(&self) -> Result<(), String> {
        self.save_to(&keyfile_path())
    }

    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize the keyfile: {}", e))?;
        fs::write(path, json).map_err(|e| format!("Failed to save the keyfile: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("pm_keyfile_{}.json", nanos))
    }

    #[test]
    fn roundtrip_and_wrong_password() {
        let path = temp_path();
        let keyfile = Keyfile::create("master-pass", true).unwrap();
        let key = keyfile.db_key("master-pass").unwrap();
        assert_eq!(key.len(), 32);
        assert!(keyfile.db_key("wrong-pass").is_err());

        keyfile.save_to(&path).unwrap();
        let loaded = Keyfile::load_from(&path).unwrap().unwrap();
        assert_eq!(loaded.db_key("master-pass").unwrap(), key);
        assert!(loaded.dark_mode);

        std::fs::remove_file(&path).unwrap();
    }
}
