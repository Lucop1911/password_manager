#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod helpers;
mod pages;

use crate::helpers::utils::{AppData, PasswordEntry, UserData, load_data};
use eframe::egui;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(PartialEq)]
pub enum AppState {
    Registration,
    Login,
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

    // Dati dell'app
    pub app_data: AppData,
    pub current_user: Option<UserData>,

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

    // Mostra password temporaneamente (indice -> (password, tempo_inizio))
    pub shown_passwords: HashMap<usize, (String, Instant)>,

    // Tab attivo (0 = Aggiungi, 1 = Modifica)
    pub active_tab: usize,

    pub show_password: bool,
    pub show_password1: bool,

    pub show_popup_add: bool,
    pub show_popup_edit: bool,
    pub show_popup_reg: bool,
}

impl Default for PasswordManagerApp {
    fn default() -> Self {
        let app_data = load_data();
        let state = if app_data.user.is_some() {
            AppState::Login
        } else {
            AppState::Registration
        };

        let dark_mode = app_data.dark_mode.unwrap_or(true);

        Self {
            state,
            reg_username: String::new(),
            reg_password: String::new(),
            reg_confirm_password: String::new(),
            login_username: String::new(),
            login_password: String::new(),
            app_data,
            current_user: None,
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
            show_popup_reg: false,
        }
    }
}

impl eframe::App for PasswordManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        // Pulizia delle password mostrate dopo 10 secondi
        let now = Instant::now();
        let expired_keys: Vec<usize> = self
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

        // Per migliorare la performance e possibili bug:
        // 1: Aggiorno la GUI solo se sono nella pagina Main
        // 2: Aggiorno la GUI solo se non ci sono password "scoperte" e quindi non devo tenere il timer aggiornato
        if !self.shown_passwords.is_empty() && self.state == AppState::Main {
            ctx.request_repaint_after(Duration::from_secs(1));
        }

        // Temi
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

            ctx.set_visuals(visuals);
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

            ctx.set_visuals(visuals);
        }

        let mut style = (*ctx.style()).clone();
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
        ctx.set_style(style);

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("🔐 Password Manager");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let theme_text = if self.dark_mode { "🌙" } else { "☀" };

                    if ui.button(theme_text).on_hover_text("Cambia tema").clicked() {
                        self.toggle_theme();
                    }

                    if self.state == AppState::Login || self.state == AppState::Login {
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
            egui::TopBottomPanel::bottom("messages").show(ctx, |ui| {
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);

            match self.state {
                AppState::Registration => self.show_registration(ui),
                AppState::Login => self.show_login(ui),
                AppState::Main => self.show_main(ctx, ui),
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