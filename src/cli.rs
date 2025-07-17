use clap::{value_parser, Parser, Subcommand};

use crate::models::Session;

#[derive(Parser)]
#[command(
    name = "timber",
    version = "1.0",
    author = "Mason",
    about = "A simple time tracker"
)]

pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
#[derive(Clone)]
pub enum UserInput {
    ByName(String),
    ById(i32)
}

fn parse_input(s: &str) -> Result<UserInput, String> {
    if let Ok(id) = s.parse::<i32>() {
        Ok(UserInput::ById(id))
    } else {
        Ok(UserInput::ByName(s.to_string()))
    }
}

#[derive(Subcommand)]
pub enum Commands {
    AddClient {
        name: String,
        #[arg(short, long)]
        note: Option<String>,
    },
    RemoveClient {
        #[arg(value_parser = parse_input)]
        input: UserInput,
    },
    ListClients,
    StartSession {
        #[arg(value_parser = parse_input)]
        input: UserInput,
        note: Option<String>,
    },
    EndSession,
    RemoveSession {
        id: i32,
    },
    ListSessions {
        #[arg(short, long)]
        #[arg(value_parser = parse_input)]
        client: Option<UserInput>,
    },
    ActiveSession,
}
