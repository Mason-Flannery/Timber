use chrono::{DateTime, TimeDelta, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result};

use crate::models::{Client, Session};

pub fn init_db() -> Connection {
    let conn = Connection::open("timber.db").expect("No issues");
    init_schema(&conn);
    conn // we assume conn is always valid here
}

pub fn init_schema(conn: &Connection) {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS clients (
            id    INTEGER PRIMARY KEY AUTOINCREMENT,
            name  TEXT NOT NULL UNIQUE,
            note  TEXT
        );
        
        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            client_id INTEGER NOT NULL,
            start_timestamp TEXT NOT NULL,
            end_timestamp TEXT, 
            note TEXT,
            FOREIGN KEY (client_id) REFERENCES clients(id)
        );",
    )
    .expect("Scema init failed");
}

pub fn store_session(conn: &Connection, session: &Session) -> Result<i32, rusqlite::Error> {
    let stmt = conn.execute(
        "INSERT INTO sessions (client_id, start_timestamp, end_timestamp, note) VALUES (?1, ?2, ?3, ?4)",
        params![session.client_id, session.start_timestamp, session.end_timestamp, session.note],
    )?;
    Ok(conn.last_insert_rowid() as i32)
}

pub fn get_session_by_id(conn: &Connection, id: i32) -> Result<Session, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, client_id, start_timestamp, end_timestamp, note FROM sessions WHERE id = ?1 LIMIT 1")?;

    stmt.query_row(params![id], |row| {
        Ok(Session {
            id: row.get(0)?,
            client_id: row.get(1)?,
            start_timestamp: row.get(2)?,
            end_timestamp: row.get::<_, Option<String>>(3)?,
            note: row.get::<_, Option<String>>(4)?,
        })
    })
}

pub fn get_session_id_by_name(conn: &Connection, name: String) -> Result<Option<i32>, rusqlite::Error> {
    conn.query_row("SELECT id FROM sessions WHERE name = ?1", [name], |row| row.get(0)).optional()
}

pub fn remove_session(conn: &Connection, id: i32) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM sessions WHERE id == ?1", [id])?;
    Ok(())
}

pub fn list_sessions(
    conn: &Connection,
    client_id: Option<i32>,
) -> Result<Vec<Session>, rusqlite::Error> {
    if let Some(id) = client_id {
        let mut stmt = conn.prepare(
            "SELECT id, 
        client_id, 
        start_timestamp, 
        end_timestamp, 
        note FROM sessions WHERE client_id = ?1 ORDER BY start_timestamp DESC",
        )?;
        let session_iter = stmt.query_map([id], |row| {
            Ok(Session {
                id: row.get(0)?,
                client_id: row.get(1)?,
                start_timestamp: row.get(2)?,
                end_timestamp: row.get(3)?,
                note: row.get(4)?,
            })
        })?;
        return Ok(session_iter.collect::<Result<Vec<Session>, _>>()?);
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, 
        client_id, 
        start_timestamp, 
        end_timestamp, 
        note FROM sessions ORDER BY start_timestamp DESC",
        )?;

        let session_iter = stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                client_id: row.get(1)?,
                start_timestamp: row.get(2)?,
                end_timestamp: row.get(3)?,
                note: row.get(4)?,
            })
        })?;
        return Ok(session_iter.collect::<Result<Vec<Session>, _>>()?);
    };
}

pub fn store_client(conn: &Connection, client: &Client) -> Result<Option<i32>, rusqlite::Error> {
    match conn.execute(
        "INSERT INTO clients (name, note) VALUES (?1, ?2)",
        params![client.name, client.note],
    ) {
        Ok(_) => (),
        Err(rusqlite::Error::SqliteFailure(err, _))
            if err.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            println!("Client already exists! Insert ignored.");
            return Ok(Option::None)
        }
        Err(e) => return Err(e.into()),
    };
    Ok(Some(conn.last_insert_rowid() as i32)) // ! Do we want to return this if an error is encountered above?
}

pub fn get_client_by_id(conn: &Connection, id: i32) -> Result<Client, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, name, note FROM clients WHERE id = ?1 LIMIT 1")?;

    stmt.query_row(params![id], |row| {
        Ok(Client {
            id: row.get(0)?,
            name: row.get(1)?,
            note: row.get::<_, Option<String>>(2)?,
        })
    })
}

pub fn get_client_id_by_name(conn: &Connection, name: String) -> Result<Option<i32>, rusqlite::Error> {
    conn.query_row("SELECT id FROM clients WHERE name = ?1", [name], |row| row.get(0)).optional()
}

pub fn remove_client(conn: &Connection, id: i32) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM clients WHERE id == ?1", [id])?;
    Ok(())
}

