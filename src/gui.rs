use crate::{
    commands, config::Config, db, models::{Client, Session}, utils, views::SessionView
};
use chrono::{Datelike, TimeZone, Utc};
use eframe::egui;
use rusqlite::Connection;

struct TimberApp {
    conn: Connection, // direct DB connection
    clients: Vec<Client>,
    current_session: Option<SessionView>,
    new_client_name: String,
    status_message: String,
}

impl TimberApp {
    fn refresh_clients(&mut self) {
        self.clients = db::list_clients(&self.conn).unwrap_or_default();
    }

    fn refresh_current_session(&mut self) {
        match db::get_active_session(&self.conn).expect("645") {
            Some(session) => self.current_session = Some(SessionView::from_session(&self.conn, session).expect("Should be fine")),
            None => self.current_session = Option::None,
        }
    }
}

impl Default for TimberApp {
    fn default() -> Self {
        let conn = db::init_db(&Config::default());

        let mut app = Self {
            conn,
            clients: vec![],
            current_session: None,
            new_client_name: String::new(),
            status_message: String::new(),
        };

        app.refresh_clients();
        app.refresh_current_session();
        app
    }
}

impl eframe::App for TimberApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🌲 Timber Time Tracker");

            ui.separator();

            // CLIENT LIST
            ui.horizontal(|ui| {
                ui.label("Clients:");
                if ui.button("➕ New").clicked() {
                    if !self.new_client_name.trim().is_empty() {
                        match db::store_client(
                            &self.conn,
                            &Client {
                                name: self.new_client_name.clone(),
                                id: 0,
                                note: Option::None,
                            },
                        ) {
                            Ok(_) => {
                                self.status_message =
                                    format!("Added client: {}", self.new_client_name);
                                self.new_client_name.clear();
                                self.refresh_clients();
                            }
                            Err(_) => self.status_message = "Failed to add client".into(),
                        }
                    }
                }
                ui.text_edit_singleline(&mut self.new_client_name);
            });

            egui::ScrollArea::vertical()
                .max_height(100.0)
                .show(ui, |ui| {
                    for client in &self.clients {
                        ui.label(format!("• {} (id {})", client.name, client.id));
                    }
                });

            ui.separator();

            // SESSION CONTROLS
            ui.heading("Session");
            match &self.current_session {
                Some(session) => {
                    let elapsed = session.session
                        .get_timedelta();
                    let (hours, minutes) = utils::split_minutes(elapsed.num_minutes() as u32);

                    ui.label(format!(
                        "Active session for client {}: {}h {}m",
                        session.client_name, hours, minutes
                    ));
                    if ui.button("⏹ Stop Session").clicked() {
                        if let Err(_) = commands::session::end_session(&self.conn) {
                            self.status_message = "Failed to stop session".into();
                        } else {
                            self.status_message = "Stopped current session".into();
                            self.refresh_current_session();
                        }
                    }
                }
                None => {
                    ui.label("No active session");
                    if ui.button("▶ Start Session (first client)").clicked() {
                        if let Some(first) = self.clients.first() {
                            if let Err(_) = db::store_session(&self.conn, &Session {
                                id: 0,
                                client_id: first.id,
                                start_timestamp: Utc::now().to_rfc3339(),
                                end_timestamp: Option::None,
                                note: Option::None,
                                offset_minutes: 0
                            }) {
                                self.status_message = "Failed to start session".into();
                            } else {
                                self.status_message = format!("Started session for {}", first.name);
                                self.refresh_current_session();
                            }
                        }
                    }
                }
            }

            ui.separator();

            // DAILY TOTALS
            ui.heading("Daily Totals");
            let now: chrono::DateTime<Utc> = chrono::Utc::now();
            let start = chrono::Utc
                .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
                .unwrap(); // Get time at start of day
            let end = start + chrono::Duration::days(1); // Get time at end of day

            let sessions_today = db::get_sessions_within_range(&self.conn, &start, &end)
                .expect("An error occurred while fetching the daily sessions");

            let mut totals: std::collections::HashMap<i32, i64> = std::collections::HashMap::new();
            for s in sessions_today {
                if let delta = s.get_timedelta() {
                    *totals.entry(s.client_id).or_insert(0) += delta.num_minutes();
                }
            }

            for (&cid, &minutes) in &totals {
                let client_name = db::get_client_by_id(&self.conn, cid)
                    .map(|c| c.name)
                    .unwrap_or_else(|_| "Unknown".into());
                let (h, m) = utils::split_minutes(minutes as u32);
                ui.label(format!("{}: {}h {}m", client_name, h, m));
            }

            let total_minutes: i64 = totals.values().sum();
            let (h, m) = utils::split_minutes(total_minutes as u32);
            ui.label(format!("Total: {}h {}m", h, m));

            ui.separator();

            if !self.status_message.is_empty() {
                ui.colored_label(egui::Color32::LIGHT_GREEN, &self.status_message);
            }
        });
    }
}

// Main function
pub fn main(conn: Connection) -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    let mut app = Box::new(TimberApp::default());

    app.conn = conn; // Replace database connection
    eframe::run_native(
        "Timber",
        options,
        Box::new(|_cc| Ok(Box::new(TimberApp::default()))),
    )
}
