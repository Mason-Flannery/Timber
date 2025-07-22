use std::{collections::HashMap};

use chrono::{Datelike, TimeZone, Utc};
use clap::Parser;
use cli::{Cli, Commands};

use crate::{cli::{ClientOptions, SessionOptions}, models::{Client, Session}};
mod cli;
mod db;
mod models;
mod utils;

fn main() {
    let conn = db::init_db(); // make sure the database exists

    let cli = Cli::parse();

    match cli.command {
        Commands::Client(client_cmd) => match client_cmd {
            ClientOptions::Add { name, note } => {
                match db::store_client(&conn, &Client { id: 0, name, note }) {
                    Ok(Some(id)) => println!("Client added with id {}", id),
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
                    Ok(_) => println!("Successfully removed client {}", client_id),
                    Err(_) => println!("Failed to remove client {}", client_id),
                };
            }
            ClientOptions::List => {
            let client_list =
                db::list_clients(&conn).expect("Error encountered getting client list");
            println!("Clients (Name, Id):");
            for client in client_list {
                println!("({}, {})", client.name, client.id);
                }
            }
        }
        Commands::Session(session_cmd) => match session_cmd {
            SessionOptions::Start { input, note } => {
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
                match db::get_active_session(&conn) {
                Ok(Some(session)) => {
                            println!(
                                "Cannot start a session because you are currently have a session for {}",
                                db::get_client_by_id(&conn, session.client_id)
                                    .expect("Error encountered getting client information")
                                    .name
                            )
                        }
                Ok(None) => {
                            match db::store_session(
                                &conn,
                                &Session {
                                    id: 0, // Will be assigned by sqlite instead
                                    client_id,
                                    start_timestamp: Utc::now().to_rfc3339(),
                                    end_timestamp: Option::None,
                                    note,
                                },
                            ) {
                                Ok(id) => println!("Started logging session {}", id),
                                Err(_) => println!("An error occured while trying to start a new session"),
                            }
                        }
                Err(e) => eprintln!("Failed to find current sessions: {}", e),
                }
            }
            SessionOptions::End => {
                match db::end_session(&conn) {
                    Ok(Some(delta)) => {
                        println!("Finished logging: {}hr {}m", delta.num_hours(), delta.num_minutes().wrapping_rem(60));
                    },
                    Ok(None) => println!("No active session was found to end!"),
                    Err(_) => println!("An error occurred trying to finish the session"),
                };
            }
            SessionOptions::Remove { id } => {
                let _ = db::remove_session(&conn, id);
            }
            SessionOptions::List {  client } => {
                // let client_id = match client {
                //     Some(UserInput::ById(id)) => id,
                //     Some(UserInput::ByName(name)) => match db::get_client_id_by_name(&conn, name),
                //     None => todo!(),
                // }
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
                println!("Sessions:");
                for session in session_list {
                    println!(
                        "{}:",
                        db::get_client_by_id(&conn, session.id)
                            .expect("The client must exist")
                            .name
                    );
                    match session.note {
                        Some(note) => println!("-- {}", note),
                        None => (),
                    }
                    println!();
                }
            }
            
            SessionOptions::Current => {
                match db::get_active_session(&conn) {
                    Ok(Some(session)) => {
                        let client = match db::get_client_by_id(&conn, session.id.clone()) {
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
        Commands::Summary { daily, weekly, monthly } => {
            if daily {
                let now = chrono::Local::now();
                let start = chrono::Local.with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0,0).unwrap(); // Get time at start of day
                let end = start + chrono::Duration::days(1); // Get time at end of day

                let results = db::get_sessions_within_range(&conn, &start.to_rfc3339(), &end.to_rfc3339()).expect("An error occurred while fetching the daily sessions");

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
                    println!("{}: {:?}", db::get_client_by_id(&conn, key).unwrap().name, utils::split_minutes(value as u32))
                });
            }
            if weekly {
                println!("Weekly summaries are not yet implemented!");
            }
            if monthly {
                println!("Monthly summaries are not yet implemented!")
            }
        },
    }
}
