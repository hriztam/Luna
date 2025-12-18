//! Luna - Natural language system control for macOS
//!
//! A CLI tool that accepts natural language commands and executes
//! corresponding macOS system actions.

pub mod cli;
pub mod exec;
pub mod intent;
pub mod output;

use anyhow::Result;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::cli::Cli;
use crate::exec::{execute, get_command_string};
use crate::intent::parse_intent;
use crate::output::{
    print_dry_run_human, print_dry_run_json, print_error_human, print_error_json, print_human,
    print_json,
};

fn main() {
    // Initialize tracing (only shows errors by default)
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::ERROR)
        .with_writer(std::io::stderr)
        .without_time()
        .finish();

    if tracing::subscriber::set_global_default(subscriber).is_err() {
        // Tracing already set, continue anyway
    }

    if let Err(code) = run() {
        std::process::exit(code);
    }
}

fn run() -> Result<(), i32> {
    // Parse CLI arguments
    let cli = Cli::parse_args();
    let input = &cli.command;

    // Parse intent from natural language
    let action = match parse_intent(input) {
        Ok(action) => action,
        Err(e) => {
            if cli.json {
                print_error_json(input, &e.to_string());
            } else {
                print_error_human(input, &e.to_string());
            }
            return Err(1);
        }
    };

    // Handle dry-run mode
    if cli.dry_run {
        let command = get_command_string(&action);
        if cli.json {
            print_dry_run_json(input, &action, &command);
        } else {
            print_dry_run_human(input, &action, &command);
        }
        return Ok(());
    }

    // Execute the action
    match execute(&action) {
        Ok(result) => {
            if cli.json {
                print_json(input, &action, &result);
            } else {
                print_human(input, &action, &result);
            }

            if result.success {
                Ok(())
            } else {
                Err(1)
            }
        }
        Err(e) => {
            if cli.json {
                print_error_json(input, &e.to_string());
            } else {
                print_error_human(input, &e.to_string());
            }
            Err(1)
        }
    }
}
