#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod utils;

use eframe::egui;
use utils::{UserData, PasswordEntry, AppData, generate_salt, hash_password, derive_key, 
           encrypt_password, decrypt_password, load_data, save_data, notifica_conferma};

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
    
    encryption_key: Option<[u8; 32]>,
    
    // Campi per aggiungere password
    new_entry_name: String,
    new_entry_username: String,
    new_entry_password: String,
    
    // Messaggi di errore/successo
    message: String,
    message_color: egui::Color32,
    
    // Tema
    dark_mode: bool,
    
    // Ricerca
    search_query: String,
}

impl Default for PasswordManagerApp {
    fn default() -> Self {
        let app_data = load_data();
        let state = if app_data.user.is_some() {
            AppState::Login
        } else {
            AppState::Registration
        };
        
        let dark_mode = app_data.dark_mode.unwrap_or(true);
        
        Self {
            state,
            reg_username: String::new(),
            reg_password: String::new(),
            reg_confirm_password: String::new(),
            login_username: String::new(),
            login_password: String::new(),
            app_data,
            current_user: None,
            encryption_key: None,
            new_entry_name: String::new(),
            new_entry_username: String::new(),
            new_entry_password: String::new(),
            message: String::new(),
            message_color: egui::Color32::GREEN,
            dark_mode,
            search_query: String::new(),
        }
    }
}

impl eframe::App for PasswordManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Cambia tema
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 10.0);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.spacing.indent = 20.0;
        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::new(26.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Body, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Button, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Monospace, egui::FontId::new(14.0, egui::FontFamily::Monospace)),
            (egui::TextStyle::Small, egui::FontId::new(13.0, egui::FontFamily::Proportional)),
        ]
        .into();
        ctx.set_style(style);

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("🔐 Password Manager");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let theme_text = if self.dark_mode { "🌙" } else { "☀" };

                    if ui.button(theme_text).on_hover_text("Cambia tema").clicked() {
                        self.toggle_theme();
                    }
                    
                    if self.state == AppState::Main {
                        ui.separator();
                        if ui.button("🚪 Exit").clicked() {
                            self.logout();
                        }
                        ui.label(format!("👤 {}", 
                            self.current_user.as_ref().unwrap().username));
                    }
                });
            });
            ui.add_space(4.0);
        });

        if !self.message.is_empty() {
            egui::TopBottomPanel::bottom("messages").show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let icon = match self.message_color {
                        egui::Color32::RED => "⚠",
                        egui::Color32::GREEN => "✓",
                        egui::Color32::YELLOW => "ℹ",
                        _ => "ℹ",
                    };
                    ui.colored_label(self.message_color, format!("{} {}", icon, &self.message));
                });
                ui.add_space(8.0);
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);
            
            match self.state {
                AppState::Registration => self.show_registration(ui),
                AppState::Login => self.show_login(ui),
                AppState::Main => self.show_main(ctx, ui),
            }
        });
    }
}

impl PasswordManagerApp {
    fn toggle_theme(&mut self) {
        self.dark_mode = !self.dark_mode;
        self.app_data.dark_mode = Some(self.dark_mode);
        save_data(&self.app_data);
    }
    
