use crate::utils::{generate_salt, hash_password, derive_key, encrypt_password, save_data, UserData, PasswordEntry};
use crate::{PasswordManagerApp, AppState};
use eframe::egui;

impl PasswordManagerApp {
    pub fn toggle_theme(&mut self) {
        self.dark_mode = !self.dark_mode;
        self.app_data.dark_mode = Some(self.dark_mode);
        save_data(&self.app_data);
    }
    
    pub fn handle_registration(&mut self) {
        if self.reg_username.is_empty() || self.reg_password.is_empty() {
            self.message = "Username e password sono obbligatori!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }
        
        if self.reg_password != self.reg_confirm_password {
            self.message = "Le password non coincidono!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }
        
        if self.reg_password.len() < 6 {
            self.message = "La password deve essere di almeno 6 caratteri!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }
        
        // Genera salt per l'hash della password e per la derivazione della chiave
        let salt = generate_salt();
        let key_salt = generate_salt();
        let p_h = hash_password(&self.reg_password, &salt);
        
        // Deriva la chiave di crittografia dalla password
        self.encryption_key = Some(derive_key(&self.reg_password, &key_salt));
        
        let user_data = UserData {
            u: self.reg_username.clone(),
            p_h,
            salt,
            key_salt,
        };
        
        self.app_data.user = Some(user_data.clone());
        self.current_user = Some(user_data);
        
        save_data(&self.app_data);
        
        self.message = "Registrazione completata con successo!".to_string();
        self.message_color = egui::Color32::GREEN;
        self.state = AppState::Main;
        
        // Pulisci i campi
        self.reg_username.clear();
        self.reg_password.clear();
        self.reg_confirm_password.clear();
    }
    
    pub fn handle_login(&mut self) {
        if let Some(user) = &self.app_data.user {
            let p_h = hash_password(&self.login_password, &user.salt);
            
            if self.login_username == user.u && p_h == user.p_h {
                // Deriva la chiave di crittografia dalla password
                self.encryption_key = Some(derive_key(&self.login_password, &user.key_salt));
                
                self.current_user = Some(user.clone());
                self.state = AppState::Main;
                self.message = "Accesso effettuato con successo!".to_string();
                self.message_color = egui::Color32::GREEN;
                
                // Pulisci i campi
                self.login_username.clear();
                self.login_password.clear();
            } else {
                self.message = "Username o password non corretti!".to_string();
                self.message_color = egui::Color32::RED;
            }
        }
    }
    
    pub fn add_password(&mut self) {
        if self.new_entry_name.is_empty() || self.new_entry_password.is_empty() {
            self.message = "Nome servizio e password sono obbligatori!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }
        
        // Cripta la password
        if let Some(encryption_key) = &self.encryption_key {
            match encrypt_password(&self.new_entry_password, encryption_key) {
                Ok((e_c, nonce)) => {
                    let entry = PasswordEntry {
                        name: self.new_entry_name.clone(),
                        u: self.new_entry_username.clone(),
                        e_c,
                        nonce,
                    };
                    
                    self.app_data.ps.push(entry);
                    save_data(&self.app_data);
                    
                    self.message = "Password aggiunta con successo!".to_string();
                    self.message_color = egui::Color32::GREEN;
                    
                    // Pulisci i campi
                    self.new_entry_name.clear();
                    self.new_entry_username.clear();
                    self.new_entry_password.clear();
                }
                Err(_) => {
                    self.message = "Errore nella crittografia della password!".to_string();
                    self.message_color = egui::Color32::RED;
                }
            }
        } else {
            self.message = "Chiave di crittografia non disponibile!".to_string();
            self.message_color = egui::Color32::RED;
        }
    }
    
    pub fn edit_password(&mut self) {
        if self.edit_service_name.is_empty() || self.edit_new_password.is_empty() {
            self.message = "Nome servizio e nuova password sono obbligatori!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }
        
        if self.edit_new_password != self.edit_confirm_password {
            self.message = "Le password non coincidono!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }
        
        // Trova l'entry da modificare
        let entry_index = self.app_data.ps.iter().position(|entry| {
            entry.name.to_lowercase() == self.edit_service_name.to_lowercase()
        });
        
        match entry_index {
            Some(index) => {
                if let Some(encryption_key) = &self.encryption_key {
                    match encrypt_password(&self.edit_new_password, encryption_key) {
                        Ok((e_c, nonce)) => {
                            // Modifica l'entry esistente
                            let entry = &mut self.app_data.ps[index];
                            entry.e_c = e_c;
                            entry.nonce = nonce;
                            
                            // Modifica l'username solo se è stato specificato
                            if !self.edit_new_username.is_empty() {
                                entry.u = self.edit_new_username.clone();
                            }
                            
                            // Rimuovi dalla lista delle password mostrate se presente
                            self.shown_passwords.remove(&index);
                            
                            save_data(&self.app_data);
                            
                            self.message = format!("Password di '{}' modificata con successo!", self.edit_service_name);
                            self.message_color = egui::Color32::GREEN;
                            
                            // Pulisci i campi
                            self.edit_service_name.clear();
                            self.edit_new_username.clear();
                            self.edit_new_password.clear();
                            self.edit_confirm_password.clear();
                        }
                        Err(_) => {
                            self.message = "Errore nella crittografia della password!".to_string();
                            self.message_color = egui::Color32::RED;
                        }
                    }
                } else {
                    self.message = "Chiave di crittografia non disponibile!".to_string();
                    self.message_color = egui::Color32::RED;
                }
            }
            None => {
                self.message = format!("Servizio '{}' non trovato!", self.edit_service_name);
                self.message_color = egui::Color32::RED;
            }
        }
    }
    
    pub fn logout(&mut self) {
        self.current_user = None;
        self.encryption_key = None;
        self.shown_passwords.clear();
        self.state = AppState::Login;
        self.message = "Logout effettuato.".to_string();
        self.message_color = egui::Color32::BLUE;
    }
}