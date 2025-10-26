use eframe::egui;
use crate::PasswordManagerApp;

impl PasswordManagerApp {
    pub fn show_edit_password_panel(&mut self, ui: &mut egui::Ui) {
        ui.push_id("edit_password_panel", |ui| {
            egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(8.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.strong("⚙ Modifica Password");
                        ui.add_space(15.0);

                        ui.vertical(|ui| {
                            ui.label("🎯 Servizio da modificare");
                            ui.add(egui::TextEdit::singleline(&mut self.edit_service_name)
                                .hint_text("Nome del servizio esistente")
                                .min_size(egui::vec2(230.0, 25.0)));
                            ui.add_space(10.0);
                            
                            ui.label("👤 Nuovo username (opzionale)");
                            ui.add(egui::TextEdit::singleline(&mut self.edit_new_username)
                                .hint_text("Lascia vuoto per non modificare")
                                .min_size(egui::vec2(230.0, 25.0)));
                            ui.add_space(10.0);
                            
                            ui.label("🔑 Nuova password");
                            ui.add(egui::TextEdit::singleline(&mut self.edit_new_password)
                                .password(!self.show_password)
                                .hint_text("Nuova password sicura")
                                .min_size(egui::vec2(230.0, 25.0)));
                            ui.checkbox(&mut self.show_password, "Mostra");
                            ui.add_space(10.0);
                            
                            ui.label("🔑 Conferma password");
                            ui.add(egui::TextEdit::singleline(&mut self.edit_confirm_password)
                                .password(!self.show_password1)
                                .hint_text("Ripeti la nuova password")
                                .min_size(egui::vec2(230.0, 25.0)));
                            ui.checkbox(&mut self.show_password1, "Mostra");
                            ui.add_space(15.0);
                        });
                        
                        if ui.add_sized([230.0, 35.0], 
                            egui::Button::new("🔄 Modifica Password")).clicked() {
                            self.show_password = false;
                            self.show_password1 = false;
                            self.edit_password();
                        }
                    });
                });
        });
    }
}