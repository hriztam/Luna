//! macOS execution layer for Luna.
//!
//! Executes actions using shell commands via std::process::Command.

use crate::intent::Action;
use std::process::Command;
use thiserror::Error;

/// Errors that can occur during command execution.
#[derive(Error, Debug)]
pub enum ExecError {
    #[error("Failed to execute command: {0}")]
    CommandFailed(String),

    #[error("Command returned non-zero exit code: {0}")]
    NonZeroExit(i32),

    #[error("Failed to spawn process: {0}")]
    SpawnFailed(#[from] std::io::Error),
}

/// Result of executing an action.
#[derive(Debug, Clone)]
pub struct ExecResult {
    /// The command that was executed
    pub command: String,
    /// Whether execution was successful
    pub success: bool,
    /// Optional output from the command
    pub output: Option<String>,
}

/// Execute an action on macOS.
///
/// # Arguments
/// * `action` - The parsed action to execute
///
/// # Returns
/// * `Ok(ExecResult)` - Execution completed (check success field)
/// * `Err(ExecError)` - Failed to execute command
pub fn execute(action: &Action) -> Result<ExecResult, ExecError> {
    match action {
        Action::OpenApp { name } => execute_open_app(name),
        Action::OpenUrl { url } => execute_open_url(url),
        Action::SetVolume { level } => execute_set_volume(*level),
        Action::Mute => execute_mute(),
        Action::Unmute => execute_unmute(),
    }
}

/// Get the command string that would be executed for an action (for dry-run).
pub fn get_command_string(action: &Action) -> String {
    match action {
        Action::OpenApp { name } => format!("open -a \"{}\"", name),
        Action::OpenUrl { url } => format!("open \"{}\"", url),
        Action::SetVolume { level } => {
            format!("osascript -e 'set volume output volume {}'", level)
        }
        Action::Mute => "osascript -e 'set volume with output muted'".to_string(),
        Action::Unmute => "osascript -e 'set volume without output muted'".to_string(),
    }
}

/// Execute: open -a "AppName"
fn execute_open_app(name: &str) -> Result<ExecResult, ExecError> {
    let command_str = format!("open -a \"{}\"", name);

    let output = Command::new("open")
        .arg("-a")
        .arg(name)
        .output()?;

    Ok(ExecResult {
        command: command_str,
        success: output.status.success(),
        output: if output.status.success() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        },
    })
}

/// Execute: open "URL"
fn execute_open_url(url: &str) -> Result<ExecResult, ExecError> {
    let command_str = format!("open \"{}\"", url);

    let output = Command::new("open")
        .arg(url)
        .output()?;

    Ok(ExecResult {
        command: command_str,
        success: output.status.success(),
        output: if output.status.success() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        },
    })
}

/// Execute: osascript -e 'set volume output volume <level>'
fn execute_set_volume(level: u8) -> Result<ExecResult, ExecError> {
    let script = format!("set volume output volume {}", level);
    let command_str = format!("osascript -e '{}'", script);

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()?;

    Ok(ExecResult {
        command: command_str,
        success: output.status.success(),
        output: if output.status.success() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        },
    })
}

/// Execute: osascript -e 'set volume with output muted'
fn execute_mute() -> Result<ExecResult, ExecError> {
    let script = "set volume with output muted";
    let command_str = format!("osascript -e '{}'", script);

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()?;

    Ok(ExecResult {
        command: command_str,
        success: output.status.success(),
        output: if output.status.success() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        },
    })
}

/// Execute: osascript -e 'set volume without output muted'
fn execute_unmute() -> Result<ExecResult, ExecError> {
    let script = "set volume without output muted";
    let command_str = format!("osascript -e '{}'", script);

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()?;

    Ok(ExecResult {
        command: command_str,
        success: output.status.success(),
        output: if output.status.success() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_command_string_open_app() {
        let action = Action::OpenApp { name: "Safari".to_string() };
        assert_eq!(get_command_string(&action), "open -a \"Safari\"");
    }

    #[test]
    fn test_get_command_string_open_url() {
        let action = Action::OpenUrl { url: "https://google.com".to_string() };
        assert_eq!(get_command_string(&action), "open \"https://google.com\"");
    }

    #[test]
    fn test_get_command_string_set_volume() {
        let action = Action::SetVolume { level: 40 };
        assert_eq!(
            get_command_string(&action),
            "osascript -e 'set volume output volume 40'"
        );
    }

    #[test]
    fn test_get_command_string_mute() {
        assert_eq!(
            get_command_string(&Action::Mute),
            "osascript -e 'set volume with output muted'"
        );
    }

    #[test]
    fn test_get_command_string_unmute() {
        assert_eq!(
            get_command_string(&Action::Unmute),
            "osascript -e 'set volume without output muted'"
        );
    }
}
