use eframe::egui;
use crate::PasswordManagerApp;

impl PasswordManagerApp {
    pub fn show_add_password_panel(&mut self, ui: &mut egui::Ui) {
        ui.push_id("add_password_panel", |ui| {
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
                                .password(!self.show_password)
                                .hint_text("password sicura")
                                .min_size(egui::vec2(230.0, 25.0)));
                            ui.checkbox(&mut self.show_password, "Mostra");
                            ui.add_space(15.0);
                        });
                        
                        if ui.add_sized([230.0, 35.0], 
                            egui::Button::new("💾 Salva Password")).clicked() {
                            self.show_password = false;
                            self.show_password1 = false;
                            self.add_password();
                        }
                    });
                });
        });
    }
}