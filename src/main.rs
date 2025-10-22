#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod utils;
mod handlers;

use eframe::egui;
use utils::{UserData, PasswordEntry, AppData, load_data, decrypt_password, save_data, confirm_notification};
use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(PartialEq)]
pub enum AppState {
    Registration,
    Login,
    Main,
}

pub struct PasswordManagerApp {
    pub state: AppState,
    
    // Campi per registrazione/login
    pub reg_username: String,
    pub reg_password: String,
    pub reg_confirm_password: String,
    
    pub login_username: String,
    pub login_password: String,
    
    // Dati dell'app
    pub app_data: AppData,
    pub current_user: Option<UserData>,
    
    pub encryption_key: Option<[u8; 32]>,
    
    // Campi per aggiungere password
    pub new_entry_name: String,
    pub new_entry_username: String,
    pub new_entry_password: String,
    
    // Campi per modificare password
    pub edit_service_name: String,
    pub edit_new_username: String,
    pub edit_new_password: String,
    pub edit_confirm_password: String,
    
    // Messaggi di errore/successo
    pub message: String,
    pub message_color: egui::Color32,
    
    // Tema
    pub dark_mode: bool,
    
    // Ricerca
    pub search_query: String,
    
    // Mostra password temporaneamente (indice -> (password, tempo_inizio))
    pub shown_passwords: HashMap<usize, (String, Instant)>,
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
            edit_service_name: String::new(),
            edit_new_username: String::new(),
            edit_new_password: String::new(),
            edit_confirm_password: String::new(),
            message: String::new(),
            message_color: egui::Color32::GREEN,
            dark_mode,
            search_query: String::new(),
            shown_passwords: HashMap::new(),
        }
    }
}

// Struttura per contenere le dimensioni responsive
struct ResponsiveSizes {
    heading_size: f32,
    body_size: f32,
    button_size: f32,
    small_size: f32,
    item_spacing: f32,
    button_padding: egui::Vec2,
    frame_margin: f32,
    panel_width: f32,
    text_field_width: f32,
    button_height: f32,
}

impl ResponsiveSizes {
    fn new(screen_width: f32) -> Self {
        if screen_width < 600.0 {
            // Schermi molto piccoli
            Self {
                heading_size: 18.0,
                body_size: 12.0,
                button_size: 12.0,
                small_size: 10.0,
                item_spacing: 4.0,
                button_padding: egui::vec2(6.0, 4.0),
                frame_margin: 8.0,
                panel_width: 220.0,
                text_field_width: 150.0,
                button_height: 28.0,
            }
        } else if screen_width < 900.0 {
            // Schermi piccoli
            Self {
                heading_size: 20.0,
                body_size: 13.0,
                button_size: 13.0,
                small_size: 11.0,
                item_spacing: 6.0,
                button_padding: egui::vec2(8.0, 5.0),
                frame_margin: 12.0,
                panel_width: 280.0,
                text_field_width: 180.0,
                button_height: 30.0,
            }
        } else if screen_width < 1200.0 {
            // Schermi medi
            Self {
                heading_size: 22.0,
                body_size: 14.0,
                button_size: 14.0,
                small_size: 12.0,
                item_spacing: 7.0,
                button_padding: egui::vec2(10.0, 6.0),
                frame_margin: 15.0,
                panel_width: 320.0,
                text_field_width: 200.0,
                button_height: 32.0,
            }
        } else {
            // Schermi grandi
            Self {
                heading_size: 26.0,
                body_size: 16.0,
                button_size: 16.0,
                small_size: 13.0,
                item_spacing: 8.0,
                button_padding: egui::vec2(12.0, 8.0),
                frame_margin: 20.0,
                panel_width: 360.0,
                text_field_width: 230.0,
                button_height: 35.0,
            }
        }
    }
}

