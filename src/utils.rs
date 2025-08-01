use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{PathBuf};
use base64::Engine;
use rand::Rng;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key
};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use rfd::{MessageDialog, MessageButtons, MessageLevel, MessageDialogResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub u: String,
    pub p_h: String,
    pub salt: String,
    pub key_salt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntry {
    pub name: String,
    pub u: String,
    pub e_c: String,
    pub nonce: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppData {
    pub user: Option<UserData>,
    pub ps: Vec<PasswordEntry>,
    pub dark_mode: Option<bool>,
}

fn get_data_file_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Unable to find home directory");
    let app_dir = home_dir.join("p_manager");
    
   if !app_dir.exists() {
        if let Err(_e) = fs::create_dir_all(&app_dir) {
            return PathBuf::from("data.json");
        }
    }
    
    app_dir.join("data.json")
}

pub fn generate_salt() -> String {
    let mut rng = rand::rng();
    let salt: [u8; 16] = rng.random();
    base64::engine::general_purpose::STANDARD.encode(salt)
}

pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::default();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let result = hasher.finalize();
    base64::engine::general_purpose::STANDARD.encode(result)
}

pub fn derive_key(password: &str, salt: &str) -> [u8; 32] {
    let argon2 = Argon2::default();
    let salt_bytes = base64::engine::general_purpose::STANDARD.decode(salt)
        .unwrap_or_else(|_| salt.as_bytes().to_vec());
    
    let mut fixed_salt = [0u8; 16];
    if salt_bytes.len() >= 16 {
        fixed_salt.copy_from_slice(&salt_bytes[..16]);
    } else {
        fixed_salt[..salt_bytes.len()].copy_from_slice(&salt_bytes);
    }
    
    let salt_string = SaltString::encode_b64(&fixed_salt).unwrap();
    
    let password_hash = argon2.hash_password(password.as_bytes(), &salt_string).unwrap();
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

pub fn encrypt_password(password: &str, key_bytes: &[u8; 32]) -> Result<(String, String), String> {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    
    let nonce_bytes = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce_bytes, password.as_bytes())
        .map_err(|e| format!("Encryption error: {:?}", e))?;
    
    let encrypted_password = base64::engine::general_purpose::STANDARD.encode(ciphertext);
    let nonce = base64::engine::general_purpose::STANDARD.encode(nonce_bytes);
    
    Ok((encrypted_password, nonce))
}

pub fn decrypt_password(entry: &PasswordEntry, key_bytes: &[u8; 32]) -> Result<String, String> {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);
    
    let ciphertext = base64::engine::general_purpose::STANDARD.decode(&entry.e_c)
        .map_err(|e| format!("Base64 decode error: {:?}", e))?;
    let nonce_bytes = base64::engine::general_purpose::STANDARD.decode(&entry.nonce)
        .map_err(|e| format!("Nonce decode error: {:?}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption error: {:?}", e))?;
    String::from_utf8(plaintext)
        .map_err(|e| format!("UTF-8 conversion error: {:?}", e))
}

pub fn load_data() -> AppData {
    let data_file = get_data_file_path();
    
    if data_file.exists() {
        let data = fs::read_to_string(&data_file).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_else(|_| AppData {
            user: None,
            ps: Vec::new(),
            dark_mode: Some(true),
        })
    } else {
        AppData {
            user: None,
            ps: Vec::new(),
            dark_mode: Some(true),
        }
    }
}

pub fn save_data(data: &AppData) {
    let data_file = get_data_file_path();
    
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = fs::write(data_file, json);
    }
}

pub fn notifica_conferma() -> bool {
    let result = MessageDialog::new()
        .set_level(MessageLevel::Warning)
        .set_title("Conferma Eliminazione")
        .set_description("Sei sicuro di voler eliminare questa password? Questa azione non può essere annullata.")
        .set_buttons(MessageButtons::YesNo)
        .show();
    
    result == MessageDialogResult::Yes
}