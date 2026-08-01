use crate::PasswordManagerApp;
use crate::helpers::db::migration;
use eframe::egui;

impl PasswordManagerApp {
    pub fn show_migration(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);

            ui.heading("🔄 Data Migration");
            ui.add_space(10.0);
            ui.label(
                "A legacy data file was detected. Enter your master password to migrate your data into the new encrypted database.",
            );
            ui.add_space(10.0);

            if let Some(username) = migration::legacy_username() {
                ui.label(format!("👤 Account found: {}", username));
            }

            ui.add_space(30.0);

            egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(8.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.set_max_width(400.0);

                    ui.vertical_centered_justified(|ui| {
                        ui.label("🔓 Migration");
                        ui.add_space(15.0);

                        egui::Grid::new("migration_grid")
                            .num_columns(2)
                            .spacing([10.0, 15.0])
                            .show(ui, |ui| {
                                ui.label("🔑 Master password:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.migration_password)
                                        .password(!self.show_password)
                                        .desired_width(200.0),
                                );
                                ui.checkbox(&mut self.show_password, "Show");
                                ui.end_row();
                            });

                        ui.add_space(20.0);

                        if ui
                            .add_sized([180.0, 35.0], egui::Button::new("🔄 Migrate Data"))
                            .clicked()
                        {
                            self.show_password = false;
                            self.show_password1 = false;
                            self.handle_migration();
                        }
                    });
                });
        });
    }
}
