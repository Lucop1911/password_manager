use crate::helpers::db::database::Database;
use crate::helpers::db::keyfile::Keyfile;
use crate::helpers::db::migration;
use crate::helpers::db::paths::{db_path, legacy_backup_path, legacy_path};
use crate::helpers::utils::{
    PasswordEntry, UserData, derive_key, encrypt_password, generate_salt, hash_password_argon2,
    verify_password,
};
use crate::{AppState, PasswordManagerApp};
use eframe::egui;
use std::fs;

/// Result of a background login attempt. Pure data, applied to the app
/// state by the UI thread when the background thread finishes.
pub struct LoginResult {
    pub db: Database,
    pub user: UserData,
    pub entries: Vec<PasswordEntry>,
    pub key: [u8; 32],
    pub dark_mode: bool,
    pub warning: Option<String>,
}

impl PasswordManagerApp {
    pub fn toggle_theme(&mut self) {
        self.dark_mode = !self.dark_mode;
        // Persist the preference in the keyfile (readable before login)
        if let Ok(Some(mut keyfile)) = Keyfile::load() {
            keyfile.dark_mode = self.dark_mode;
            let _ = keyfile.save();
        }
    }

    pub fn handle_registration(&mut self) {
        if self.reg_username.is_empty() || self.reg_password.is_empty() {
            self.message = "Username and password are required!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }

        if self.reg_password != self.reg_confirm_password {
            self.message = "Passwords do not match!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }

        if self.reg_password.len() < 6 {
            self.message = "Password must be at least 6 characters!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }

        // Generate salts for the password verification hash and for key derivation
        let salt = generate_salt();
        let key_salt = generate_salt();
        let p_h = hash_password_argon2(&self.reg_password, &salt);

        // Order matters: the keyfile is written only after the database is
        // fully populated, so a crash mid-registration leaves no half-state
        // that would be mistaken for a valid account.
        let keyfile = match Keyfile::create(&self.reg_password, self.dark_mode) {
            Ok(keyfile) => keyfile,
            Err(e) => {
                self.message = format!("Failed to create keyfile: {}", e);
                self.message_color = egui::Color32::RED;
                return;
            }
        };
        let db_key = match keyfile.db_key(&self.reg_password) {
            Ok(key) => key,
            Err(e) => {
                self.message = format!("Failed to derive key: {}", e);
                self.message_color = egui::Color32::RED;
                return;
            }
        };
        let db = match Database::open_fresh(&db_path(), &db_key) {
            Ok(db) => db,
            Err(e) => {
                self.message = format!("Failed to create database: {}", e);
                self.message_color = egui::Color32::RED;
                return;
            }
        };

        let user_data = UserData {
            u: self.reg_username.clone(),
            p_h,
            salt,
            key_salt,
        };

        if let Err(e) = db.insert_user(&user_data) {
            self.message = e;
            self.message_color = egui::Color32::RED;
            return;
        }
        if let Err(e) = keyfile.save() {
            self.message = e;
            self.message_color = egui::Color32::RED;
            return;
        }

        // Remove any leftovers from the old JSON format
        if legacy_path().exists() {
            let _ = fs::remove_file(legacy_path());
        }

        // The encryption key is held in memory for the session only and is
        // never persisted; it must be re-derived from the master password
        // on every login.
        self.encryption_key = Some(derive_key(&self.reg_password, &user_data.key_salt));
        self.current_user = Some(user_data);
        self.db = Some(db);
        self.ps.clear();

        self.message = "Registration completed successfully!".to_string();
        self.message_color = egui::Color32::GREEN;
        self.state = AppState::Main;

        // Clear the form fields
        self.reg_username.clear();
        self.reg_password.clear();
        self.reg_confirm_password.clear();
    }

    pub fn handle_migration(&mut self) {
        if self.migration_password.is_empty() {
            self.message = "Enter the master password!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }

        match migration::migrate(&self.migration_password) {
            Ok((db, user, entries, corrupt_entries)) => {
                self.ps = entries;
                self.current_user = Some(user.clone());
                self.encryption_key = Some(derive_key(&self.migration_password, &user.key_salt));
                self.db = Some(db);

                if let Ok(Some(keyfile)) = Keyfile::load() {
                    self.dark_mode = keyfile.dark_mode;
                }

                if corrupt_entries.is_empty() {
                    self.message = "Data migrated successfully!".to_string();
                    self.message_color = egui::Color32::GREEN;
                } else {
                    self.message = format!(
                        "Data migrated, but {} entry could not be decrypted ({}). It may have been corrupted in the old file.",
                        corrupt_entries.len(),
                        corrupt_entries.join(", ")
                    );
                    self.message_color = egui::Color32::YELLOW;
                }
                self.state = AppState::Main;

                self.migration_password.clear();
            }
            Err(e) => {
                self.message = format!("Migration failed: {}", e);
                self.message_color = egui::Color32::RED;
            }
        }
    }
}

/// Unlocks the keyfile with the master password, then the SQLCipher
/// database, and finally verifies the credentials. A wrong password
/// fails at the keyfile step (AES-GCM authentication), keeping the
/// database untouched.
pub fn perform_login(
    username: &str,
    password: &str,
) -> Result<LoginResult, String> {
    let keyfile = Keyfile::load()?.ok_or_else(|| "No account found!".to_string())?;

    // Derive the database key from the master password
    let db_key = keyfile
        .db_key(password)
        .map_err(|_| "Incorrect username or password!".to_string())?;

    let db = Database::open(&db_path(), &db_key)
        .map_err(|_| "Could not open the database!".to_string())?;

    let user = db
        .get_user()
        .map_err(|e| format!("Could not read the database: {}", e))?
        .ok_or_else(|| "Incorrect username or password!".to_string())?;

    let p_h = verify_password(password, &user.salt, &user.p_h);
    if username != user.u || !p_h {
        return Err("Incorrect username or password!".to_string());
    }

    // Derive the entry encryption key from the master password
    let entries = db
        .list_passwords()
        .map_err(|e| format!("Could not read the database: {}", e))?;
    let entry_key = derive_key(password, &user.key_salt);

    // One-time check: while the pre-migration backup exists, verify the
    // database against it. Once it matches, the backup has served its
    // purpose and is removed automatically. On mismatch it is kept as the
    // only remaining copy of the legacy data.
    let mut warning: Option<String> = None;
    if migration::backup_exists() {
        match migration::verify_backup(&db, &legacy_backup_path(), Some(&entry_key)) {
            Ok(verification) => {
                if let Some(mismatch) = verification.mismatch {
                    warning = Some(format!(
                        "Signed in, but the data backup does not match the database: {}",
                        mismatch
                    ));
                } else {
                    if !verification.corrupt_entries.is_empty() {
                        warning = Some(format!(
                            "Signed in. {} old entry could not be decrypted: {}",
                            verification.corrupt_entries.len(),
                            verification.corrupt_entries.join(", ")
                        ));
                    }
                    let _ = migration::remove_backup();
                }
            }
            Err(_) => {
                // Unreadable backup: leave it in place
            }
        }
    }

    Ok(LoginResult {
        db,
        user,
        entries,
        key: entry_key,
        dark_mode: keyfile.dark_mode,
        warning,
    })
}

impl PasswordManagerApp {
    pub fn add_password(&mut self) {
        if self.new_entry_name.is_empty() || self.new_entry_password.is_empty() {
            self.message = "Service name and password are required!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }

        if let Some(encryption_key) = &self.encryption_key {
            match encrypt_password(&self.new_entry_password, encryption_key) {
                Ok((e_c, nonce)) => {
                    let entry = PasswordEntry {
                        id: 0,
                        name: self.new_entry_name.clone(),
                        u: self.new_entry_username.clone(),
                        e_c,
                        nonce,
                    };

                    let db = match &self.db {
                        Some(db) => db,
                        None => {
                            self.message = "Database unavailable!".to_string();
                            self.message_color = egui::Color32::RED;
                            return;
                        }
                    };

                    match db.insert_password(&entry) {
                        Ok(id) => {
                            let mut entry = entry;
                            entry.id = id;
                            self.ps.push(entry);

                            self.message = "Password added successfully!".to_string();
                            self.message_color = egui::Color32::GREEN;

                            // Clear the form fields
                            self.new_entry_name.clear();
                            self.new_entry_username.clear();
                            self.new_entry_password.clear();
                        }
                        Err(e) => {
                            self.message = e;
                            self.message_color = egui::Color32::RED;
                        }
                    }
                }
                Err(_) => {
                    self.message = "Failed to encrypt the password!".to_string();
                    self.message_color = egui::Color32::RED;
                }
            }
        } else {
            self.message = "Encryption key unavailable!".to_string();
            self.message_color = egui::Color32::RED;
        }
    }

    pub fn edit_password(&mut self) {
        if self.edit_service_name.is_empty() || self.edit_new_password.is_empty() {
            self.message = "Service name and new password are required!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }

        if self.edit_new_password != self.edit_confirm_password {
            self.message = "Passwords do not match!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }

        // Find the entry to edit by service name (case-insensitive)
        let entry_index = self
            .ps
            .iter()
            .position(|entry| entry.name.to_lowercase() == self.edit_service_name.to_lowercase());

        match entry_index {
            Some(index) => {
                if let Some(encryption_key) = &self.encryption_key {
                    match encrypt_password(&self.edit_new_password, encryption_key) {
                        Ok((e_c, nonce)) => {
                            // Update the existing entry in place
                            let entry = &mut self.ps[index];
                            entry.e_c = e_c;
                            entry.nonce = nonce;

                            // Update the username only if one was provided
                            if !self.edit_new_username.is_empty() {
                                entry.u = self.edit_new_username.clone();
                            }

                            // Drop the revealed password, if any
                            self.shown_passwords.remove(&entry.id);

                            if let Some(db) = &self.db {
                                if let Err(e) = db.update_password(entry) {
                                    self.message = e;
                                    self.message_color = egui::Color32::RED;
                                    return;
                                }
                            }

                            self.message = format!(
                                "Password for '{}' updated successfully!",
                                self.edit_service_name
                            );
                            self.message_color = egui::Color32::GREEN;

                            // Clear the form fields
                            self.edit_service_name.clear();
                            self.edit_new_username.clear();
                            self.edit_new_password.clear();
                            self.edit_confirm_password.clear();
                        }
                        Err(_) => {
                            self.message = "Failed to encrypt the password!".to_string();
                            self.message_color = egui::Color32::RED;
                        }
                    }
                } else {
                    self.message = "Encryption key unavailable!".to_string();
                    self.message_color = egui::Color32::RED;
                }
            }
            None => {
                self.message = format!("Service '{}' not found!", self.edit_service_name);
                self.message_color = egui::Color32::RED;
            }
        }
    }

    pub fn remove_password(&mut self, id: i64) {
        let Some(index) = self.ps.iter().position(|entry| entry.id == id) else {
            return;
        };

        let removed_entry = self.ps.remove(index);
        self.shown_passwords.remove(&id);

        if let Some(db) = &self.db {
            let _ = db.delete_password(id);
        }

        self.message = format!("Password for '{}' deleted!", removed_entry.name);
        self.message_color = egui::Color32::RED;
    }

    pub fn logout(&mut self) {
        // Drop the DB connection and purge session secrets from memory
        self.db = None;
        self.ps.clear();
        self.current_user = None;
        self.encryption_key = None;
        self.shown_passwords.clear();
        self.state = AppState::Login;
        self.message = "Logged out successfully.".to_string();
        self.message_color = egui::Color32::CYAN;
    }
}
