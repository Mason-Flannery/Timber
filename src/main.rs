use std::{collections::HashMap};

use chrono::{Datelike, TimeZone, Utc};
use clap::Parser;
use cli::{Cli, Commands};
use rusqlite::Connection;

use crate::{cli::{ClientOptions, SessionOptions, UserInput}, models::{Client, Session}};
mod cli;
mod db;
mod models;
mod utils;
mod commands;
fn main() {
    let conn = db::init_db(); // make sure the database exists

    let cli = Cli::parse();

    match cli.command {
        Commands::Client(client_cmd) => match client_cmd {
                                ClientOptions::Add { name, note } => {
                                    match db::store_client(&conn, &Client { id: 0, name, note }) {
                                        Ok(Some(id)) => println!("Client added with id {id}"),
                                        Ok(None) => println!("The client already exists!"),
                                        Err(_) => println!("Failed to add new client"),
                                    }
                                }
                                ClientOptions::Remove { input } => {
                                    let client_id: i32 = match input {
                                        cli::UserInput::ByName(name) => {
                                            match db::get_client_id_by_name(&conn, name) {
                                                Ok(Some(id)) => id,
                                                Ok(None) => {
                                                    println!("Provided client could not be found!");
                                                    return;
                                                }
                                                Err(_) => {
                                                    println!("Error: Unable to start session");
                                                    return
                                                },
                                            }
                                        }
                                        cli::UserInput::ById(id) => id,
                                    };
        
                                    match db::remove_client(&conn, client_id) {
                                        Ok(_) => println!("Successfully removed client {client_id}"),
                                        Err(rusqlite::Error::SqliteFailure(_err, _)) => {
                                            println!("This client is referenced in some of your sessions! Removal is not yet supported.");
                                        },
                                        Err(e) => println!("Failed to remove client {client_id}, {e}"),
                                    }
                                }
                                ClientOptions::List => {
                                let client_list =
                                    db::list_clients(&conn).expect("Error encountered getting client list");
                                if client_list.is_empty() {
                                    println!("No clients to display!");
                                    return;
                                }
                                println!("Clients (Name, Id):");
                                for client in client_list {
                                    println!("({}, {})", client.name, client.id);
                                    }
                                }
                    }
        Commands::Session(session_cmd) => match session_cmd {
                        SessionOptions::Start { input, note } => {
                            start_session(&conn, input, note)
                        }
                        SessionOptions::End => {
                            end_session(&conn);
                        }
                        SessionOptions::Remove { id } => {
                            let _ = db::remove_session(&conn, id);
                        }
                        SessionOptions::List {  client } => {
                            let client_id: Option<i32> = match client {
                                Some(cli::UserInput::ByName(name)) => {
                                    match db::get_client_id_by_name(&conn, name) {
                                        Ok(Some(id)) => Some(id),
                                        Ok(None) => {
                                            println!("Provided client could not be found!");
                                            return;
                                        }
                                        Err(_) => {
                                            println!("Error: Unable to start session");
                                            return
                                        },
                                    }
                                }
                                Some(cli::UserInput::ById(id)) => Some(id),
                                None => None
                            };
                            let session_list = match client_id {
                                Some(client_id) => db::list_sessions(&conn, Some(client_id))
                                    .expect("Error encountered getting session list"),
                                None => db::list_sessions(&conn, None).expect("Error encountered getting session list"), // Swap to other func
                            };
                            if session_list.is_empty() {
                                println!("No sessions to display!");
                                return;
                            }
                            for session in session_list {
                                let client = db::get_client_by_id(&conn, session.client_id)
                                        .expect("The client must exist");
                                match session.get_timedelta() {
                                    Some(delta) => {
                                        let (hours, minutes) = utils::split_minutes(delta.num_minutes() as u32);
                                        println!(
                                            "{}:\n{}h {}m ({})", // ! Why in the world is this broken???
                                            client.name,
                                            hours,
                                            minutes,
                                            session.start_timestamp
                                        );
                                    },
                                    None => {
                                        println!("{}:\n {}", client.name, session.start_timestamp)
                                    },
                                }
                        
                                if let Some(note) = session.note { println!("-- {note}") }
                                println!();
                            }
                        }
            
                        SessionOptions::Current => {
                            match db::get_active_session(&conn) {
                                Ok(Some(session)) => {
                                    let client = match db::get_client_by_id(&conn, session.client_id) {
                                        Ok(client) => client.name,
                                        Err(_) => "Unknown".to_string(),
                                    };

                                    println!("{}: {:?}\n Started at {} ", client, session.note, session.start_timestamp, )
                                },
                                Ok(None) => {
                                    println!("No active session found!")
                                }
                                Err(_) => println!("An error occurred"),
                            }
                        }
                    }
        Commands::Summary { range } => {
                    match range {
                        cli::SummaryRange::Daily => {
                            let now = chrono::Utc::now();
                            let start = chrono::Utc.with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0,0).unwrap(); // Get time at start of day
                            let end = start + chrono::Duration::days(1); // Get time at end of day

                            let results = db::get_sessions_within_range(&conn, &start, &end).expect("An error occurred while fetching the daily sessions");
                            let mut client_totals: HashMap<i32, i64> = HashMap::new();
                            for result in results {
                                // println!("{}, {}, {}", result.clien)
                        
                                if client_totals.contains_key(&result.client_id) {
                                    match result.get_timedelta() {
                                        Some(delta) => client_totals.insert(result.client_id, client_totals.get(&result.client_id).unwrap() + delta.num_minutes()),
                                        None => continue,
                                    };
                                }
                                else {
                                    match result.get_timedelta() {
                                        Some(delta) => client_totals.insert(result.client_id, delta.num_minutes()),
                                        None => continue,
                                    };
                                }
                            }
                            client_totals.iter().for_each(|(&key, &value)| {
                                let (hours, minutes) = utils::split_minutes(value as u32);
                                println!("{}:\n{}h {}m\n", db::get_client_by_id(&conn, key).unwrap().name, hours, minutes)
                            });
                        }
                        cli::SummaryRange::Weekly => println!("Weekly summaries are not yet implemented!"),
                        cli::SummaryRange::Monthly => println!("Monthly summaries are not yet implemented!"),
                    }
                },
        Commands::Switch { input, note } =>{
        end_session(&conn);
        start_session(&conn, input, note);
        },
    Commands::Patch { minutes } => {
        match commands::session::patch_session(&conn, minutes) {
            Ok(Some(_)) => {
                println!("Successfully patched active session with {minutes} minutes!")
            },
            Ok(None) => {
                println!("No active session was found to patch!")
            },
            Err(e) => eprintln!("Failed to patch active session: {e}"),
        }
    },
    }
}

