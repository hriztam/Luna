//! Intent types for Luna.
//!
//! Defines the core Action enum representing all supported commands.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a parsed user action/intent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum Action {
    /// Open an application by name
    OpenApp { name: String },

    /// Open a URL in the default browser
    OpenUrl { url: String },

    /// Set the system volume to a specific level (0-100)
    SetVolume { level: u8 },

    /// Mute system audio
    Mute,

    /// Unmute system audio
    Unmute,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::OpenApp { name } => write!(f, "OpenApp(name=\"{}\")", name),
            Action::OpenUrl { url } => write!(f, "OpenUrl(url=\"{}\")", url),
            Action::SetVolume { level } => write!(f, "SetVolume(level={})", level),
            Action::Mute => write!(f, "Mute"),
            Action::Unmute => write!(f, "Unmute"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_display() {
        assert_eq!(
            Action::OpenApp { name: "Safari".to_string() }.to_string(),
            "OpenApp(name=\"Safari\")"
        );
        assert_eq!(
            Action::OpenUrl { url: "https://google.com".to_string() }.to_string(),
            "OpenUrl(url=\"https://google.com\")"
        );
        assert_eq!(
            Action::SetVolume { level: 40 }.to_string(),
            "SetVolume(level=40)"
        );
        assert_eq!(Action::Mute.to_string(), "Mute");
        assert_eq!(Action::Unmute.to_string(), "Unmute");
    }

    #[test]
    fn test_action_serialization() {
        let action = Action::SetVolume { level: 50 };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("SetVolume"));
        assert!(json.contains("50"));
    }
}
