use clap::{Parser, Subcommand};
use tabled::settings::Style;

pub mod models;
pub mod schema;
mod db;
mod web;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::AdjustmentType { command }) => {
            match command {
                Some(AdjustmentTypeCommands::List { limit }) => {
                    list_adjustment_types(*limit);
                }
                Some(AdjustmentTypeCommands::Add { description, adjustment }) => {
                    db::add_adjustment_type(description, *adjustment);
                }
                None => {}
            }
        }
        Some(Commands::Adjustment { command }) => {
            match command {
                Some(AdjustmentCommands::List { limit }) => {
                    list_adjustments(*limit);
                }
                Some(AdjustmentCommands::Add { adjustment_type_id, comment }) => {
                    add_adjustment(*adjustment_type_id, comment.as_deref());
                }
                None => {}
            }
        }
        Some(Commands::Serve) => web::serve().await,
        None => {}
    }
}

/// Lists the available adjustments.
fn list_adjustments(limit: Option<u8>) {
    let results = db::get_adjustments(limit);

    // Output results as a table.
    let mut table = tabled::Table::new(results);
    table.with(Style::sharp());
    println!("{}", table);
}

/// Adds an adjustment.
fn add_adjustment(adjustment_type_id: u64, comment: Option<&str>) {
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
    println!("{}", table);
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
    Serve
}

#[derive(Subcommand)]
#[command(arg_required_else_help = true)]
enum AdjustmentCommands {
    /// Lists the available adjustments.
    List {
        /// The maximum number of adjustments to return.
        #[arg(short, long)]
        limit: Option<u8>,
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
}
