use crate::db::AdjustmentQueryFilter;
use clap::{Parser, Subcommand};
use diesel::MysqlConnection;
use tabled::settings::Style;

mod db;
pub mod models;
pub mod schema;
mod web;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let pool = db::get_connection_pool();
    let connection = &mut pool.get().unwrap();

    // Todo: Return an exit code if the command failed.
    match &cli.command {
        None => {}
        Some(Commands::AdjustmentType { command }) => match command {
            Some(AdjustmentTypeCommands::List { limit }) => {
                list_adjustment_types(connection, *limit);
            }
            Some(AdjustmentTypeCommands::Add {
                description,
                adjustment,
            }) => {
                db::add_adjustment_type(connection, description.clone(), *adjustment);
            }
            Some(AdjustmentTypeCommands::Delete { id }) => {
                let result = db::delete_adjustment_type(connection, *id);
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
                list_adjustments(
                    connection,
                    &AdjustmentQueryFilter {
                        limit: *limit,
                        atid: *adjustment_type_id,
                        since: since.map(|d| d.and_hms_opt(0, 0, 0).unwrap()),
                    },
                );
            }
            Some(AdjustmentCommands::Add {
                adjustment_type_id,
                comment,
            }) => {
                add_adjustment(connection, *adjustment_type_id, comment);
            }
            None => {}
        },
        Some(Commands::Serve) => web::serve().await,
        Some(Commands::Time) => {
            print_adjusted_time(connection);
        }
        Some(Commands::TimeEntry { command }) => match command {
            None => {}
            Some(TimeEntryCommands::Current) => {
                print_current_time_entry(connection);
            }
            Some(TimeEntryCommands::List { limit }) => {
                list_time_entries(connection, *limit);
            }
            Some(TimeEntryCommands::Add { time }) => {
                db::add_time_entry(connection, *time, None);
            }
            Some(TimeEntryCommands::Delete { id }) => {
                db::delete_time_entry(connection, *id);
            }
        },
    }
}

/// Lists the available adjustments.
fn list_adjustments(connection: &mut MysqlConnection, filter: &AdjustmentQueryFilter) {
    let results = db::get_adjustments(connection, filter);

    // Output results as a table.
    let mut table = tabled::Table::new(results);
    table.with(Style::sharp());
    println!("{table}");
}

/// Adds an adjustment.
fn add_adjustment(
    connection: &mut MysqlConnection,
    adjustment_type_id: u64,
    comment: &Option<String>,
) {
    let adjustment_type = db::get_adjustment_types(connection, None)
        .into_iter()
        .find(|at| at.id == adjustment_type_id)
        .expect("Adjustment type not found");

    db::add_adjustment(connection, &adjustment_type, comment, &None);
}

/// Lists the available adjustment types.
fn list_adjustment_types(connection: &mut MysqlConnection, limit: Option<u8>) {
    let results = db::get_adjustment_types(connection, limit);

    // Output results as a table.
    let mut table = tabled::Table::new(results);
    table.with(Style::sharp());
    println!("{table}");
}

/// Prints the current, adjusted time.
///
/// This calculates the current time by taking the most recent time entry and adding all adjustments
/// to it.
fn print_adjusted_time(connection: &mut MysqlConnection) {
    let adjusted_time = db::get_adjusted_time(connection);
    println!("{:01}:{:02}", adjusted_time / 60, adjusted_time % 60);
}

/// Prints the current time.
fn print_current_time_entry(connection: &mut MysqlConnection) {
    let time_entry = db::get_current_time_entry(connection);
    if let Some(time_entry) = time_entry {
        println!("{time_entry}");
    }
}

/// Lists the available time entries.
fn list_time_entries(connection: &mut MysqlConnection, limit: Option<u8>) {
    let results = db::get_time_entries(connection, limit);

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
    /// Returns the current screen time.
    Time,
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
    /// Deletes the time entry with the given ID.
    Delete {
        /// The ID of the time entry to delete.
        id: u64,
    },
}