    fn show_registration(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            
            ui.heading("Benvenuto!");
            ui.add_space(10.0);
            ui.label("Crea il tuo account per iniziare a gestire le tue password in sicurezza.");
            ui.add_space(30.0);
            
            egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(8.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.set_max_width(400.0);
                    
                    ui.vertical_centered_justified(|ui| {
                        ui.label("📝 Registrazione");
                        ui.add_space(15.0);
                        
                        egui::Grid::new("reg_grid")
                            .num_columns(2)
                            .spacing([10.0, 12.0])
                            .show(ui, |ui| {
                                ui.label("👤 Username:");
                                ui.add(egui::TextEdit::singleline(&mut self.reg_username)
                                    .desired_width(200.0));
                                ui.end_row();
                                
                                ui.label("🔑 Password:");
                                ui.add(egui::TextEdit::singleline(&mut self.reg_password)
                                    .password(true)
                                    .desired_width(200.0));
                                ui.end_row();
                                
                                ui.label("🔑 Conferma:");
                                ui.add(egui::TextEdit::singleline(&mut self.reg_confirm_password)
                                    .password(true)
                                    .desired_width(200.0));
                                ui.end_row();
                            });
                        
                        ui.add_space(15.0);
                        ui.small("💡 La password deve essere di almeno 6 caratteri");
                        ui.add_space(15.0);
                        
                        if ui.add_sized([120.0, 35.0], egui::Button::new("Registrati")).clicked() {
                            self.handle_registration();
                        }
                    });
                });
        });
    }
    
    fn show_login(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            
            ui.heading("Bentornato!");
            ui.add_space(10.0);
            ui.label("Inserisci le tue credenziali per accedere.");
            ui.add_space(40.0);
            
            egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(8.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.set_max_width(400.0);
                    
                    ui.vertical_centered_justified(|ui| {
                        ui.label("🔓 Accesso");
                        ui.add_space(15.0);
                        
                        egui::Grid::new("login_grid")
                            .num_columns(2)
                            .spacing([10.0, 15.0])
                            .show(ui, |ui| {
                                ui.label("👤 Username:");
                                ui.add(egui::TextEdit::singleline(&mut self.login_username)
                                    .desired_width(200.0));
                                ui.end_row();
                                
                                ui.label("🔑 Password:");
                                ui.add(egui::TextEdit::singleline(&mut self.login_password)
                                    .password(true)
                                    .desired_width(200.0));
                                ui.end_row();
                            });
                        
                        ui.add_space(20.0);
                        
                        if ui.add_sized([100.0, 35.0], egui::Button::new("Accedi")).clicked() {
                            self.handle_login();
                        }
                    });
                });
        });
    }
    
    fn show_main(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_min_width(360.0);
                
                egui::Frame::new()
                    .fill(ui.visuals().faint_bg_color)
                    .corner_radius(8.0)
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.strong("➕ Aggiungi Password");
                            ui.add_space(15.0);

                            ui.vertical(|ui| {
                                ui.label("🏷 Nome servizio");
                                ui.add(egui::TextEdit::singleline(&mut self.new_entry_name)
                                    .hint_text("es. Gmail, Facebook...")
                                    .min_size(egui::vec2(230.0, 25.0)));
                                ui.add_space(10.0);
                                
                                ui.label("👤 Username");
                                ui.add(egui::TextEdit::singleline(&mut self.new_entry_username)
                                    .hint_text("username o email")
                                    .min_size(egui::vec2(230.0, 25.0)));
                                ui.add_space(10.0);
                                
                                ui.label("🔑 Password");
                                ui.add(egui::TextEdit::singleline(&mut self.new_entry_password)
                                    .hint_text("password sicura")
                                    .min_size(egui::vec2(230.0, 25.0)));
                                ui.add_space(15.0);
                            });
                            
                            if ui.add_sized([230.0, 35.0], 
                                egui::Button::new("💾 Salva Password")).clicked() {
                                self.add_password();
                            }
                        });
                    });
            });
            
            ui.separator();
            
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.strong("🗂 Le tue Password");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("❌").on_hover_text("Cancella ricerca").clicked() {
                            self.search_query.clear();
                        }
                        ui.add(egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("🔍 Cerca...")
                            .desired_width(150.0));
                    });
                });
                
                ui.add_space(10.0);
                
                // Filtra passwords
                let filtered_entries: Vec<(usize, &PasswordEntry)> = self.app_data.passwords
                    .iter()
                    .enumerate()
                    .filter(|(_, entry)| {
                        if self.search_query.is_empty() {
                            true
                        } else {
                            entry.name.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                            entry.username.to_lowercase().contains(&self.search_query.to_lowercase())
                        }
                    })
                    .collect();
                
                if !self.search_query.is_empty() && !filtered_entries.is_empty() {
                    ui.small(format!("🎯 {} risultati trovati", filtered_entries.len()));
                    ui.add_space(5.0);
                }
                
                let available_height = ui.available_height();
                
                if filtered_entries.is_empty() {
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), available_height),
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            if self.search_query.is_empty() {
                                ui.label("📭 Nessuna password salvata");
                                ui.small("Aggiungi la tua prima password usando il pannello a sinistra");
                            } else {
                                ui.label("🔍 Nessun risultato");
                                ui.small(format!("Nessuna password trovata per '{}'", self.search_query));
                            }
                        }
                    );
                } else {
                    let mut remove_indices = Vec::new();
                    
                    let entries_to_show: Vec<(usize, PasswordEntry)> = filtered_entries
                        .into_iter()
                        .map(|(index, entry)| (index, entry.clone()))
                        .collect();
                    
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(available_height)
                        .show(ui, |ui| {
                            for (index, entry_clone) in entries_to_show {
                                egui::Frame::new()
                                    .fill(ui.visuals().window_fill)
                                    .corner_radius(6.0)
                                    .inner_margin(12.0)
                                    .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {

                                            ui.vertical(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.strong(&entry_clone.name);
                                                    ui.label("•");
                                                    ui.weak(&entry_clone.username);
                                                });
                                                
                                                if let Some(key) = &self.encryption_key {
                                                    match decrypt_password(&entry_clone, key) {
                                                        Ok(_) => {
                                                            ui.small("🔒 Password protetta");
                                                        }
                                                        Err(_) => {
                                                            ui.colored_label(egui::Color32::RED, "⚠ Errore decrittografia");
                                                        }
                                                    }
                                                } else {
                                                    ui.colored_label(egui::Color32::RED, "⚠ Chiave non disponibile");
                                                }
                                            });
                                            
                                            // Azioni
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {

                                                if ui.button("🗑").on_hover_text("Elimina").clicked() && notifica_conferma() {
                                                    remove_indices.push(index);
                                                }
                                                
                                                // Copia password
                                                if ui.button("📋").on_hover_text("Copia password").clicked() {
                                                    if let Some(key) = &self.encryption_key {
                                                        match decrypt_password(&entry_clone, key) {
                                                            Ok(decrypted_password) => {
                                                                ctx.copy_text(decrypted_password);
                                                                self.message = format!("Password di '{}' copiata!", entry_clone.name);
                                                                self.message_color = egui::Color32::GREEN;
                                                            }
                                                            Err(_) => {
                                                                self.message = "Errore nella decrittografia!".to_string();
                                                                self.message_color = egui::Color32::RED;
                                                            }
                                                        }
                                                    } else {
                                                        self.message = "Chiave di crittografia non disponibile!".to_string();
                                                        self.message_color = egui::Color32::RED;
                                                    }
                                                }
                                                
                                                // Copia user
                                                if ui.button("👤").on_hover_text("Copia username").clicked() {
                                                    ctx.copy_text(entry_clone.username.clone());
                                                    self.message = format!("Username di '{}' copiato!", entry_clone.name);
                                                    self.message_color = egui::Color32::GREEN;
                                                }
                                            });
                                        });
                                    });
                                
                                ui.add_space(8.0);
                            }
                        });
                    
                    // Rimuovi passwords
                    remove_indices.sort_by(|a, b| b.cmp(a));
                    for &index in &remove_indices {
                        let removed_entry = self.app_data.passwords.remove(index);
                        save_data(&self.app_data);
                        self.message = format!("Password '{}' eliminata!", removed_entry.name);
                        self.message_color = egui::Color32::YELLOW;
                    }
                }
            });
        });
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
        
        // Genera salt per l'hash della password e per la derivazione della chiave
        let salt = generate_salt();
        let key_salt = generate_salt();
        let password_hash = hash_password(&self.reg_password, &salt);
        
        // Deriva la chiave di crittografia dalla password
        self.encryption_key = Some(derive_key(&self.reg_password, &key_salt));
        
        let user_data = UserData {
            username: self.reg_username.clone(),
            password_hash,
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
    
    fn handle_login(&mut self) {
        if let Some(user) = &self.app_data.user {
            let password_hash = hash_password(&self.login_password, &user.salt);
            
            if self.login_username == user.username && password_hash == user.password_hash {
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
    
    fn add_password(&mut self) {
        if self.new_entry_name.is_empty() || self.new_entry_password.is_empty() {
            self.message = "Nome servizio e password sono obbligatori!".to_string();
            self.message_color = egui::Color32::RED;
            return;
        }
        
        // Cripta la password
        if let Some(encryption_key) = &self.encryption_key {
            match encrypt_password(&self.new_entry_password, encryption_key) {
                Ok((encrypted_password, nonce)) => {
                    let entry = PasswordEntry {
                        name: self.new_entry_name.clone(),
                        username: self.new_entry_username.clone(),
                        encrypted_password,
                        nonce,
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
    
    fn logout(&mut self) {
        self.current_user = None;
        self.encryption_key = None;
        self.state = AppState::Login;
        self.message = "Logout effettuato.".to_string();
        self.message_color = egui::Color32::BLUE;
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]).with_maximized(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "Password Manager",
        options,
        Box::new(|_cc| Ok(Box::new(PasswordManagerApp::default()))),
    )
}