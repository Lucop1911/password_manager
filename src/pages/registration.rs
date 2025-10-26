use eframe::egui;
use crate::PasswordManagerApp;

impl PasswordManagerApp {
    pub fn show_registration(&mut self, ui: &mut egui::Ui) {
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
                                    .password(!self.show_password)
                                    .desired_width(200.0));
                                ui.checkbox(&mut self.show_password, "Mostra");
                                ui.end_row();
                                
                                ui.label("🔑 Conferma:");
                                ui.add(egui::TextEdit::singleline(&mut self.reg_confirm_password)
                                    .password(!self.show_password1)
                                    .desired_width(200.0));
                                ui.checkbox(&mut self.show_password1, "Mostra");
                                ui.end_row();
                            });
                        
                        ui.add_space(15.0);
                        ui.small("💡 La password deve essere di almeno 6 caratteri");
                        ui.add_space(15.0);
                        
                        if ui.add_sized([120.0, 35.0], egui::Button::new("Registrati")).clicked() {
                            self.show_password = false;
                            self.show_password1 = false;
                            self.handle_registration();
                        }
                    });
                });
        });
    }
}