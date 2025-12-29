//! IPC message types using unified envelope format
//!
//! All messages use newline-delimited JSON (JSONL) with a common envelope:
//! ```json
//! {"id": "uuid", "kind": "command|event|response|error", "type": "MESSAGE_TYPE", "payload": {...}}
//! ```

use serde::{Deserialize, Serialize};

use crate::state::State;

// =============================================================================
// Message Envelope
// =============================================================================

/// The kind of message being sent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    /// Client -> Server: Request an action
    Command,
    /// Server -> Client: Push notification (streaming, heartbeat)
    Event,
    /// Server -> Client: Success response to a command
    Response,
    /// Server -> Client: Error response to a command
    Error,
}

/// Universal message envelope
/// 
/// Every IPC message follows this structure. The payload is generic
/// to allow different content based on message type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<P> {
    /// Unique identifier for request/response correlation
    pub id: String,
    /// Message classification
    pub kind: MessageKind,
    /// Specific message type (e.g., "SET_MODE", "MODE_CHANGED")
    #[serde(rename = "type")]
    pub message_type: String,
    /// Type-specific payload
    pub payload: P,
}

impl<P> Envelope<P> {
    /// Create a new envelope
    pub fn new(id: impl Into<String>, kind: MessageKind, message_type: impl Into<String>, payload: P) -> Self {
        Self {
            id: id.into(),
            kind,
            message_type: message_type.into(),
            payload,
        }
    }

    /// Create a response envelope for a given request ID
    pub fn response(request_id: impl Into<String>, message_type: impl Into<String>, payload: P) -> Self {
        Self::new(request_id, MessageKind::Response, message_type, payload)
    }

    /// Create an error envelope for a given request ID
    pub fn error(request_id: impl Into<String>, message_type: impl Into<String>, payload: P) -> Self {
        Self::new(request_id, MessageKind::Error, message_type, payload)
    }

    /// Create an event envelope with a new ID
    pub fn event(id: impl Into<String>, message_type: impl Into<String>, payload: P) -> Self {
        Self::new(id, MessageKind::Event, message_type, payload)
    }
}

// =============================================================================
// Message Type Constants
// =============================================================================

pub mod message_types {
    // Commands (Client -> Server)
    pub const SET_MODE: &str = "SET_MODE";

    // Responses (Server -> Client)
    pub const MODE_CHANGED: &str = "MODE_CHANGED";

    // Errors (Server -> Client)
    pub const INVALID_MODE_TRANSITION: &str = "INVALID_MODE_TRANSITION";
    pub const INVALID_MODE: &str = "INVALID_MODE";
    pub const MALFORMED_MESSAGE: &str = "MALFORMED_MESSAGE";
    pub const UNKNOWN_COMMAND: &str = "UNKNOWN_COMMAND";

    // Events (Server -> Client)
    pub const DICTATION_PARTIAL: &str = "DICTATION_PARTIAL";
    pub const DICTATION_FINAL: &str = "DICTATION_FINAL";
    pub const HEARTBEAT: &str = "HEARTBEAT";
}

// =============================================================================
// Mode
// =============================================================================

/// Operating modes of the daemon
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Idle,
    Dictation,
    Intelligent,
    Agent,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Idle
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Idle => write!(f, "idle"),
            Mode::Dictation => write!(f, "dictation"),
            Mode::Intelligent => write!(f, "intelligent"),
            Mode::Agent => write!(f, "agent"),
        }
    }
}

impl From<State> for Mode {
    fn from(state: State) -> Self {
        match state {
            State::Idle => Mode::Idle,
            State::DictationActive => Mode::Dictation,
            State::IntelligentActive => Mode::Intelligent,
            State::AgentActive => Mode::Agent,
        }
    }
}

impl Mode {
    /// Parse a mode from a string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "idle" => Some(Mode::Idle),
            "dictation" => Some(Mode::Dictation),
            "intelligent" => Some(Mode::Intelligent),
            "agent" => Some(Mode::Agent),
            _ => None,
        }
    }
}

