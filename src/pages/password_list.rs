use std::time::Instant;
use eframe::egui;
use crate::PasswordManagerApp;
use crate::PasswordEntry;
use crate::helpers::utils::*;

impl PasswordManagerApp {
    pub fn show_password_list(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.strong("📃 Le tue Password");
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
                ui.add_space(5.0);
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
                    .id_salt("password_list_scroll")
                    .auto_shrink([false, false])
                    .min_scrolled_height(remaining_space.y)
                    .max_height(remaining_space.y)
                    .show(ui, |ui| {
                        for (index, entry_clone) in entries_to_show {
                            ui.push_id(format!("password_entry_{}", index), |ui| {
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
                                                                if self.shown_passwords.contains_key(&index) {
                                                                    self.shown_passwords.remove(&index);
                                                                } else {
                                                                    self.shown_passwords.insert(index, (decrypted_password, Instant::now()));
                                                                }
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
                            });
                            
                            ui.add_space(8.0);
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
}