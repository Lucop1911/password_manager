use crate::PasswordManagerApp;
use crate::helpers::generate_password::generate_password;
use eframe::egui;

impl PasswordManagerApp {
    pub fn show_add_password_panel(&mut self, ui: &mut egui::Ui) {
        ui.push_id("add_password_panel", |ui| {
            egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(8.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.strong("➕ Add Password");
                        ui.add_space(15.0);

                        ui.vertical(|ui| {
                            ui.label("🏷 Service name");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.new_entry_name)
                                    .hint_text("e.g. Gmail, Facebook...")
                                    .min_size(egui::vec2(230.0, 25.0)),
                            );
                            ui.add_space(10.0);

                            ui.label("👤 Username");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.new_entry_username)
                                    .hint_text("username or email")
                                    .min_size(egui::vec2(230.0, 25.0)),
                            );
                            ui.add_space(10.0);

                            ui.label("🔑 Password");
                            
                            let password_response = ui.add(
                                egui::TextEdit::singleline(&mut self.new_entry_password)
                                    .password(!self.show_password)
                                    .hint_text("strong password")
                                    .min_size(egui::vec2(230.0, 25.0)),
                            );
                            
                            // When the password field is empty and gains
                            // focus, offer a generated-password popup
                            if password_response.gained_focus() && self.new_entry_password.is_empty() {
                                self.show_popup_add = true;
                            }
                            
                            // Password suggestion popup
                            if self.show_popup_add {
                                let popup_id = ui.make_persistent_id("password_gen_popup_add");
                                egui::Area::new(popup_id)
                                    .fixed_pos(password_response.rect.left_bottom() + egui::vec2(0.0, 5.0))
                                    .show(ui.ctx(), |ui| {
                                        egui::Frame::popup(ui.style())
                                            .show(ui, |ui| {
                                                ui.set_min_width(230.0);
                                                ui.vertical(|ui| {
                                                    ui.label("🎲 Generate a strong password?");
                                                    ui.add_space(8.0);
                                                    
                                                    ui.horizontal(|ui| {
                                                        if ui.button("✅ Generate").clicked() {
                                                            self.new_entry_password = generate_password();
                                                            self.show_popup_add = false;
                                                        }
                                                        
                                                        if ui.button("❌ No thanks").clicked() {
                                                            self.show_popup_add = false;
                                                        }
                                                    });
                                                });
                                            });
                                    });
                            }
                            
                            // Dismiss the popup when clicking anywhere
                            // outside the field or the popup itself
                            if self.show_popup_add && ui.input(|i| i.pointer.any_click()) {
                                let popup_id = ui.make_persistent_id("password_gen_popup_add");
                                if let Some(area_response) = ui.ctx().memory(|mem| {
                                    mem.area_rect(popup_id)
                                }) {
                                    if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                                        if !area_response.contains(pointer_pos) && !password_response.rect.contains(pointer_pos) {
                                            self.show_popup_add = false;
                                        }
                                    }
                                }
                            }
                            
                            ui.checkbox(&mut self.show_password, "Show");
                            ui.add_space(15.0);
                        });

                        if ui
                            .add_sized([230.0, 35.0], egui::Button::new("💾 Save Password"))
                            .clicked()
                        {
                            self.show_password = false;
                            self.show_password1 = false;
                            self.add_password();
                        }
                    });
                });
        });
    }
}
