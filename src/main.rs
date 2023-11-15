use clap::{Parser, Subcommand};
use tabled::settings::Style;

pub mod models;
pub mod schema;
mod db;

fn main() {
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
        None => {}
    }
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
    /// Commands related to adjustment types.
    AdjustmentType {
        #[command(subcommand)]
        command: Option<AdjustmentTypeCommands>,
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