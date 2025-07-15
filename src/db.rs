use std::time::Instant;

use rusqlite::{Connection, Result};

use crate::models::Client;

pub fn init_db() -> Connection {
    let conn = Connection::open("timber.db").expect("No issues");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS clients (
            id    INTEGER PRIMARY KEY AUTOINCREMENT,
            name  TEXT NOT NULL,
            note  TEXT
        );
        
        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            client_id INTEGER NOT NULL,
            start_timestamp TEXT NOT NULL,
            elapsed_seconds INTEGER NOT NULL, 
            note TEXT,
            FOREIGN KEY (client_id) REFERENCES clients(id)
        );",
    )
    .expect("no issues");
    // conn.execute(
    //     "INSERT INTO client (name, note) VALUES (?1, ?2)",
    //     (&me.name, &me.note),
    // )
    // .expect("no issues");
    // conn.is_autocommit();
    conn // we assume conn is always valid here
}
