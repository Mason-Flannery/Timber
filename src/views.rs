use chrono::{DateTime, Local, Utc};
use std::{collections::HashMap, fmt};

use rusqlite::Connection;

use crate::{db, models::Session, utils};
#[derive(Debug)]
pub struct SessionView {
    pub session: Session,
    pub client_name: String,
}
impl SessionView {
    pub fn from_session(conn: &Connection, session: Session) -> Result<Self, rusqlite::Error> {
        let client = db::get_client_by_id(conn, session.client_id)?;
        Ok(SessionView {
            session,
            client_name: client.name,
        })
    }
}

impl std::fmt::Display for SessionView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start_utc = self
            .session
            .start_timestamp
            .parse::<DateTime<Utc>>()
            .unwrap_or_else(|_| Utc::now());
        let start_local = start_utc.with_timezone(&Local);

        let end_str = match &self.session.end_timestamp {
            Some(end_ts) => {
                let end_utc = end_ts
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now());
                let end_local = end_utc.with_timezone(&Local);
                format!("{}", end_local.format("%b %d, %Y %I:%M %p"))
            }
            None => "In progress".to_string(),
        };

        let (hours, minutes) =
            utils::split_minutes(self.session.get_timedelta().num_minutes() as u32);
        let duration_str = format!("Duration: {hours}h {minutes}m");

        let note_str = match &self.session.note {
            Some(note) => format!("\nNote: {note}"),
            None => "".to_string(),
        };

        write!(
            f,
            "Session for client '{}'\nStart: {}\nEnd: {}\n{}{}",
            self.client_name,
            start_local.format("%b %d, %Y %I:%M %p"),
            end_str,
            duration_str,
            note_str,
        )
    }
}

pub fn display_client_time_summaries(
    conn: &Connection,
    start: &DateTime<Utc>,
    end: &DateTime<Utc>,
) {
    let results = db::get_sessions_within_range(conn, start, end)
        .expect("An error occurred while fetching the daily sessions");
    let mut client_totals: HashMap<i32, i64> = HashMap::new();
    for result in results {
        if client_totals.contains_key(&result.client_id) {
            client_totals.insert(
                result.client_id,
                client_totals.get(&result.client_id).unwrap()
                    + result.get_timedelta().num_minutes(),
            );
        } else {
            client_totals.insert(result.client_id, result.get_timedelta().num_minutes());
        }
    }
    client_totals.iter().for_each(|(&key, &value)| {
        let (hours, minutes) = utils::split_minutes(value as u32);
        println!(
            "{}:\n{}h {}m\n",
            db::get_client_by_id(conn, key).unwrap().name,
            hours,
            minutes
        )
    });

    let (hours, minutes) = utils::split_minutes(client_totals.values().sum::<i64>() as u32);
    println!("Total: {hours}h {minutes}m");
}

pub fn display_sessions(conn: &Connection, sessions: Vec<Session>) {
    for session in sessions {
        if let Ok(view) = SessionView::from_session(conn, session) {
            println!("\n{view}");
        } else {
            println!("Error displaying session.");
        }
    }
}

pub fn display_active_session(conn: &Connection) {
    match db::get_active_session(conn) {
        Ok(Some(session)) => {
            display_sessions(conn, vec![session]);
        }
        Ok(None) => {
            println!("No active session found!")
        }
        Err(_) => println!("An error occurred"),
    }
}
