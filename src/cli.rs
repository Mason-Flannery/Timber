use std::path::PathBuf;
use clap::{Parser, Subcommand};

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
    ById(i32),
}
#[derive(Subcommand)]
pub enum ClientOptions {
    #[command(about = "Add a new client")]
    Add {
        name: String,
        #[arg(short, long)]
        note: Option<String>,
    },
    #[command(
        alias = "rm",
        about = "Remove the client with the provided id or name (alias: rm)"
    )]
    Remove {
        #[arg(value_parser = parse_input)]
        input: UserInput,
    },
    #[command(alias = "ls", about = "List all clients (alias: ls)")]
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
    #[command(
        alias = "new",
        about = "Start a new time-tracking session (alias: new)"
    )]
    Start {
        #[arg(value_parser = parse_input)]
        input: UserInput,
        note: Option<String>,
    },
    #[command(alias = "stop", about = "End the session tracking (alias: stop)")]
    End,
    #[command(
        alias = "rm",
        about = "Remove the session with the provided id (alias: rm)"
    )]
    Remove { id: i32 },
    #[command(
        alias = "ls",
        about = "List all sessions, optionally specify a specific client (alias: ls)"
    )]
    List {
        #[arg(value_parser = parse_input)]
        client: Option<UserInput>,
    },
    #[command(
        alias = "show",
        about = "Display the current working session (alias: show)"
    )]
    Current,
}

#[derive(clap::Subcommand, clap::ValueEnum, Clone, Debug)]
pub enum SummaryRange {
    /// Summary of hours tracked in the current day (UTC)
    Daily,
    /// Summary of hours tracked in the current pay week (Sat - Fri, UTC)
    Weekly,
    /// Summary of hours tracked in the current month (UTC)
    Monthly,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Set config values
    Set {
        #[arg(long)]
        //// Path to the database
        database_path: Option<PathBuf>,
        // Add other config fields here later
    },
    /// Show the current config
    Show,
    /// Reset the config to the default
    Reset,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(
        subcommand,
        alias = "project",
        about = "Manage clients (alias: project)"
    )]
    Client(ClientOptions),
    #[command(subcommand, about = "Manage sessions")]
    Session(SessionOptions),
    #[command(
        alias = "sum",
        about = "Display a formatted job summary of worked time"
    )]
    Summary {
        #[arg(value_enum, help = "Time range for summary (daily, weekly, monthly)")]
        range: SummaryRange,
    },
    #[command(about = "End current session and switch to a different client / project")]
    #[command(about = "End current session and switch to a different client / project")]
    Switch {
        #[arg(value_parser = parse_input)]
        input: UserInput,
        note: Option<String>,
    },
    #[command(
        alias = "fix",
        about = "Add or remove time from the current session",
        allow_hyphen_values = true
    )]
    Patch {
        #[arg(short, long, help = "Minutes to adjust (positive or negative)")]
        minutes: i32,
    },
    Config {
        #[command(subcommand, help = "View config options")]
        command: ConfigCommand,
    },
    #[command(about = "Display short status summary")]
    Status,
}
