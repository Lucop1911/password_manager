use crate::PasswordManagerApp;
use crate::helpers::generate_password::generate_password;
use eframe::egui;

impl PasswordManagerApp {
    pub fn show_edit_password_panel(&mut self, ui: &mut egui::Ui) {
        ui.push_id("edit_password_panel", |ui| {
            egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(8.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.strong("⚙ Edit Password");
                        ui.add_space(15.0);

                        ui.vertical(|ui| {
                            ui.label("🎯 Service to edit");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.edit_service_name)
                                    .hint_text("Name of the existing service")
                                    .min_size(egui::vec2(230.0, 25.0)),
                            );
                            ui.add_space(10.0);

                            ui.label("👤 New username (optional)");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.edit_new_username)
                                    .hint_text("Leave empty to keep current")
                                    .min_size(egui::vec2(230.0, 25.0)),
                            );
                            ui.add_space(10.0);

                            ui.label("🔑 New password");
                            
                            let password_response = ui.add(
                                egui::TextEdit::singleline(&mut self.edit_new_password)
                                    .password(!self.show_password)
                                    .hint_text("New strong password")
                                    .min_size(egui::vec2(230.0, 25.0)),
                            );
                            
                            // Offer a generated-password popup when the new
                            // password field is empty and gains focus
                            if password_response.gained_focus() && self.edit_new_password.is_empty() {
                                self.show_popup_edit = true;
                            }
                            
                            // Password suggestion popup
                            if self.show_popup_edit {
                                let popup_id = ui.make_persistent_id("password_gen_popup_edit");
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
                                                            let p = generate_password();
                                                            self.edit_new_password = p.clone();
                                                            self.edit_confirm_password = p;
                                                            self.show_popup_edit = false;
                                                        }
                                                        
                                                        if ui.button("❌ No thanks").clicked() {
                                                            self.show_popup_edit = false;
                                                        }
                                                    });
                                                });
                                            });
                                    });
                            }
                            
                            // Dismiss the popup when clicking anywhere
                            // outside the field or the popup itself
                            if self.show_popup_edit && ui.input(|i| i.pointer.any_click()) {
                                let popup_id = ui.make_persistent_id("password_gen_popup_edit");
                                if let Some(area_response) = ui.ctx().memory(|mem| {
                                    mem.area_rect(popup_id)
                                }) {
                                    if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                                        if !area_response.contains(pointer_pos) && !password_response.rect.contains(pointer_pos) {
                                            self.show_popup_edit = false;
                                        }
                                    }
                                }
                            }

                            ui.checkbox(&mut self.show_password, "Show");
                            ui.add_space(10.0);

                            ui.label("🔑 Confirm password");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.edit_confirm_password)
                                    .password(!self.show_password1)
                                    .hint_text("Repeat the new password")
                                    .min_size(egui::vec2(230.0, 25.0)),
                            );
                            ui.checkbox(&mut self.show_password1, "Show");
                            ui.add_space(15.0);
                        });

                        if ui
                            .add_sized([230.0, 35.0], egui::Button::new("🔄 Update Password"))
                            .clicked()
                        {
                            self.show_password = false;
                            self.show_password1 = false;
                            self.edit_password();
                        }
                    });
                });
        });
    }
}
