use clap::{value_parser, Args, Parser, Subcommand};

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
#[derive(Subcommand)]
pub enum ClientOptions {
    Add {
        name: String,
        #[arg(short, long)]
        note: Option<String>,
    },
    Remove {
        #[arg(value_parser = parse_input)]
        input: UserInput,
    },
    List,
}

fn parse_input(s: &str) -> Result<UserInput, String> {
    if let Ok(id) = s.parse::<i32>() {
        Ok(UserInput::ById(id))
    } else {
        Ok(UserInput::ByName(s.to_string()))
    }
}

#[derive(Subcommand)]
pub enum SessionOptions {
    Start {
        #[arg(value_parser = parse_input)]
        input: UserInput,
        note: Option<String>,
    },
    End,
    Remove {
        id: i32,
    },
    List {
        #[arg(short, long)]
        #[arg(value_parser = parse_input)]
        client: Option<UserInput>,
    },
    Current,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(subcommand, alias = "project", about = "Manage clients (alias: project)")]
    Client(ClientOptions),
    #[command(subcommand, about = "Manage sessions")]
    Session(SessionOptions)
}
