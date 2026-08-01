use crate::PasswordManagerApp;
use eframe::egui;

impl PasswordManagerApp {
    pub fn show_main(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // Two-column layout: fixed-width left panel for add/edit forms,
        // right side for the password list
        ui.allocate_ui_with_layout(
            ui.available_size(),
            egui::Layout::left_to_right(egui::Align::TOP),
            |ui| {
                // Left panel with tabs
                ui.vertical(|ui| {
                    ui.set_min_width(360.0);
                    ui.set_max_width(360.0);

                    // Tab selector
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.active_tab, 0, "➕ Add");
                        ui.selectable_value(&mut self.active_tab, 1, "⚙ Edit");
                    });
                    ui.add_space(10.0);

                    // Scrollable content
                    egui::ScrollArea::vertical()
                        .id_salt("left_panel_scroll")
                        .auto_shrink([false, true])
                        .show(ui, |ui| match self.active_tab {
                            0 => self.show_add_password_panel(ui),
                            1 => self.show_edit_password_panel(ui),
                            _ => {}
                        });
                });

                ui.separator();

                // Right panel (password list)
                self.show_password_list(ctx, ui);
            },
        );
    }
}
