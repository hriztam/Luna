//! Intent parser for Luna.
//!
//! Rule-based natural language parsing into Action types.

use super::types::Action;
use thiserror::Error;

/// Errors that can occur during intent parsing.
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Could not understand command: \"{0}\"")]
    UnrecognizedCommand(String),

    #[error("Invalid volume level: {0}. Must be 0-100.")]
    InvalidVolume(String),

    #[error("Missing target for 'open' command. Try: 'open safari' or 'open https://example.com'")]
    MissingOpenTarget,

    #[error("Missing volume level. Try: 'set volume to 50'")]
    MissingVolumeLevel,
}

/// Parse a natural language command into an Action.
///
/// # Arguments
/// * `input` - Raw natural language input from the user
///
/// # Returns
/// * `Ok(Action)` - Successfully parsed action
/// * `Err(ParseError)` - Parsing failed with helpful error message
///
/// # Examples
/// ```
/// use luna::intent::parse::parse_intent;
///
/// let action = parse_intent("open safari").unwrap();
/// ```
pub fn parse_intent(input: &str) -> Result<Action, ParseError> {
    // Normalize input: lowercase, trim, collapse whitespace
    let normalized = normalize_input(input);

    // Try each parsing rule in order
    if let Some(action) = try_parse_mute(&normalized) {
        return Ok(action);
    }

    if let Some(action) = try_parse_unmute(&normalized) {
        return Ok(action);
    }

    if let Some(result) = try_parse_volume(&normalized) {
        return result;
    }

    if let Some(result) = try_parse_open(&normalized) {
        return result;
    }

    // No rule matched
    Err(ParseError::UnrecognizedCommand(input.to_string()))
}

/// Normalize input for consistent parsing.
fn normalize_input(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Try to parse "mute" command.
fn try_parse_mute(input: &str) -> Option<Action> {
    if input == "mute" {
        Some(Action::Mute)
    } else {
        None
    }
}

/// Try to parse "unmute" command.
fn try_parse_unmute(input: &str) -> Option<Action> {
    if input == "unmute" {
        Some(Action::Unmute)
    } else {
        None
    }
}

/// Try to parse "set volume to <level>" command.
fn try_parse_volume(input: &str) -> Option<Result<Action, ParseError>> {
    // Match patterns like "set volume to 40" or "volume 40"
    let patterns = [
        "set volume to ",
        "set volume ",
        "volume to ",
        "volume ",
    ];

    for pattern in patterns {
        if let Some(rest) = input.strip_prefix(pattern) {
            let level_str = rest.trim();
            if level_str.is_empty() {
                return Some(Err(ParseError::MissingVolumeLevel));
            }

            match level_str.parse::<u8>() {
                Ok(level) if level <= 100 => {
                    return Some(Ok(Action::SetVolume { level }));
                }
                Ok(level) => {
                    return Some(Err(ParseError::InvalidVolume(format!(
                        "{} (exceeds 100)",
                        level
                    ))));
                }
                Err(_) => {
                    return Some(Err(ParseError::InvalidVolume(level_str.to_string())));
                }
            }
        }
    }

    None
}

/// Try to parse "open <target>" command.
fn try_parse_open(input: &str) -> Option<Result<Action, ParseError>> {
    if let Some(rest) = input.strip_prefix("open ") {
        let target = rest.trim();
        if target.is_empty() {
            return Some(Err(ParseError::MissingOpenTarget));
        }

        // Check if it's a URL
        if is_url(target) {
            // Preserve original case for URLs
            return Some(Ok(Action::OpenUrl {
                url: target.to_string(),
            }));
        }

        // It's an app name - capitalize first letter of each word
        let app_name = capitalize_app_name(target);
        return Some(Ok(Action::OpenApp { name: app_name }));
    }

    None
}

/// Check if the target looks like a URL.
fn is_url(target: &str) -> bool {
    target.starts_with("http://")
        || target.starts_with("https://")
        || target.contains("://")
        || (target.contains('.') && !target.contains(' '))
}

/// Capitalize the first letter of each word for app names.
fn capitalize_app_name(name: &str) -> String {
    name.split_whitespace()
        .map(|word| {
            let mut chars: Vec<char> = word.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_uppercase().next().unwrap_or(*first);
            }
            chars.into_iter().collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_open_app() {
        let action = parse_intent("open safari").unwrap();
        assert_eq!(action, Action::OpenApp { name: "Safari".to_string() });
    }

    #[test]
    fn test_parse_open_app_with_spaces() {
        let action = parse_intent("open visual studio code").unwrap();
        assert_eq!(action, Action::OpenApp { name: "Visual Studio Code".to_string() });
    }

    #[test]
    fn test_parse_open_url_https() {
        let action = parse_intent("open https://google.com").unwrap();
        assert_eq!(action, Action::OpenUrl { url: "https://google.com".to_string() });
    }

    #[test]
    fn test_parse_open_url_http() {
        let action = parse_intent("open http://example.com").unwrap();
        assert_eq!(action, Action::OpenUrl { url: "http://example.com".to_string() });
    }

    #[test]
    fn test_parse_set_volume() {
        let action = parse_intent("set volume to 40").unwrap();
        assert_eq!(action, Action::SetVolume { level: 40 });
    }

    #[test]
    fn test_parse_volume_shorthand() {
        let action = parse_intent("volume 50").unwrap();
        assert_eq!(action, Action::SetVolume { level: 50 });
    }

    #[test]
    fn test_parse_mute() {
        let action = parse_intent("mute").unwrap();
        assert_eq!(action, Action::Mute);
    }

    #[test]
    fn test_parse_unmute() {
        let action = parse_intent("unmute").unwrap();
        assert_eq!(action, Action::Unmute);
    }

    #[test]
    fn test_parse_with_extra_whitespace() {
        let action = parse_intent("  open   safari  ").unwrap();
        assert_eq!(action, Action::OpenApp { name: "Safari".to_string() });
    }

    #[test]
    fn test_parse_case_insensitive() {
        let action = parse_intent("OPEN SAFARI").unwrap();
        assert_eq!(action, Action::OpenApp { name: "Safari".to_string() });
    }

    #[test]
    fn test_parse_invalid_volume() {
        let result = parse_intent("set volume to abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_volume_out_of_range() {
        let result = parse_intent("set volume to 150");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unrecognized() {
        let result = parse_intent("do something random");
        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_input() {
        assert_eq!(normalize_input("  HELLO   WORLD  "), "hello world");
    }
}