impl eframe::App for PasswordManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Pulizia delle password mostrate dopo 10 secondi
        let now = Instant::now();
        let expired_keys: Vec<usize> = self.shown_passwords
            .iter()
            .filter(|(_, (_, start_time))| now.duration_since(*start_time) > Duration::from_secs(10))
            .map(|(&key, _)| key)
            .collect();
        
        for key in expired_keys {
            self.shown_passwords.remove(&key);
        }
        
        // Richiedo il refresh ogni secondo per aggiornare i timer
        ctx.request_repaint_after(Duration::from_secs(1));
        
        // Ottieni le dimensioni dello schermo
        let screen_width = ctx.screen_rect().width();
        let sizes = ResponsiveSizes::new(screen_width);
        
        // Cambia tema
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(sizes.item_spacing, sizes.item_spacing + 2.0);
        style.spacing.button_padding = sizes.button_padding;
        style.spacing.indent = sizes.item_spacing * 2.5;
        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::new(sizes.heading_size, egui::FontFamily::Proportional)),
            (egui::TextStyle::Body, egui::FontId::new(sizes.body_size, egui::FontFamily::Proportional)),
            (egui::TextStyle::Button, egui::FontId::new(sizes.button_size, egui::FontFamily::Proportional)),
            (egui::TextStyle::Monospace, egui::FontId::new(sizes.body_size - 2.0, egui::FontFamily::Monospace)),
            (egui::TextStyle::Small, egui::FontId::new(sizes.small_size, egui::FontFamily::Proportional)),
        ]
        .into();
        ctx.set_style(style);

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(sizes.item_spacing);
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
                            self.current_user.as_ref().unwrap().u));
                    }
                });
            });
            ui.add_space(sizes.item_spacing / 2.0);
        });

        if !self.message.is_empty() {
            egui::TopBottomPanel::bottom("messages").show(ctx, |ui| {
                ui.add_space(sizes.item_spacing);
                ui.horizontal(|ui| {
                    let icon = match self.message_color {
                        egui::Color32::RED => "❌",
                        egui::Color32::GREEN => "✅",
                        egui::Color32::YELLOW => "🔸",
                        _ => "🔸",
                    };
                    ui.colored_label(self.message_color, format!("{} {}", icon, &self.message));
                });
                ui.add_space(sizes.item_spacing);
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(sizes.frame_margin);
            
            match self.state {
                AppState::Registration => self.show_registration(ui, &sizes),
                AppState::Login => self.show_login(ui, &sizes),
                AppState::Main => self.show_main(ctx, ui, &sizes),
            }
        });
    }
}

impl PasswordManagerApp {
    fn show_registration(&mut self, ui: &mut egui::Ui, sizes: &ResponsiveSizes) {
        ui.vertical_centered(|ui| {
            ui.add_space(sizes.frame_margin * 2.0);
            
            ui.heading("Benvenuto!");
            ui.add_space(sizes.item_spacing);
            ui.label("Crea il tuo account per iniziare a gestire le tue password in sicurezza.");
            ui.add_space(sizes.frame_margin * 1.5);
            
            egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(sizes.item_spacing)
                .inner_margin(sizes.frame_margin)
                .show(ui, |ui| {
                    ui.set_max_width(sizes.panel_width * 1.5);
                    
                    ui.vertical_centered_justified(|ui| {
                        ui.label("📝 Registrazione");
                        ui.add_space(sizes.frame_margin * 0.75);
                        
                        egui::Grid::new("reg_grid")
                            .num_columns(2)
                            .spacing([sizes.item_spacing, sizes.item_spacing * 1.5])
                            .show(ui, |ui| {
                                ui.label("👤 Username:");
                                ui.add(egui::TextEdit::singleline(&mut self.reg_username)
                                    .desired_width(sizes.text_field_width));
                                ui.end_row();
                                
                                ui.label("🔑 Password:");
                                ui.add(egui::TextEdit::singleline(&mut self.reg_password)
                                    .password(true)
                                    .desired_width(sizes.text_field_width));
                                ui.end_row();
                                
                                ui.label("🔑 Conferma:");
                                ui.add(egui::TextEdit::singleline(&mut self.reg_confirm_password)
                                    .password(true)
                                    .desired_width(sizes.text_field_width));
                                ui.end_row();
                            });
                        
                        ui.add_space(sizes.frame_margin * 0.75);
                        ui.small("💡 La password deve essere di almeno 6 caratteri");
                        ui.add_space(sizes.frame_margin * 0.75);
                        
                        if ui.add_sized([sizes.text_field_width * 0.6, sizes.button_height], 
                            egui::Button::new("Registrati")).clicked() {
                            self.handle_registration();
                        }
                    });
                });
        });
    }
    
    fn show_login(&mut self, ui: &mut egui::Ui, sizes: &ResponsiveSizes) {
        ui.vertical_centered(|ui| {
            ui.add_space(sizes.frame_margin * 3.0);
            
            ui.heading("Bentornato!");
            ui.add_space(sizes.item_spacing);
            ui.label("Inserisci le tue credenziali per accedere.");
            ui.add_space(sizes.frame_margin * 2.0);
            
            egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(sizes.item_spacing)
                .inner_margin(sizes.frame_margin)
                .show(ui, |ui| {
                    ui.set_max_width(sizes.panel_width * 1.5);
                    
                    ui.vertical_centered_justified(|ui| {
                        ui.label("🔓 Accesso");
                        ui.add_space(sizes.frame_margin * 0.75);
                        
                        egui::Grid::new("login_grid")
                            .num_columns(2)
                            .spacing([sizes.item_spacing, sizes.item_spacing * 1.875])
                            .show(ui, |ui| {
                                ui.label("👤 Username:");
                                ui.add(egui::TextEdit::singleline(&mut self.login_username)
                                    .desired_width(sizes.text_field_width));
                                ui.end_row();
                                
                                ui.label("🔑 Password:");
                                ui.add(egui::TextEdit::singleline(&mut self.login_password)
                                    .password(true)
                                    .desired_width(sizes.text_field_width));
                                ui.end_row();
                            });
                        
                        ui.add_space(sizes.frame_margin);
                        
                        if ui.add_sized([sizes.text_field_width * 0.5, sizes.button_height], 
                            egui::Button::new("Accedi")).clicked() {
                            self.handle_login();
                        }
                    });
                });
        });
    }
    
