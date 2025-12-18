//! CLI argument parsing for Luna.
//!
//! Handles command-line interface using clap derive macros.

use clap::Parser;

/// Luna - Natural language system control for macOS
#[derive(Parser, Debug)]
#[command(name = "luna")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Natural language command to execute
    #[arg(required = true)]
    pub command: String,

    /// Parse and print intent without executing
    #[arg(long)]
    pub dry_run: bool,

    /// Output result in JSON format
    #[arg(long)]
    pub json: bool,
}

impl Cli {
    /// Parse CLI arguments from environment
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::try_parse_from(["luna", "open safari"]).unwrap();
        assert_eq!(cli.command, "open safari");
        assert!(!cli.dry_run);
        assert!(!cli.json);
    }

    #[test]
    fn test_cli_dry_run() {
        let cli = Cli::try_parse_from(["luna", "--dry-run", "open safari"]).unwrap();
        assert!(cli.dry_run);
    }

    #[test]
    fn test_cli_json() {
        let cli = Cli::try_parse_from(["luna", "--json", "set volume to 40"]).unwrap();
        assert!(cli.json);
    }
}
