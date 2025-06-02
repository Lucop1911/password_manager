use eframe::egui;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use base64::Engine;
use rand::Rng;

const DATA_FILE: &str = "password_data.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserData {
    username: String,
    password_hash: String,
    salt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PasswordEntry {
    name: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppData {
    user: Option<UserData>,
    passwords: Vec<PasswordEntry>,
}

#[derive(PartialEq)]
enum AppState {
    Registration,
    Login,
    Main,
}

struct PasswordManagerApp {
    state: AppState,
    
    // Campi per registrazione/login
    reg_username: String,
    reg_password: String,
    reg_confirm_password: String,
    
    login_username: String,
    login_password: String,
    
    // Dati dell'app
    app_data: AppData,
    current_user: Option<UserData>,
    
    // Campi per aggiungere password
    new_entry_name: String,
    new_entry_username: String,
    new_entry_password: String,
    
    // Messaggi di errore/successo
    message: String,
    message_color: egui::Color32,
}

impl Default for PasswordManagerApp {
    fn default() -> Self {
        let app_data = load_data();
        let state = if app_data.user.is_some() {
            AppState::Login
        } else {
            AppState::Registration
        };
        
        Self {
            state,
            reg_username: String::new(),
            reg_password: String::new(),
            reg_confirm_password: String::new(),
            login_username: String::new(),
            login_password: String::new(),
            app_data,
            current_user: None,
            new_entry_name: String::new(),
            new_entry_username: String::new(),
            new_entry_password: String::new(),
            message: String::new(),
            message_color: egui::Color32::GREEN,
        }
    }
}

impl eframe::App for PasswordManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state {
                AppState::Registration => self.show_registration(ui),
                AppState::Login => self.show_login(ui),
                AppState::Main => self.show_main(ui),
            }
            
            // Mostra messaggi
            if !self.message.is_empty() {
                ui.separator();
                ui.colored_label(self.message_color, &self.message);
            }
        });
    }
}

impl PasswordManagerApp {
    fn show_registration(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔐 Registrazione Password Manager");
        ui.separator();
        
        ui.label("Crea il tuo account per iniziare:");
        
        ui.horizontal(|ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut self.reg_username);
        });
        
        ui.horizontal(|ui| {
            ui.label("Password:");
            ui.add(egui::TextEdit::singleline(&mut self.reg_password).password(true));
        });
        
        ui.horizontal(|ui| {
            ui.label("Conferma Password:");
            ui.add(egui::TextEdit::singleline(&mut self.reg_confirm_password).password(true));
        });
        
        ui.separator();
        
        if ui.button("Registrati").clicked() {
            self.handle_registration();
        }
    }
    
    fn show_login(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔓 Accesso Password Manager");
        ui.separator();
        
        ui.label("Inserisci le tue credenziali:");
        
        ui.horizontal(|ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut self.login_username);
        });
        
        ui.horizontal(|ui| {
            ui.label("Password:");
            ui.add(egui::TextEdit::singleline(&mut self.login_password).password(true));
        });
        
        ui.separator();
        
        if ui.button("Accedi").clicked() {
            self.handle_login();
        }
    }
    
    fn show_main(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔐 Le tue Password");
        
        // Pulsante logout
        ui.horizontal(|ui| {
            if ui.button("Exit").clicked() {
                self.logout();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("Benvenuto, {}!", 
                    self.current_user.as_ref().unwrap().username));
            });
        });
        
        ui.separator();
        
        // Sezione per aggiungere nuove password
        ui.heading("Aggiungi nuova password");
        
        ui.horizontal(|ui| {
            ui.label("Nome servizio:");
            ui.text_edit_singleline(&mut self.new_entry_name);
        });
        
        ui.horizontal(|ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut self.new_entry_username);
        });
        
        ui.horizontal(|ui| {
            ui.label("Password:");
            ui.text_edit_singleline(&mut self.new_entry_password);
        });
        
        if ui.button("Aggiungi Password").clicked() {
            self.add_password();
        }
        
        ui.separator();
        
        // Lista delle password salvate
        ui.heading("Password salvate");
        
        if self.app_data.passwords.is_empty() {
            ui.label("Nessuna password salvata ancora.");
        } else {
            // Creiamo una copia per evitare problemi di borrowing
            let passwords_to_remove: Vec<usize> = Vec::new();
            let mut remove_indices = passwords_to_remove;
            
            for (index, entry) in self.app_data.passwords.iter().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong(&entry.name);
                            ui.label(format!("Username: {}", entry.username));
                            ui.label(format!("Password: {}", entry.password));
                        });
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑").clicked() {
                                remove_indices.push(index);
                            }
                        });
                    });
                });
                ui.separator();
            }
            
            // Rimuovi le password selezionate
            for &index in remove_indices.iter().rev() {
                self.app_data.passwords.remove(index);
                save_data(&self.app_data);
                self.message = "Password eliminata!".to_string();
                self.message_color = egui::Color32::YELLOW;
            }
        }
    }
    
    fn handle_registration(&mut self) {
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
        
        // Genera salt e hash della password
        let salt = generate_salt();
        let password_hash = hash_password(&self.reg_password, &salt);
        
        let user_data = UserData {
            username: self.reg_username.clone(),
            password_hash,
            salt,
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
    
    fn handle_login(&mut self) {
        if let Some(user) = &self.app_data.user {
            let password_hash = hash_password(&self.login_password, &user.salt);
            
            if self.login_username == user.username && password_hash == user.password_hash {
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
    
    fn add_password(&mut self) {
        if self.new_entry_name.is_empty() || self.new_entry_password.is_empty() {
            self.message = "Nome servizio e password sono obbligatori!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }
        
        let entry = PasswordEntry {
            name: self.new_entry_name.clone(),
            username: self.new_entry_username.clone(),
            password: self.new_entry_password.clone(),
        };
        
        self.app_data.passwords.push(entry);
        save_data(&self.app_data);
        
        self.message = "Password aggiunta con successo!".to_string();
        self.message_color = egui::Color32::GREEN;
        
        // Pulisci i campi
        self.new_entry_name.clear();
        self.new_entry_username.clear();
        self.new_entry_password.clear();
    }
    
    fn logout(&mut self) {
        self.current_user = None;
        self.state = AppState::Login;
        self.message = "Logout effettuato.".to_string();
        self.message_color = egui::Color32::BLUE;
    }
}

// Funzioni di utilità
fn generate_salt() -> String {
    let mut rng = rand::rng();
    let salt: [u8; 16] = rng.random();
    base64::engine::general_purpose::STANDARD.encode(salt)
}

fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let result = hasher.finalize();
    base64::engine::general_purpose::STANDARD.encode(result)
}

fn load_data() -> AppData {
    if Path::new(DATA_FILE).exists() {
        let data = fs::read_to_string(DATA_FILE).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_else(|_| AppData {
            user: None,
            passwords: Vec::new(),
        })
    } else {
        AppData {
            user: None,
            passwords: Vec::new(),
        }
    }
}

fn save_data(data: &AppData) {
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = fs::write(DATA_FILE, json);
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 500.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Password Manager",
        options,
        Box::new(|_cc| Ok(Box::new(PasswordManagerApp::default()))),
    )
}