// =============================================================================
// Command Payloads (Client -> Server)
// =============================================================================

/// Payload for SET_MODE command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetModePayload {
    pub mode: Mode,
}

// =============================================================================
// Response Payloads (Server -> Client)
// =============================================================================

/// Payload for MODE_CHANGED response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeChangedPayload {
    pub mode: Mode,
}

// =============================================================================
// Error Payloads (Server -> Client)
// =============================================================================

/// Payload for INVALID_MODE_TRANSITION error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidModeTransitionPayload {
    pub from: Mode,
    pub to: Mode,
}

/// Payload for INVALID_MODE error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidModePayload {
    pub provided: String,
}

/// Payload for MALFORMED_MESSAGE error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalformedMessagePayload {
    pub reason: String,
}

/// Payload for UNKNOWN_COMMAND error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownCommandPayload {
    pub command: String,
}

// =============================================================================
// Event Payloads (Server -> Client)
// =============================================================================

/// Payload for DICTATION_PARTIAL and DICTATION_FINAL events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationPayload {
    pub text: String,
}

/// Payload for HEARTBEAT event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    pub uptime_ms: u64,
}

// =============================================================================
// Raw Message Parsing
// =============================================================================

/// Raw envelope for initial parsing (payload as generic Value)
#[derive(Debug, Clone, Deserialize)]
pub struct RawEnvelope {
    pub id: String,
    pub kind: MessageKind,
    #[serde(rename = "type")]
    pub message_type: String,
    pub payload: serde_json::Value,
}

impl RawEnvelope {
    /// Parse from a JSON line
    pub fn from_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_serialization() {
        let envelope = Envelope::new(
            "test-1",
            MessageKind::Command,
            message_types::SET_MODE,
            SetModePayload { mode: Mode::Dictation },
        );
        
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("\"id\":\"test-1\""));
        assert!(json.contains("\"kind\":\"command\""));
        assert!(json.contains("\"type\":\"SET_MODE\""));
        assert!(json.contains("\"mode\":\"dictation\""));
    }

    #[test]
    fn test_raw_envelope_parsing() {
        let json = r#"{"id":"1","kind":"command","type":"SET_MODE","payload":{"mode":"dictation"}}"#;
        let raw = RawEnvelope::from_line(json).unwrap();
        
        assert_eq!(raw.id, "1");
        assert_eq!(raw.kind, MessageKind::Command);
        assert_eq!(raw.message_type, "SET_MODE");
    }

    #[test]
    fn test_mode_response_envelope() {
        let envelope = Envelope::response(
            "1",
            message_types::MODE_CHANGED,
            ModeChangedPayload { mode: Mode::Dictation },
        );
        
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("\"kind\":\"response\""));
        assert!(json.contains("\"type\":\"MODE_CHANGED\""));
    }

    #[test]
    fn test_error_envelope() {
        let envelope = Envelope::error(
            "1",
            message_types::INVALID_MODE_TRANSITION,
            InvalidModeTransitionPayload {
                from: Mode::Dictation,
                to: Mode::Agent,
            },
        );
        
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("\"kind\":\"error\""));
        assert!(json.contains("\"type\":\"INVALID_MODE_TRANSITION\""));
        assert!(json.contains("\"from\":\"dictation\""));
        assert!(json.contains("\"to\":\"agent\""));
    }

    #[test]
    fn test_heartbeat_event() {
        let envelope = Envelope::event(
            "sys",
            message_types::HEARTBEAT,
            HeartbeatPayload { uptime_ms: 123456 },
        );
        
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("\"kind\":\"event\""));
        assert!(json.contains("\"type\":\"HEARTBEAT\""));
        assert!(json.contains("\"uptime_ms\":123456"));
    }

    #[test]
    fn test_dictation_event() {
        let envelope = Envelope::event(
            "stream-1",
            message_types::DICTATION_PARTIAL,
            DictationPayload { text: "hello this is".to_string() },
        );
        
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("\"type\":\"DICTATION_PARTIAL\""));
        assert!(json.contains("\"text\":\"hello this is\""));
    }
}
