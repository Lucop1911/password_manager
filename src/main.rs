#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod helpers;
mod pages;

use crate::helpers::db::database::Database;
use crate::helpers::db::keyfile::Keyfile;
use crate::helpers::db::migration;
use crate::helpers::utils::{PasswordEntry, UserData};
use eframe::egui;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// App lifecycle: a keyfile in the data directory marks the presence of an
/// account; a leftover `data.json` triggers the one-time migration flow.
#[derive(PartialEq)]
pub enum AppState {
    Registration,
    Login,
    Migration,
    Main,
}

pub struct PasswordManagerApp {
    pub state: AppState,

    // Campi per registrazione/login
    pub reg_username: String,
    pub reg_password: String,
    pub reg_confirm_password: String,

    pub login_username: String,
    pub login_password: String,

    // Background login: the Argon2id derivation runs on a worker thread so
    // the UI can show a spinner instead of freezing
    pub login_busy: bool,
    pub login_rx: Option<std::sync::mpsc::Receiver<Result<crate::helpers::handlers::LoginResult, String>>>,

    // Campo per la migrazione dal vecchio formato
    pub migration_password: String,

    // Session state: the DB connection stays open only while logged in
    pub db: Option<Database>,
    pub current_user: Option<UserData>,
    pub ps: Vec<PasswordEntry>,

    // Master-password-derived AES key, kept in memory for the session only
    pub encryption_key: Option<[u8; 32]>,

    // Campi per aggiungere password
    pub new_entry_name: String,
    pub new_entry_username: String,
    pub new_entry_password: String,

    // Campi per modificare password
    pub edit_service_name: String,
    pub edit_new_username: String,
    pub edit_new_password: String,
    pub edit_confirm_password: String,

    // Messaggi di errore/successo
    pub message: String,
    pub message_color: egui::Color32,

    // Tema
    pub dark_mode: bool,

    // Ricerca
    pub search_query: String,

    // Mostra password temporaneamente (id -> (password, tempo_inizio))
    pub shown_passwords: HashMap<i64, (String, Instant)>,

    // Tab attivo (0 = Aggiungi, 1 = Modifica)
    pub active_tab: usize,

    // Booleans per i checkbox mostra password
    pub show_password: bool,
    pub show_password1: bool,

    // Booleans per i popup
    pub show_popup_add: bool,
    pub show_popup_edit: bool,
}

impl Default for PasswordManagerApp {
    fn default() -> Self {
        // Bootstrap: keyfile present -> account exists; legacy data.json
        // present (without keyfile) -> offer migration; otherwise register.
        let keyfile = Keyfile::load().ok().flatten();
        let legacy = migration::legacy_exists();

        let (state, dark_mode) = if let Some(keyfile) = &keyfile {
            (AppState::Login, keyfile.dark_mode)
        } else if legacy && migration::legacy_has_account() {
            (AppState::Migration, true)
        } else {
            if legacy {
                let _ = migration::remove_legacy();
            }
            (AppState::Registration, true)
        };

        Self {
            state,
            reg_username: String::new(),
            reg_password: String::new(),
            reg_confirm_password: String::new(),
            login_username: String::new(),
            login_password: String::new(),
            login_busy: false,
            login_rx: None,
            migration_password: String::new(),
            db: None,
            current_user: None,
            ps: Vec::new(),
            encryption_key: None,
            new_entry_name: String::new(),
            new_entry_username: String::new(),
            new_entry_password: String::new(),
            edit_service_name: String::new(),
            edit_new_username: String::new(),
            edit_new_password: String::new(),
            edit_confirm_password: String::new(),
            message: String::new(),
            message_color: egui::Color32::GREEN,
            dark_mode,
            search_query: String::new(),
            shown_passwords: HashMap::new(),
            active_tab: 0,
            show_password: false,
            show_password1: false,
            show_popup_add: false,
            show_popup_edit: false,
        }
    }
}

