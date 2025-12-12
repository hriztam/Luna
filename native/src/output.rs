//! Output formatting for Luna.
//!
//! Handles both human-readable and JSON output formats.

use crate::exec::ExecResult;
use crate::intent::Action;
use serde::Serialize;

/// JSON output structure for machine-readable output.
#[derive(Debug, Serialize)]
pub struct JsonOutput {
    pub input: String,
    pub intent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executed: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Print the result of executing an action in human-readable format.
pub fn print_human(input: &str, action: &Action, result: &ExecResult) {
    println!("Input: \"{}\"", input);
    println!("Intent: {}", action);
    println!("Executed: {}", result.command);
    if result.success {
        println!("Result: success");
    } else {
        println!("Result: failed");
        if let Some(ref output) = result.output {
            println!("Error: {}", output.trim());
        }
    }
}

/// Print the result of executing an action in JSON format.
pub fn print_json(input: &str, action: &Action, result: &ExecResult) {
    let output = JsonOutput {
        input: input.to_string(),
        intent: action.to_string(),
        executed: Some(result.command.clone()),
        status: if result.success { "success" } else { "failed" }.to_string(),
        error: if result.success {
            None
        } else {
            result.output.clone()
        },
    };

    // Pretty print for readability
    if let Ok(json) = serde_json::to_string_pretty(&output) {
        println!("{}", json);
    }
}

/// Print dry-run output in human-readable format.
pub fn print_dry_run_human(input: &str, action: &Action, command: &str) {
    println!("Input: \"{}\"", input);
    println!("Intent: {}", action);
    println!("Would execute: {}", command);
    println!("(dry-run mode - no action taken)");
}

/// Print dry-run output in JSON format.
pub fn print_dry_run_json(input: &str, action: &Action, command: &str) {
    let output = JsonOutput {
        input: input.to_string(),
        intent: action.to_string(),
        executed: Some(command.to_string()),
        status: "dry-run".to_string(),
        error: None,
    };

    if let Ok(json) = serde_json::to_string_pretty(&output) {
        println!("{}", json);
    }
}

/// Print an error in human-readable format.
pub fn print_error_human(input: &str, error: &str) {
    eprintln!("Input: \"{}\"", input);
    eprintln!("Error: {}", error);
    eprintln!();
    eprintln!("Examples of valid commands:");
    eprintln!("  luna \"open safari\"");
    eprintln!("  luna \"open https://google.com\"");
    eprintln!("  luna \"set volume to 50\"");
    eprintln!("  luna \"mute\"");
    eprintln!("  luna \"unmute\"");
}

/// Print an error in JSON format.
pub fn print_error_json(input: &str, error: &str) {
    let output = JsonOutput {
        input: input.to_string(),
        intent: "unknown".to_string(),
        executed: None,
        status: "error".to_string(),
        error: Some(error.to_string()),
    };

    if let Ok(json) = serde_json::to_string_pretty(&output) {
        println!("{}", json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_output_serialization() {
        let output = JsonOutput {
            input: "open safari".to_string(),
            intent: "OpenApp(name=\"Safari\")".to_string(),
            executed: Some("open -a \"Safari\"".to_string()),
            status: "success".to_string(),
            error: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("open safari"));
        assert!(json.contains("success"));
        assert!(!json.contains("error")); // Should be skipped when None
    }

    #[test]
    fn test_json_output_with_error() {
        let output = JsonOutput {
            input: "invalid".to_string(),
            intent: "unknown".to_string(),
            executed: None,
            status: "error".to_string(),
            error: Some("Could not parse".to_string()),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("Could not parse"));
    }
}
