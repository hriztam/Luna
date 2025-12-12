//! Intent module for Luna.
//!
//! Contains types and parsing logic for converting natural language
//! commands into structured actions.

pub mod parse;
pub mod types;

// Re-export commonly used items
pub use parse::{parse_intent, ParseError};
pub use types::Action;