pub fn list_clients(conn: &Connection) -> Result<Vec<Client>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, name, note FROM clients")?;

    let clients_iter = stmt.query_map([], |row| {
        Ok(Client {
            id: row.get(0)?,
            name: row.get(1)?,
            note: row.get::<_, Option<String>>(2)?,
        })
    })?;
    Ok(clients_iter.collect::<Result<Vec<Client>, _>>()?)
}

pub fn get_active_session(conn: &Connection) -> Result<Option<Session>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, client_id, start_timestamp, end_timestamp, note
         FROM sessions
         WHERE end_timestamp IS NULL
         ORDER BY start_timestamp DESC
         LIMIT 1",
    )?;

    let result = stmt.query_row([], |row| {
        Ok(Session {
            id: row.get(0)?,
            client_id: row.get(1)?,
            start_timestamp: row.get(2)?,
            end_timestamp: None,
            note: row.get(4)?,
        })
    });

    match result {
        Ok(session) => Ok(Some(session)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn end_session(conn: &Connection) -> Result<Option<TimeDelta>, rusqlite::Error> {
    match get_active_session(&conn) {
        Ok(Some(mut session)) => {
            session.end_timestamp = Some(Utc::now().to_rfc3339());
            let _ = commit_session_changes(&conn, &session);
            let delta = session.get_timedelta();
            Ok(delta)
        }
        Ok(None) => Ok(None),
        Err(err) => Err(err),
    }
}

fn commit_session_changes(conn: &Connection, session: &Session) -> Result<(), rusqlite::Error> {
    match conn.execute(
        "UPDATE sessions
        SET client_id=?1, start_timestamp=?2, end_timestamp=?3, note=?4
        WHERE id=?5",
        params![
            session.client_id,
            session.start_timestamp,
            session.end_timestamp,
            session.note,
            session.id
        ],
    ) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

fn commit_client_changes(conn: &Connection, client: &Client) -> Result<(), rusqlite::Error> {
    match conn.execute(
        "UPDATE clients
        SET id=?1, name=?2, note=?3
        WHERE id=?1",
        params![client.id, client.name, client.note],
    ) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

pub fn get_sessions_within_range(conn: &Connection, start: &DateTime<Utc>, end: &DateTime<Utc>) -> Result<Vec<Session>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, client_id, start_timestamp, end_timestamp, note
         FROM sessions
         WHERE start_timestamp >= ?1 AND start_timestamp <= ?2
         ORDER BY start_timestamp ASC"
    )?;

    let sessions = stmt.query_map([start.to_rfc3339(), end.to_rfc3339()], |row| {
        Ok(Session {
            id: row.get(0)?,
            client_id: row.get(1)?,
            start_timestamp: row.get(2)?,
            end_timestamp: row.get(3)?,
            note: row.get(4)?,
        })
    })?.collect::<Result<Vec<_>>>()?;

    Ok(sessions)
}

// TESTS

#[test]
fn test_store_client() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    init_schema(&conn); // maybe split schema creation into its own fn

    let client_id = insert_test_client(&conn);
    assert!(client_id > 0);
}

#[test]
fn test_get_client() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    init_schema(&conn); // maybe split schema creation into its own fn

    let client_id = insert_test_client(&conn);
    assert!(get_client_by_id(&conn, client_id).unwrap().id == client_id) // Assert we pull the right client
}

#[test]
fn test_store_session() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    init_schema(&conn); // maybe split schema creation into its own fn

    insert_test_client(&conn);
    let session_id = insert_test_session(&conn);

    assert!(session_id > 0);
}

#[test]
fn test_get_session() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    init_schema(&conn); // maybe split schema creation into its own fn

    let session_id = insert_test_session(&conn);
    assert!(get_client_by_id(&conn, session_id).unwrap().id == session_id) // Assert we pull the right client
}

#[test]
fn test_get_unfinished_session_empty_db() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    init_schema(&conn); // maybe split schema creation into its own fn

    get_active_session(&conn).unwrap();
}

#[test]
fn test_get_unfinished_session() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    init_schema(&conn); // maybe split schema creation into its own fn
    let session_id = insert_test_session(&conn);
    match get_active_session(&conn) {
        Ok(sesh) => assert!(sesh.unwrap().id == session_id),
        Err(_) => panic!("Unexpected!"),
    };
}

fn insert_test_client(conn: &Connection) -> i32 {
    init_schema(&conn); // maybe split schema creation into its own fn

    let client = Client {
        id: 0,
        name: "Alice".into(),
        note: Some("test client".into()),
    };

    store_client(&conn, &client).unwrap().expect("This is a test and should not fail")
}

fn insert_test_session(conn: &Connection) -> i32 {
    insert_test_client(&conn); // We need a client to insert a session
    let session = Session {
        client_id: 1,
        id: 0,
        start_timestamp: Utc::now().to_rfc3339(),
        end_timestamp: Option::None,
        note: Option::Some("testing".to_string()),
    };

    store_session(&conn, &session).unwrap()
}
