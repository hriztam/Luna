//! Execution module for Luna.
//!
//! Platform-specific command execution logic.

pub mod macos;

// Re-export for convenience
pub use macos::{execute, get_command_string, ExecError, ExecResult};
