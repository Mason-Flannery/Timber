use std::collections::HashMap;

use chrono::{Datelike, TimeZone, Utc};
use clap::Parser;
use cli::{Cli, Commands};
use rusqlite::Connection;

use crate::{
    cli::{ClientOptions, SessionOptions, UserInput},
    models::{Client, Session},
};
mod cli;
mod commands;
mod db;
mod models;
mod utils;
mod views;
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
                    cli::UserInput::ByName(name) => match db::get_client_id_by_name(&conn, name) {
                        Ok(Some(id)) => id,
                        Ok(None) => {
                            println!("Provided client could not be found!");
                            return;
                        }
                        Err(_) => {
                            println!("Error: Unable to start session");
                            return;
                        }
                    },
                    cli::UserInput::ById(id) => id,
                };

                match db::remove_client(&conn, client_id) {
                    Ok(_) => println!("Successfully removed client {client_id}"),
                    Err(rusqlite::Error::SqliteFailure(_err, _)) => {
                        println!(
                            "This client is referenced in some of your sessions! Removal is not yet supported."
                        );
                    }
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
        },
        Commands::Session(session_cmd) => match session_cmd {
            SessionOptions::Start { input, note } => start_session(&conn, input, note),
            SessionOptions::End => {
                end_session(&conn);
            }
            SessionOptions::Remove { id } => {
                let _ = db::remove_session(&conn, id);
            }
            SessionOptions::List { client } => {
                let client_id = Option::None;
                if client.is_some() {
                    let client_id: Option<i32> = utils::handle_user_client_input(&conn, client);
                    if client_id.is_none() {
                        println!("Provided client could not be found!");
                        return;
                    }
                }
                let sessions =
                    db::list_sessions(&conn, client_id).expect("Failed to list sessions");

                if sessions.is_empty() {
                    println!("No sessions to display!");
                    return;
                }
                views::display_sessions(&conn, sessions);
            }

            SessionOptions::Current => {
                views::display_active_session(&conn);
            }
        },
        Commands::Summary { range } => match range {
            cli::SummaryRange::Daily => {
                let (start, end) = utils::current_day_range();
                views::display_client_time_summaries(&conn, &start, &end);
            }
            cli::SummaryRange::Weekly => {
                let (start, end) = utils::current_week_range();
                views::display_client_time_summaries(&conn, &start, &end);
            }
            cli::SummaryRange::Monthly => {
                let (start, end) = utils::current_month_range();
                views::display_client_time_summaries(&conn, &start, &end);
            }
        },
        Commands::Switch { input, note } => {
            end_session(&conn);
            start_session(&conn, input, note);
        }
        Commands::Patch { minutes } => match commands::session::patch_session(&conn, minutes) {
            Ok(Some(_)) => {
                println!("Successfully patched active session with {minutes} minutes!")
            }
            Ok(None) => {
                println!("Error: No active session was found to patch!")
            }
            Err(e) => eprintln!("Error: Failed to patch active session: {e}"),
        },
    }
}

fn end_session(conn: &Connection) {
    match commands::session::end_session(conn) {
        Ok(Some(delta)) => {
            println!(
                "Finished logging: {}hr {}m",
                delta.num_hours(),
                delta.num_minutes().wrapping_rem(60)
            );
        }
        Ok(None) => println!("Error: No active session was found to end!"),
        Err(_) => println!("Error: Unable to to finish the session"),
    };
}

fn start_session(conn: &Connection, input: UserInput, note: Option<String>) {
    let client_id = match utils::handle_user_client_input(conn, Some(input)) {
        Some(id) => id,
        None => {
            println!("Error: No client with that name found. Do they exist?");
            return;
        }
    };
    match db::get_active_session(conn) {
        // ! Wrote func in view? ?
        Ok(Some(session)) => {
            println!(
                "Error: Cannot start a session because you are currently have a session for {}",
                db::get_client_by_id(conn, session.client_id)
                    .expect("Error: Unable to get client information")
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
                Ok(id) => println!(
                    "Started logging session {} for {}",
                    id,
                    db::get_client_by_id(conn, client_id).unwrap().name
                ),
                Err(_) => println!("Error: Unable to start a new session"),
            }
        }
        Err(e) => eprintln!("Error: Unable to get the current session: {e}"),
    }
}
