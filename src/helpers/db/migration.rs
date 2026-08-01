use crate::helpers::db::database::Database;
use crate::helpers::db::keyfile::Keyfile;
use crate::helpers::db::paths::{db_path, keyfile_path, legacy_backup_path, legacy_path};
use crate::helpers::utils::{
    PasswordEntry, UserData, decrypt_password, derive_key, hash_password,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyUser {
    u: String,
    p_h: String,
    salt: String,
    key_salt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyEntry {
    name: String,
    u: String,
    e_c: String,
    nonce: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LegacyData {
    user: Option<LegacyUser>,
    ps: Vec<LegacyEntry>,
    dark_mode: Option<bool>,
}

fn load_legacy(path: &Path) -> Result<LegacyData, String> {
    let raw =
        fs::read_to_string(path).map_err(|e| format!("Failed to read data.json: {}", e))?;
    serde_json::from_str(&raw).map_err(|e| format!("Invalid data.json file: {}", e))
}

pub fn legacy_exists() -> bool {
    legacy_path().exists()
}

pub fn legacy_username() -> Option<String> {
    load_legacy(&legacy_path()).ok()?.user.map(|user| user.u)
}

pub fn legacy_has_account() -> bool {
    match load_legacy(&legacy_path()) {
        Ok(data) => data.user.is_some(),
        Err(_) => false,
    }
}

pub fn remove_legacy() -> Result<(), String> {
    if legacy_path().exists() {
        fs::remove_file(legacy_path())
            .map_err(|e| format!("Failed to remove data.json: {}", e))?;
    }
    Ok(())
}

pub fn backup_exists() -> bool {
    legacy_backup_path().exists()
}

pub fn remove_backup() -> Result<(), String> {
    let path = legacy_backup_path();
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Failed to remove backup file: {}", e))?;
    }
    Ok(())
}

/// Result of comparing the database against the pre-migration backup.
/// `mismatch` indicates that the backup contains data the database does not;
/// `corrupt_entries` lists legacy entries that fail to decrypt with the
/// current entry key (they were already corrupted before the migration).
pub struct BackupVerification {
    pub mismatch: Option<String>,
    pub corrupt_entries: Vec<String>,
}

/// Verifies that every entry (and the user row) stored in the backup file is
/// present in the database. Entries added after the migration are allowed.
/// Since migration copies the AES-GCM ciphertexts verbatim, matching
/// ciphertext means matching plaintext once the same key is applied.
pub fn verify_backup(
    db: &Database,
    backup_path: &Path,
    entry_key: Option<&[u8; 32]>,
) -> Result<BackupVerification, String> {
    let legacy = load_legacy(backup_path)?;
    let mut verification = BackupVerification {
        mismatch: None,
        corrupt_entries: Vec::new(),
    };

    // The user row must match exactly.
    let db_user = db.get_user()?;
    let users_match = match (&db_user, &legacy.user) {
        (Some(u1), Some(u2)) => {
            u1.u == u2.u && u1.p_h == u2.p_h && u1.salt == u2.salt && u1.key_salt == u2.key_salt
        }
        (None, None) => true,
        _ => false,
    };
    if !users_match {
        verification.mismatch =
            Some("the stored user account does not match the backup".to_string());
        return Ok(verification);
    }

    // Every backup entry must exist in the database (count-aware subset).
    let db_entries = db.list_passwords()?;
    let mut available: HashMap<(String, String, String, String), usize> = HashMap::new();
    for entry in &db_entries {
        *available
            .entry((
                entry.name.clone(),
                entry.u.clone(),
                entry.e_c.clone(),
                entry.nonce.clone(),
            ))
            .or_insert(0) += 1;
    }

    let mut missing: Vec<String> = Vec::new();
    for legacy_entry in &legacy.ps {
        let key = (
            legacy_entry.name.clone(),
            legacy_entry.u.clone(),
            legacy_entry.e_c.clone(),
            legacy_entry.nonce.clone(),
        );
        match available.get_mut(&key) {
            Some(count) if *count > 0 => *count -= 1,
            _ => missing.push(legacy_entry.name.clone()),
        }
    }
    if !missing.is_empty() {
        verification.mismatch = Some(format!(
            "{} entries missing from the database: {}",
            missing.len(),
            missing.join(", ")
        ));
        return Ok(verification);
    }

    // Decrypt smoke test: flag legacy entries that were already corrupted.
    if let Some(key) = entry_key {
        for legacy_entry in &legacy.ps {
            let check_entry = PasswordEntry {
                id: 0,
                name: legacy_entry.name.clone(),
                u: legacy_entry.u.clone(),
                e_c: legacy_entry.e_c.clone(),
                nonce: legacy_entry.nonce.clone(),
            };
            if decrypt_password(&check_entry, key).is_err() {
                verification.corrupt_entries.push(legacy_entry.name.clone());
            }
        }
    }

    Ok(verification)
}

/// Migrates the legacy `data.json` into a new SQLCipher database.
/// Verifies the master password against the legacy account, creates the
/// keyfile, populates the database (entries are copied as-is: they are already
/// AES-GCM encrypted), verifies the database against the source file and only
/// then archives `data.json` as a backup. Returns the opened database, the
/// migrated user and entries, plus the names of any legacy entries that fail
/// to decrypt.
pub fn migrate(
    master_password: &str,
) -> Result<(Database, UserData, Vec<PasswordEntry>, Vec<String>), String> {
    migrate_with_paths(
        master_password,
        &legacy_path(),
        &db_path(),
        &keyfile_path(),
        &legacy_backup_path(),
    )
}

fn migrate_with_paths(
    master_password: &str,
    legacy_path: &Path,
    db_path: &Path,
    keyfile_path: &Path,
    backup_path: &Path,
) -> Result<(Database, UserData, Vec<PasswordEntry>, Vec<String>), String> {
    let legacy = load_legacy(legacy_path)?;
    let legacy_user = legacy
        .user
        .ok_or_else(|| "No account found in data.json".to_string())?;

    let p_h = hash_password(master_password, &legacy_user.salt);
    if p_h != legacy_user.p_h {
        return Err("Incorrect master password".to_string());
    }

    let keyfile = Keyfile::create(master_password, legacy.dark_mode.unwrap_or(true))?;
    let db_key = keyfile.db_key(master_password)?;

    let db = Database::open_fresh(db_path, &db_key)?;

    let user = UserData {
        u: legacy_user.u,
        p_h: legacy_user.p_h,
        salt: legacy_user.salt,
        key_salt: legacy_user.key_salt,
    };
    db.insert_user(&user)?;

    let mut entries = Vec::new();
    for legacy_entry in legacy.ps {
        let mut entry = PasswordEntry {
            id: 0,
            name: legacy_entry.name,
            u: legacy_entry.u,
            e_c: legacy_entry.e_c,
            nonce: legacy_entry.nonce,
        };
        entry.id = db.insert_password(&entry)?;
        entries.push(entry);
    }

    // Verify the database against the source file before archiving it:
    // on any mismatch the migration is aborted and data.json is left untouched.
    let entry_key = derive_key(master_password, &user.key_salt);
    let verification = verify_backup(&db, legacy_path, Some(&entry_key))?;
    if let Some(mismatch) = verification.mismatch {
        return Err(format!("Migration verification failed: {}", mismatch));
    }

    keyfile.save_to(keyfile_path)?;

    if backup_path.exists() {
        fs::remove_file(backup_path)
            .map_err(|e| format!("Failed to remove old backup: {}", e))?;
    }
    fs::rename(legacy_path, backup_path)
        .map_err(|e| format!("Failed to archive data.json: {}", e))?;

    Ok((db, user, entries, verification.corrupt_entries))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::utils::{encrypt_password, generate_salt, hash_password};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("pm_migration_{}", nanos));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_legacy(ps: Vec<LegacyEntry>) -> LegacyData {
        let salt = generate_salt();
        LegacyData {
            user: Some(LegacyUser {
                u: "luca".into(),
                p_h: hash_password("master123", &salt),
                salt,
                key_salt: generate_salt(),
            }),
            ps,
            dark_mode: Some(false),
        }
    }

    fn encrypted_entry(name: &str, master: &str, key_salt: &str) -> LegacyEntry {
        let key = derive_key(master, key_salt);
        let (e_c, nonce) = encrypt_password("hunter2", &key).unwrap();
        LegacyEntry {
            name: name.into(),
            u: "a@b.c".into(),
            e_c,
            nonce,
        }
    }

    #[test]
    fn migrates_legacy_data_and_verifies_backup() {
        let dir = temp_dir();
        let legacy_path = dir.join("data.json");
        let db_path = dir.join("data.db");
        let keyfile_path = dir.join("keyfile.json");
        let backup_path = dir.join("data.json.migrated");

        let key_salt = generate_salt();
        let mut legacy_data = sample_legacy(vec![encrypted_entry("Gmail", "master123", &key_salt)]);
        legacy_data.user.as_mut().unwrap().key_salt = key_salt.clone();
        fs::write(&legacy_path, serde_json::to_string(&legacy_data).unwrap()).unwrap();

        let (db, user, entries, corrupt) = migrate_with_paths(
            "master123",
            &legacy_path,
            &db_path,
            &keyfile_path,
            &backup_path,
        )
        .unwrap();
        assert_eq!(user.u, "luca");
        assert_eq!(entries.len(), 1);
        assert!(entries[0].id > 0);
        assert!(corrupt.is_empty(), "no corrupt entries expected");
        assert_eq!(db.list_passwords().unwrap().len(), 1);
        assert!(backup_path.exists());
        assert!(!legacy_path.exists());

        // The archived backup must verify cleanly against the database.
        let key = derive_key("master123", &user.key_salt);
        let verification = verify_backup(&db, &backup_path, Some(&key)).unwrap();
        assert!(verification.mismatch.is_none());
        assert!(verification.corrupt_entries.is_empty());

        let keyfile = Keyfile::load_from(&keyfile_path).unwrap().unwrap();
        assert!(!keyfile.dark_mode);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn verify_passes_ignoring_entries_added_later() {
        let dir = temp_dir();
        let legacy_path = dir.join("data.json");
        let db_path = dir.join("data.db");
        let keyfile_path = dir.join("keyfile.json");
        let backup_path = dir.join("data.json.migrated");

        let key_salt = generate_salt();
        let mut legacy_data = sample_legacy(Vec::new());
        legacy_data.user.as_mut().unwrap().key_salt = key_salt.clone();
        legacy_data.ps = vec![encrypted_entry("Gmail", "master123", &key_salt)];
        fs::write(&legacy_path, serde_json::to_string(&legacy_data).unwrap()).unwrap();

        let (db, _, _, _) = migrate_with_paths(
            "master123",
            &legacy_path,
            &db_path,
            &keyfile_path,
            &backup_path,
        )
        .unwrap();

        // A password added after the migration must not fail verification.
        let extra = PasswordEntry {
            id: 0,
            name: "GitHub".into(),
            u: "luca".into(),
            e_c: "ct".into(),
            nonce: "n".into(),
        };
        db.insert_password(&extra).unwrap();

        let key = derive_key("master123", &key_salt);
        let verification = verify_backup(&db, &backup_path, Some(&key)).unwrap();
        assert!(verification.mismatch.is_none());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn verify_detects_missing_entry() {
        let dir = temp_dir();
        let legacy_path = dir.join("data.json");
        let db_path = dir.join("data.db");
        let keyfile_path = dir.join("keyfile.json");
        let backup_path = dir.join("data.json.migrated");

        let key_salt = generate_salt();
        let mut legacy_data = sample_legacy(Vec::new());
        legacy_data.user.as_mut().unwrap().key_salt = key_salt.clone();
        legacy_data.ps = vec![
            encrypted_entry("Gmail", "master123", &key_salt),
            encrypted_entry("GitHub", "master123", &key_salt),
        ];
        fs::write(&legacy_path, serde_json::to_string(&legacy_data).unwrap()).unwrap();

        let (db, _, entries, _) = migrate_with_paths(
            "master123",
            &legacy_path,
            &db_path,
            &keyfile_path,
            &backup_path,
        )
        .unwrap();

        // Simulate data loss: remove one entry from the database.
        db.delete_password(entries[0].id).unwrap();

        let verification = verify_backup(&db, &backup_path, None).unwrap();
        let mismatch = verification.mismatch.expect("mismatch expected");
        assert!(mismatch.contains("missing"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn verify_detects_modified_entry() {
        let dir = temp_dir();
        let legacy_path = dir.join("data.json");
        let db_path = dir.join("data.db");
        let keyfile_path = dir.join("keyfile.json");
        let backup_path = dir.join("data.json.migrated");

        let key_salt = generate_salt();
        let mut legacy_data = sample_legacy(Vec::new());
        legacy_data.user.as_mut().unwrap().key_salt = key_salt.clone();
        legacy_data.ps = vec![encrypted_entry("Gmail", "master123", &key_salt)];
        fs::write(&legacy_path, serde_json::to_string(&legacy_data).unwrap()).unwrap();

        let (db, _, entries, _) = migrate_with_paths(
            "master123",
            &legacy_path,
            &db_path,
            &keyfile_path,
            &backup_path,
        )
        .unwrap();

        // Tamper with the ciphertext of the stored entry.
        let mut tampered = entries[0].clone();
        tampered.e_c = "TAMPERED".into();
        db.update_password(&tampered).unwrap();

        let verification = verify_backup(&db, &backup_path, None).unwrap();
        assert!(verification.mismatch.is_some());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn verify_flags_corrupt_legacy_entries() {
        let dir = temp_dir();
        let legacy_path = dir.join("data.json");
        let db_path = dir.join("data.db");
        let keyfile_path = dir.join("keyfile.json");
        let backup_path = dir.join("data.json.migrated");

        let key_salt = generate_salt();
        let mut legacy_data = sample_legacy(Vec::new());
        legacy_data.user.as_mut().unwrap().key_salt = key_salt.clone();
        let mut corrupt = encrypted_entry("Gmail", "master123", &key_salt);
        corrupt.e_c = "NOT-BASE64-CIPHERTEXT".into();
        legacy_data.ps = vec![corrupt];
        fs::write(&legacy_path, serde_json::to_string(&legacy_data).unwrap()).unwrap();

        // Migration succeeds but reports the corrupt entry.
        let (db, user, _, corrupt_entries) = migrate_with_paths(
            "master123",
            &legacy_path,
            &db_path,
            &keyfile_path,
            &backup_path,
        )
        .unwrap();
        assert_eq!(corrupt_entries, vec!["Gmail".to_string()]);

        let key = derive_key("master123", &user.key_salt);
        let verification = verify_backup(&db, &backup_path, Some(&key)).unwrap();
        assert!(verification.mismatch.is_none());
        assert_eq!(verification.corrupt_entries, vec!["Gmail".to_string()]);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rejects_wrong_password() {
        let dir = temp_dir();
        let legacy_path = dir.join("data.json");
        let salt = generate_salt();
        let legacy_data = LegacyData {
            user: Some(LegacyUser {
                u: "luca".into(),
                p_h: hash_password("master123", &salt),
                salt,
                key_salt: generate_salt(),
            }),
            ps: Vec::new(),
            dark_mode: None,
        };
        fs::write(&legacy_path, serde_json::to_string(&legacy_data).unwrap()).unwrap();

        let result = migrate_with_paths(
            "wrong",
            &legacy_path,
            &dir.join("data.db"),
            &dir.join("keyfile.json"),
            &dir.join("backup.json"),
        );
        assert!(result.is_err());
        assert!(legacy_path.exists(), "data.json must not be touched on failure");

        fs::remove_dir_all(&dir).unwrap();
    }
}
