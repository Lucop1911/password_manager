use crate::PasswordManagerApp;
use eframe::egui;

impl PasswordManagerApp {
    pub fn show_login(&mut self, ui: &mut egui::Ui) {
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
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.login_username)
                                        .desired_width(200.0),
                                );
                                ui.end_row();

                                ui.label("🔑 Password:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.login_password)
                                        .password(!self.show_password)
                                        .desired_width(200.0),
                                );
                                ui.checkbox(&mut self.show_password, "Mostra");
                                ui.end_row();
                            });

                        ui.add_space(20.0);

                        if ui
                            .add_sized([100.0, 35.0], egui::Button::new("Accedi"))
                            .clicked()
                        {
                            self.show_password = false;
                            self.show_password1 = false;
                            self.handle_login();
                        }
                    });
                });
        });
    }
}
