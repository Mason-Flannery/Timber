use std::fmt;
use chrono::{DateTime, Local, Utc};

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
            client_name: client.name
        })
    }
}

impl std::fmt::Display for SessionView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start_utc = self.session.start_timestamp.parse::<DateTime<Utc>>()
            .unwrap_or_else(|_| Utc::now());
        let start_local = start_utc.with_timezone(&Local);

        let end_str = match &self.session.end_timestamp {
            Some(end_ts) => {
                let end_utc = end_ts.parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now());
                let end_local = end_utc.with_timezone(&Local);
                format!("{}", end_local.format("%b %d, %Y %I:%M %p"))
            },
            None => "In progress".to_string(),
        };

        let duration_str = match self.session.get_timedelta() {
            Some(delta) => {
                let (hours, minutes) = utils::split_minutes(delta.num_minutes() as u32);
                format!("Duration: {hours}h {minutes}m")
            },
            None => "".to_string(),
        };

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
