use chrono::{TimeDelta, Utc};
use rusqlite::Connection;

use crate::{db, models::Session};

pub fn end_session(conn: &Connection) -> Result<Option<TimeDelta>, rusqlite::Error> {
    match db::get_active_session(conn) {
        Ok(Some(mut session)) => {
            session.end_timestamp = Some(Utc::now().to_rfc3339());
            let _ = db::commit_session_changes(conn, &session);
            let delta = session.get_timedelta();
            Ok(delta)
        }
        Ok(None) => Ok(None),
        Err(err) => Err(err),
    }
}

pub fn patch_session(conn: &Connection, offset: i32) -> Result<Option<()>, rusqlite::Error> {
    match db::get_active_session(conn) {
        Ok(Some(mut session)) => {
            session.offset_minutes += offset;
            let _ = db::commit_session_changes(conn, &session);
            Ok(Some(()))
        },
        Ok(None) => {Ok(None)}
        Err(e) => Err(e),
    }
}