    fn show_main(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, sizes: &ResponsiveSizes) {
        ui.allocate_ui_with_layout(
            ui.available_size(),
            egui::Layout::left_to_right(egui::Align::TOP),
            |ui| {
                // Pannello a sinistra
                ui.vertical(|ui| {
                    ui.set_min_width(sizes.panel_width);
                    ui.set_max_width(sizes.panel_width);
                    
                    // Sezione Aggiungi Password
                    egui::Frame::new()
                        .fill(ui.visuals().faint_bg_color)
                        .corner_radius(sizes.item_spacing)
                        .inner_margin(sizes.frame_margin)
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.strong("➕ Aggiungi Password");
                                ui.add_space(sizes.frame_margin * 0.75);

                                ui.vertical(|ui| {
                                    ui.label("🏷 Nome servizio");
                                    ui.add(egui::TextEdit::singleline(&mut self.new_entry_name)
                                        .hint_text("es. Gmail, Facebook...")
                                        .min_size(egui::vec2(sizes.text_field_width, sizes.button_height * 0.71)));
                                    ui.add_space(sizes.item_spacing);
                                    
                                    ui.label("👤 Username");
                                    ui.add(egui::TextEdit::singleline(&mut self.new_entry_username)
                                        .hint_text("username o email")
                                        .min_size(egui::vec2(sizes.text_field_width, sizes.button_height * 0.71)));
                                    ui.add_space(sizes.item_spacing);
                                    
                                    ui.label("🔑 Password");
                                    ui.add(egui::TextEdit::singleline(&mut self.new_entry_password)
                                        .password(true)
                                        .hint_text("password sicura")
                                        .min_size(egui::vec2(sizes.text_field_width, sizes.button_height * 0.71)));
                                    ui.add_space(sizes.frame_margin * 0.75);
                                });
                                
                                if ui.add_sized([sizes.text_field_width, sizes.button_height], 
                                    egui::Button::new("💾 Salva Password")).clicked() {
                                    self.add_password();
                                }
                            });
                        });
                    
                    ui.add_space(sizes.frame_margin * 0.75);
                    
                    // Sezione Modifica Password
                    egui::Frame::new()
                        .fill(ui.visuals().faint_bg_color)
                        .corner_radius(sizes.item_spacing)
                        .inner_margin(sizes.frame_margin)
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.strong("⚙ Modifica Password");
                                ui.add_space(sizes.frame_margin * 0.75);

                                ui.vertical(|ui| {
                                    ui.label("🎯 Servizio da modificare");
                                    ui.add(egui::TextEdit::singleline(&mut self.edit_service_name)
                                        .hint_text("Nome del servizio esistente")
                                        .min_size(egui::vec2(sizes.text_field_width, sizes.button_height * 0.71)));
                                    ui.add_space(sizes.item_spacing);
                                    
                                    ui.label("👤 Nuovo username (opzionale)");
                                    ui.add(egui::TextEdit::singleline(&mut self.edit_new_username)
                                        .hint_text("Lascia vuoto per non modificare")
                                        .min_size(egui::vec2(sizes.text_field_width, sizes.button_height * 0.71)));
                                    ui.add_space(sizes.item_spacing);
                                    
                                    ui.label("🔑 Nuova password");
                                    ui.add(egui::TextEdit::singleline(&mut self.edit_new_password)
                                        .password(true)
                                        .hint_text("Nuova password sicura")
                                        .min_size(egui::vec2(sizes.text_field_width, sizes.button_height * 0.71)));
                                    ui.add_space(sizes.item_spacing);
                                    
                                    ui.label("🔑 Conferma password");
                                    ui.add(egui::TextEdit::singleline(&mut self.edit_confirm_password)
                                        .password(true)
                                        .hint_text("Ripeti la nuova password")
                                        .min_size(egui::vec2(sizes.text_field_width, sizes.button_height * 0.71)));
                                    ui.add_space(sizes.frame_margin * 0.75);
                                });
                                
                                if ui.add_sized([sizes.text_field_width, sizes.button_height], 
                                    egui::Button::new("🔄 Modifica Password")).clicked() {
                                    self.edit_password();
                                }
                            });
                        });
                    
                    ui.allocate_space(egui::vec2(ui.available_width(), ui.available_height()));
                });
                
                ui.separator();
                
                // Pannello a destra
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong("📃 Le tue Password");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("❌").on_hover_text("Cancella ricerca").clicked() {
                                self.search_query.clear();
                            }
                            ui.add(egui::TextEdit::singleline(&mut self.search_query)
                                .hint_text("🔍 Cerca...")
                                .desired_width(sizes.text_field_width * 0.65));
                        });
                    });
                    
                    ui.add_space(sizes.item_spacing);
                    
                    let filtered_entries: Vec<(usize, &PasswordEntry)> = self.app_data.ps
                        .iter()
                        .enumerate()
                        .filter(|(_, entry)| {
                            if self.search_query.is_empty() {
                                true
                            } else {
                                entry.name.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                                entry.u.to_lowercase().contains(&self.search_query.to_lowercase())
                            }
                        })
                        .collect();
                    
                    if !self.search_query.is_empty() && !filtered_entries.is_empty() {
                        ui.small(format!("🎯 {} risultati trovati", filtered_entries.len()));
                        ui.add_space(sizes.item_spacing * 0.625);
                    }
                    
                    let remaining_space = ui.available_size();
                    
                    if filtered_entries.is_empty() {
                        ui.allocate_ui_with_layout(
                            remaining_space,
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
                        
                        // Sezione mostra password
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .min_scrolled_height(remaining_space.y)
                            .max_height(remaining_space.y)
                            .show(ui, |ui| {
                                for (index, entry_clone) in entries_to_show {
                                    egui::Frame::new()
                                        .fill(ui.visuals().window_fill)
                                        .corner_radius(sizes.item_spacing * 0.75)
                                        .inner_margin(sizes.frame_margin * 0.6)
                                        .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.vertical(|ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.strong(&entry_clone.name);
                                                        ui.label("•");
                                                        ui.weak(&entry_clone.u);
                                                    });
                                                    
                                                    if let Some((password, start_time)) = self.shown_passwords.get(&index) {
                                                        let remaining_time = 10 - start_time.elapsed().as_secs();
                                                        ui.horizontal(|ui| {
                                                            ui.colored_label(egui::Color32::YELLOW, format!("🔓 {}", password));
                                                            ui.small(format!("({}s)", remaining_time));
                                                        });
                                                    } else {
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
                                                    }
                                                });
                                                
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {

                                                    if ui.button("🗑").on_hover_text("Elimina").clicked() {
                                                        if confirm_notification() {
                                                            remove_indices.push(index);
                                                        }
                                                    }
                                                    
                                                    if ui.button("🔓").on_hover_text("Mostra Password").clicked() {
                                                        if let Some(key) = &self.encryption_key {
                                                            match decrypt_password(&entry_clone, key) {
                                                                Ok(decrypted_password) => {
                                                                    self.shown_passwords.insert(index, (decrypted_password, Instant::now()));
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

                                                    if ui.button("📋").on_hover_text("Copia password").clicked() {
                                                        if let Some(key) = &self.encryption_key {
                                                            match decrypt_password(&entry_clone, key) {
                                                                Ok(decrypted_password) => {
                                                                    ctx.copy_text(decrypted_password);
                                                                    self.message = format!("La password di '{}' è stata copiata!", entry_clone.name);
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
                                                    
                                                    if ui.button("👤").on_hover_text("Copia username").clicked() {
                                                        ctx.copy_text(entry_clone.u.clone());
                                                        self.message = format!("L'username di '{}' è stato copiato!", entry_clone.name);
                                                        self.message_color = egui::Color32::GREEN;
                                                    }
                                                });
                                            });
                                        });
                                    
                                    ui.add_space(sizes.item_spacing);
                                }
                            });
                        
                        // Rimuovi password
                        if !remove_indices.is_empty() {
                            
                            remove_indices.sort_by(|a, b| b.cmp(a));
                            
                            let mut removed_names = Vec::new();
                            for &index in &remove_indices {
                                if index < self.app_data.ps.len() {
                                    let removed_entry = self.app_data.ps.remove(index);
                                    removed_names.push(removed_entry.name);
                                    // Rimuovi anche dalle password mostrate se presente
                                    self.shown_passwords.remove(&index);
                                }
                            }
                            
                            if !removed_names.is_empty() {
                                save_data(&self.app_data);
                                if removed_names.len() == 1 {
                                    self.message = format!("La password di '{}' è stata eliminata!", removed_names[0]);
                                } else {
                                    self.message = format!("{} password eliminate!", removed_names.len());
                                }
                                self.message_color = egui::Color32::RED;
                            }
                        }
                    }
                });
            }
        );
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "Password Manager",
        options,
        Box::new(|_cc| Ok(Box::new(PasswordManagerApp::default()))),
    )
}