fn end_session(conn: &Connection) {
    match commands::session::end_session(conn) {
        Ok(Some(delta)) => {
            println!("Finished logging: {}hr {}m", delta.num_hours(), delta.num_minutes().wrapping_rem(60));
        },
        Ok(None) => println!("No active session was found to end!"),
        Err(_) => println!("An error occurred trying to finish the session"),
    };
}

fn start_session(conn: &Connection, input: UserInput, note:Option<String>) {
    let client_id: i32 = match input {
        cli::UserInput::ByName(name) => {
            match db::get_client_id_by_name(conn, name) {
                Ok(Some(id)) => id,
                Ok(None) => {
                    println!("Provided client could not be found!");
                    return;
                }
                Err(_) => {
                    println!("Error: Unable to start session");
                    return
                },
            }
        }
        cli::UserInput::ById(id) => id,
    };
    match db::get_active_session(conn) {
    Ok(Some(session)) => {
                println!(
                    "Cannot start a session because you are currently have a session for {}",
                    db::get_client_by_id(conn, session.client_id)
                        .expect("Error encountered getting client information")
                        .name
                )
            }
    Ok(None) => {
                match db::store_session(
                    conn,
                    &Session {
                        id: 0, // Will be assigned by sqlite instead
                        client_id,
                        start_timestamp: Utc::now().to_rfc3339(),
                        end_timestamp: Option::None,
                        note,
                        offset_minutes: 0,
                    },
                ) {
                    Ok(id) => println!("Started logging session {} for {}", id, db::get_client_by_id(conn, client_id).unwrap().name),
                    Err(_) => println!("An error occured while trying to start a new session"),
                }
            }
    Err(e) => eprintln!("Failed to find current sessions: {e}"),
    }
}
