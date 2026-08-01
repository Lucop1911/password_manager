use std::time::Instant;
use eframe::egui;
use crate::PasswordManagerApp;
use crate::helpers::utils::{confirm_notification, decrypt_password};

impl PasswordManagerApp {
    pub fn show_password_list(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.strong("📃 Your Passwords");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌").on_hover_text("Clear search").clicked() {
                        self.search_query.clear();
                    }
                    ui.add(egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("🔍 Search...")
                        .desired_width(150.0));
                });
            });
            
            ui.add_space(10.0);
            
            // Work on clones so we can freely mutate session state
            // (revealed passwords, pending deletions) while iterating
            let filtered_entries: Vec<_> = self.ps
                .iter()
                .filter(|entry| {
                    if self.search_query.is_empty() {
                        true
                    } else {
                        entry.name.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                        entry.u.to_lowercase().contains(&self.search_query.to_lowercase())
                    }
                })
                .cloned()
                .collect();
            
            if !self.search_query.is_empty() && !filtered_entries.is_empty() {
                ui.small(format!("🎯 {} results found", filtered_entries.len()));
                ui.add_space(5.0);
            }
            
            let remaining_space = ui.available_size();
            
            if filtered_entries.is_empty() {
                ui.allocate_ui_with_layout(
                    remaining_space,
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        if self.search_query.is_empty() {
                            ui.label("📭 No passwords saved");
                            ui.small("Add your first password using the panel on the left");
                        } else {
                            ui.label("🔍 No results");
                            ui.small(format!("No passwords found for '{}'", self.search_query));
                        }
                    }
                );
            } else {
                let mut remove_ids = Vec::new();
                
                // Password display section
                egui::ScrollArea::vertical()
                    .id_salt("password_list_scroll")
                    .auto_shrink([false, false])
                    .min_scrolled_height(remaining_space.y)
                    .max_height(remaining_space.y)
                    .show(ui, |ui| {
                        for entry in &filtered_entries {
                            ui.push_id(entry.id, |ui| {
                                egui::Frame::new()
                                    .fill(ui.visuals().window_fill)
                                    .corner_radius(6.0)
                                    .inner_margin(12.0)
                                    .stroke(egui::Stroke::new(1.0_f32, ui.visuals().widgets.noninteractive.bg_stroke.color))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.vertical(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.strong(&entry.name);
                                                    ui.label("•");
                                                    ui.weak(&entry.u);
                                                });
                                                
                                                // Revealed passwords expire after 10 seconds;
                                                // remaining time is shown as a countdown
                                                if let Some((password, start_time)) = self.shown_passwords.get(&entry.id) {
                                                    let remaining_time = 10 - start_time.elapsed().as_secs();
                                                    ui.horizontal(|ui| {
                                                        ui.colored_label(egui::Color32::YELLOW, format!("🔓 {}", password));
                                                        ui.small(format!("({}s)", remaining_time));
                                                    });
                                                } else {
                                                    if let Some(key) = &self.encryption_key {
                                                        match decrypt_password(entry, key) {
                                                            Ok(_) => {
                                                                ui.small("🔒 Password protected");
                                                            }
                                                            Err(_) => {
                                                                ui.colored_label(egui::Color32::RED, "⚠ Decryption error");
                                                            }
                                                        }
                                                    } else {
                                                        ui.colored_label(egui::Color32::RED, "⚠ Key unavailable");
                                                    }
                                                }
                                            });
                                            
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {

                                                if ui.button("🗑").on_hover_text("Delete").clicked() {
                                                    if confirm_notification() {
                                                        remove_ids.push(entry.id);
                                                    }
                                                }
                                                
                                                if ui.button("🔓").on_hover_text("Show Password").clicked() {
                                                    if let Some(key) = &self.encryption_key {
                                                        match decrypt_password(entry, key) {
                                                            Ok(decrypted_password) => {
                                                                if self.shown_passwords.remove(&entry.id).is_none() {
                                                                    self.shown_passwords.insert(entry.id, (decrypted_password, Instant::now()));
                                                                }
                                                            }
                                                            Err(_) => {
                                                                self.message = "Decryption error!".to_string();
                                                                self.message_color = egui::Color32::RED;
                                                            }
                                                        }
                                                    } else {
                                                        self.message = "Encryption key unavailable!".to_string();
                                                        self.message_color = egui::Color32::RED;
                                                    }
                                                }

                                                if ui.button("📋").on_hover_text("Copy password").clicked() {
                                                    if let Some(key) = &self.encryption_key {
                                                        match decrypt_password(entry, key) {
                                                            Ok(decrypted_password) => {
                                                                ctx.copy_text(decrypted_password);
                                                                self.message = format!("Password for '{}' copied!", entry.name);
                                                                self.message_color = egui::Color32::GREEN;
                                                            }
                                                            Err(_) => {
                                                                self.message = "Decryption error!".to_string();
                                                                self.message_color = egui::Color32::RED;
                                                            }
                                                        }
                                                    } else {
                                                        self.message = "Encryption key unavailable!".to_string();
                                                        self.message_color = egui::Color32::RED;
                                                    }
                                                }
                                                
                                                if ui.button("👤").on_hover_text("Copy username").clicked() {
                                                    ctx.copy_text(entry.u.clone());
                                                    self.message = format!("Username for '{}' copied!", entry.name);
                                                    self.message_color = egui::Color32::GREEN;
                                                }
                                            });
                                        });
                                    });
                            });
                            
                            ui.add_space(8.0);
                        }
                    });
                
                // Deletions are applied after the loop so the iterated list
                // is not mutated mid-render
                for id in remove_ids {
                    self.remove_password(id);
                }
            }
        });
    }
}
