use crate::PasswordManagerApp;
use eframe::egui;
use std::sync::mpsc;
use std::time::Duration;

impl PasswordManagerApp {
    pub fn show_login(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);

            ui.heading("Welcome back!");
            ui.add_space(10.0);
            ui.label("Enter your credentials to sign in.");
            ui.add_space(40.0);

            egui::Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .corner_radius(8.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.set_max_width(400.0);

                    ui.vertical_centered_justified(|ui| {
                        ui.label("🔓 Sign In");
                        ui.add_space(15.0);

                        // Set to true when Enter is pressed in a field
                        let mut submit = false;

                        egui::Grid::new("login_grid")
                            .num_columns(2)
                            .spacing([10.0, 15.0])
                            .show(ui, |ui| {
                                ui.label("👤 Username:");
                                let username_response = ui.add(
                                    egui::TextEdit::singleline(&mut self.login_username)
                                        .desired_width(200.0),
                                );
                                ui.end_row();

                                // Password is masked unless the user opts to
                                // preview it via the "Show" checkbox
                                ui.label("🔑 Password:");
                                let password_response = ui.add(
                                    egui::TextEdit::singleline(&mut self.login_password)
                                        .password(!self.show_password)
                                        .desired_width(200.0),
                                );
                                ui.checkbox(&mut self.show_password, "Show");
                                ui.end_row();

                                // Pressing Enter in either field submits the
                                // form, so the mouse is not needed. Note:
                                // egui surrenders focus immediately when Enter
                                // is pressed in a single-line TextEdit, so
                                // `has_focus()` is already false here; the
                                // documented pattern is `lost_focus()`.
                                let enter_pressed =
                                    ui.input(|i| i.key_pressed(egui::Key::Enter));
                                if enter_pressed
                                    && (username_response.lost_focus()
                                        || password_response.lost_focus())
                                {
                                    submit = true;
                                }
                            });

                        ui.add_space(20.0);

                        // Login runs on a background thread (Argon2id key
                        // derivation can take a moment); meanwhile a spinner
                        // keeps the UI responsive and informed.
                        let login_clicked = !self.login_busy
                            && ui
                                .add_sized([100.0, 35.0], egui::Button::new("Sign In"))
                                .clicked();

                        if (submit || login_clicked) && !self.login_busy {
                            self.show_password = false;
                            self.show_password1 = false;
                            self.start_login();
                        }

                        if self.login_busy {
                            ui.add_space(12.0);
                            ui.horizontal(|ui| {
                                ui.add(egui::Spinner::new().size(16.0));
                                ui.label("Signing in…");
                            });
                        }

                        // Collect the result of the background login attempt
                        if let Some(rx) = &self.login_rx {
                            match rx.try_recv() {
                                Ok(result) => {
                                    self.login_busy = false;
                                    self.login_rx = None;
                                    match result {
                                        Ok(login) => {
                                            self.encryption_key = Some(login.key);
                                            self.current_user = Some(login.user);
                                            self.ps = login.entries;
                                            self.dark_mode = login.dark_mode;
                                            self.db = Some(login.db);
                                            self.state = crate::AppState::Main;
                                            match login.warning {
                                                Some(warning) => {
                                                    self.message = warning;
                                                    self.message_color =
                                                        egui::Color32::YELLOW;
                                                }
                                                None => {
                                                    self.message =
                                                        "Signed in successfully!".to_string();
                                                    self.message_color =
                                                        egui::Color32::GREEN;
                                                }
                                            }
                                            self.login_username.clear();
                                            self.login_password.clear();
                                        }
                                        Err(e) => {
                                            self.message = e;
                                            self.message_color = egui::Color32::RED;
                                        }
                                    }
                                }
                                Err(mpsc::TryRecvError::Empty) => {
                                    // Keep polling while the thread runs
                                    ui.ctx().request_repaint_after(Duration::from_millis(50));
                                }
                                Err(_) => {
                                    // Thread panicked or disconnected
                                    self.login_busy = false;
                                    self.login_rx = None;
                                    self.message =
                                        "Unexpected error during login!".to_string();
                                    self.message_color = egui::Color32::RED;
                                }
                            }
                        }
                    });
                });
        });
    }

    /// Spawns a background thread that performs the (potentially slow)
    /// Argon2id derivation and database unlock; the result is collected on
    /// the next frame via `login_rx`.
    fn start_login(&mut self) {
        self.login_busy = true;
        let (tx, rx) = mpsc::channel();
        let username = self.login_username.clone();
        let password = self.login_password.clone();
        std::thread::spawn(move || {
            let _ = tx.send(crate::helpers::handlers::perform_login(&username, &password));
        });
        self.login_rx = Some(rx);
    }
}