impl eframe::App for PasswordManagerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Revealed passwords expire after 10 seconds; keep repainting
        // while any reveal is still pending so the countdown stays fresh.
        let now = Instant::now();
        let expired_keys: Vec<i64> = self
            .shown_passwords
            .iter()
            .filter(|(_, (_, start_time))| {
                now.duration_since(*start_time) > Duration::from_secs(10)
            })
            .map(|(&key, _)| key)
            .collect();

        for key in expired_keys {
            self.shown_passwords.remove(&key);
        }
        if !self.shown_passwords.is_empty() && self.state == AppState::Main {
            ctx.request_repaint_after(Duration::from_secs(1));
        }

        // Temi
        let theme = if self.dark_mode {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        };
        ctx.set_theme(theme);

        if self.dark_mode {
            let mut visuals = egui::Visuals::dark();
            // Dark theme improvements
            visuals.window_fill = egui::Color32::from_rgb(20, 20, 25);
            visuals.panel_fill = egui::Color32::from_rgb(25, 25, 30);
            visuals.faint_bg_color = egui::Color32::from_rgb(35, 35, 42);
            visuals.extreme_bg_color = egui::Color32::from_rgb(15, 15, 18);

            // Widget colors
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(40, 40, 48);
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(50, 50, 60);
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 60, 72);
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(70, 70, 85);

            // Text colors
            visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(200, 200, 210);
            visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(220, 220, 230);
            visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(240, 240, 250);
            visuals.widgets.active.fg_stroke.color = egui::Color32::WHITE;

            ctx.set_visuals_of(theme, visuals);
        } else {
            let mut visuals = egui::Visuals::light();
            // Light theme improvements - much better contrast
            visuals.window_fill = egui::Color32::from_rgb(250, 250, 252);
            visuals.panel_fill = egui::Color32::from_rgb(245, 245, 248);
            visuals.faint_bg_color = egui::Color32::from_rgb(235, 235, 240);
            visuals.extreme_bg_color = egui::Color32::WHITE;

            // Widget colors with better contrast
            visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(240, 240, 245);
            visuals.widgets.noninteractive.weak_bg_fill = egui::Color32::from_rgb(245, 245, 250);
            visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(225, 225, 235);
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(210, 210, 225);
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(190, 190, 210);

            // Text colors - dark text on light background
            visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(40, 40, 50);
            visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(30, 30, 40);
            visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(20, 20, 30);
            visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(10, 10, 20);

            // Stroke colors for borders
            visuals.widgets.noninteractive.bg_stroke.color = egui::Color32::from_rgb(200, 200, 210);
            visuals.widgets.inactive.bg_stroke.color = egui::Color32::from_rgb(180, 180, 195);
            visuals.widgets.hovered.bg_stroke.color = egui::Color32::from_rgb(160, 160, 180);
            visuals.widgets.active.bg_stroke.color = egui::Color32::from_rgb(140, 140, 165);

            // Override text color globally
            visuals.override_text_color = Some(egui::Color32::from_rgb(25, 25, 35));

            ctx.set_visuals_of(theme, visuals);
        }

        let mut style = (*ctx.style_of(theme)).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 10.0);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.spacing.indent = 20.0;
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::new(26.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(14.0, egui::FontFamily::Monospace),
            ),
            (
                egui::TextStyle::Small,
                egui::FontId::new(13.0, egui::FontFamily::Proportional),
            ),
        ]
        .into();
        ctx.set_style_of(theme, style);

        egui::Panel::top("header").show(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("🔐 Password Manager");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let theme_text = if self.dark_mode { "🌙" } else { "☀" };

                    if ui.button(theme_text).on_hover_text("Toggle theme").clicked() {
                        self.toggle_theme();
                    }

                    if self.state == AppState::Registration
                        || self.state == AppState::Login
                        || self.state == AppState::Migration
                    {
                        ui.separator();
                        if ui.button("🚪 Exit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }

                    if self.state == AppState::Main {
                        ui.separator();
                        if ui.button("🚪 Log out").clicked() {
                            self.show_password = false;
                            self.show_password1 = false;
                            self.logout();
                            return;
                        }
                        ui.label(format!("👤 {}", self.current_user.as_ref().unwrap().u));
                    }
                });
            });
            ui.add_space(4.0);
        });

        if !self.message.is_empty() {
            egui::Panel::bottom("messages").show(ui, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let icon = match self.message_color {
                        egui::Color32::RED => "❌",
                        egui::Color32::GREEN => "✅",
                        egui::Color32::YELLOW => "🔸",
                        _ => "🔸",
                    };
                    ui.colored_label(self.message_color, format!("{} {}", icon, &self.message));
                });
                ui.add_space(8.0);
            });
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(20.0);

            match self.state {
                AppState::Registration => self.show_registration(ui),
                AppState::Login => self.show_login(ui),
                AppState::Migration => self.show_migration(ui),
                AppState::Main => self.show_main(&ctx, ui),
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    eframe::run_native(
        "Password Manager",
        options,
        Box::new(|_cc| Ok(Box::new(PasswordManagerApp::default()))),
    )
}