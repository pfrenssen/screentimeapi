use crate::db::AdjustmentQueryFilter;
use clap::{Parser, Subcommand};
use tabled::settings::Style;

mod db;
pub mod models;
pub mod schema;
mod web;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Todo: Return an exit code if the command failed.
    match &cli.command {
        Some(Commands::AdjustmentType { command }) => match command {
            Some(AdjustmentTypeCommands::List { limit }) => {
                list_adjustment_types(*limit);
            }
            Some(AdjustmentTypeCommands::Add {
                description,
                adjustment,
            }) => {
                db::add_adjustment_type(description.clone(), *adjustment);
            }
            Some(AdjustmentTypeCommands::Delete { id }) => {
                let result = db::delete_adjustment_type(*id);
                match result {
                    Ok(rows_deleted) => println!("Deleted {rows_deleted} adjustment type(s)"),
                    Err(e) => println!("Error: {e}"),
                }
            }
            None => {}
        },
        Some(Commands::Adjustment { command }) => match command {
            Some(AdjustmentCommands::List {
                limit,
                adjustment_type_id,
                since,
            }) => {
                list_adjustments(&AdjustmentQueryFilter {
                    limit: *limit,
                    atid: *adjustment_type_id,
                    since: since.map(|d| d.and_hms_opt(0, 0, 0).unwrap()),
                });
            }
            Some(AdjustmentCommands::Add {
                adjustment_type_id,
                comment,
            }) => {
                add_adjustment(*adjustment_type_id, comment);
            }
            None => {}
        },
        Some(Commands::Serve) => web::serve().await,
        Some(Commands::TimeEntry { command }) => match command {
            Some(TimeEntryCommands::Current) => {
                print_current_time();
            }
            Some(TimeEntryCommands::List { limit }) => {
                list_time_entries(*limit);
            }
            Some(TimeEntryCommands::Add { time }) => {
                db::add_time_entry(*time);
            }
            None => {}
        },
        None => {}
    }
}

/// Lists the available adjustments.
fn list_adjustments(filter: &AdjustmentQueryFilter) {
    let results = db::get_adjustments(filter);

    // Output results as a table.
    let mut table = tabled::Table::new(results);
    table.with(Style::sharp());
    println!("{table}");
}

/// Adds an adjustment.
fn add_adjustment(adjustment_type_id: u64, comment: &Option<String>) {
    let adjustment_type = db::get_adjustment_types(None)
        .into_iter()
        .find(|at| at.id == adjustment_type_id)
        .expect("Adjustment type not found");

    db::add_adjustment(&adjustment_type, comment);
}

/// Lists the available adjustment types.
fn list_adjustment_types(limit: Option<u8>) {
    let results = db::get_adjustment_types(limit);

    // Output results as a table.
    let mut table = tabled::Table::new(results);
    table.with(Style::sharp());
    println!("{table}");
}

/// Prints the current time.
fn print_current_time() {
    let time_entry = db::get_current_time_entry();
    if let Some(time_entry) = time_entry {
        println!("{time_entry}");
    }
}

/// Lists the available time entries.
fn list_time_entries(limit: Option<u8>) {
    let results = db::get_time_entries(limit);

    // Output results as a table.
    let mut table = tabled::Table::new(results);
    table.with(Style::sharp());
    println!("{table}");
}

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Commands related to adjustments.
    Adjustment {
        #[command(subcommand)]
        command: Option<AdjustmentCommands>,
    },
    /// Commands related to adjustment types.
    AdjustmentType {
        #[command(subcommand)]
        command: Option<AdjustmentTypeCommands>,
    },
    /// Starts the web server.
    Serve,
    /// Commands related to time entries.
    TimeEntry {
        #[command(subcommand)]
        command: Option<TimeEntryCommands>,
    },
}

#[derive(Subcommand)]
#[command(arg_required_else_help = true)]
enum AdjustmentCommands {
    /// Lists the available adjustments.
    List {
        /// The maximum number of adjustments to return.
        #[arg(short, long)]
        limit: Option<u8>,
        /// Filters the adjustments by the given adjustment type ID.
        #[arg(short, long)]
        adjustment_type_id: Option<u64>,
        /// Return only adjustments created after the given date.
        #[arg(short, long)]
        since: Option<chrono::NaiveDate>,
    },
    /// Adds a new adjustment.
    Add {
        /// The adjustment type ID of the adjustment.
        #[arg(short, long)]
        adjustment_type_id: u64,

        /// The comment of the adjustment.
        #[arg(short, long)]
        comment: Option<String>,
    },
}

#[derive(Subcommand)]
#[command(arg_required_else_help = true)]
enum AdjustmentTypeCommands {
    /// Lists the available adjustment types.
    List {
        /// The maximum number of adjustment types to return.
        #[arg(short, long)]
        limit: Option<u8>,
    },
    /// Adds a new adjustment type.
    Add {
        /// The description of the adjustment type.
        #[arg(short, long)]
        description: String,

        /// The adjustment value of the adjustment type.
        #[arg(short, long)]
        adjustment: i8,
    },
    /// Deletes the adjustment type with the given ID.
    Delete {
        /// The ID of the adjustment type to delete.
        #[arg(short, long)]
        id: u64,
    },
}

#[derive(Subcommand)]
#[command(arg_required_else_help = true)]
enum TimeEntryCommands {
    /// Returns the current time entry.
    Current,
    /// Lists the available time entries.
    List {
        /// The maximum number of time entries to return.
        #[arg(short, long)]
        limit: Option<u8>,
    },
    /// Adds a new time entry.
    Add {
        /// The time of the time entry.
        #[arg(short, long)]
        time: u16,
    },
}
