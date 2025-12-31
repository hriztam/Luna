//! IPC command handlers
//!
//! Processes incoming commands and produces responses.
//! All validation and state mutation logic is centralized here.

use tracing::{debug, info, warn};

use super::message::{
    message_types, Envelope, InvalidModePayload, InvalidModeTransitionPayload,
    MalformedMessagePayload, Mode, ModeChangedPayload, RawEnvelope, SetModePayload,
    UnknownCommandPayload,
};

// =============================================================================
// Command Handler Result
// =============================================================================

/// Result of processing a command
pub enum CommandResult {
    /// Send a single response back to the client
    Response(String),
    /// No response needed (already handled)
    NoResponse,
}

// =============================================================================
// Transition Validation
// =============================================================================

/// Checks if a mode transition is valid
/// 
/// Valid transitions (per spec):
/// - idle -> any mode
/// - any mode -> idle (always allowed)
/// - dictation -> intelligent (upgrade allowed while holding keys)
/// - other transitions are invalid
pub fn is_valid_transition(from: Mode, to: Mode) -> bool {
    // Same mode is always valid (no-op)
    if from == to {
        return true;
    }
    
    // Transition to idle is always valid
    if to == Mode::Idle {
        return true;
    }
    
    // From idle, can go to any mode
    if from == Mode::Idle {
        return true;
    }
    
    // Upgrade from dictation to intelligent is valid
    if from == Mode::Dictation && to == Mode::Intelligent {
        return true;
    }
    
    // All other transitions are invalid
    // - dictation -> agent (must go through idle)
    // - intelligent -> dictation (must release and re-press)
    // - intelligent -> agent (must go through idle)
    // - agent -> dictation (agent mode is toggle, must toggle off first)
    // - agent -> intelligent (agent mode is toggle, must toggle off first)
    false
}

// =============================================================================
// Command Processing
// =============================================================================

/// Process a raw envelope and produce a response
/// 
/// Returns a JSON string to send back to the client.
/// The current mode is passed by reference and may be updated.
pub fn handle_command(raw: &RawEnvelope, current_mode: &mut Mode) -> CommandResult {
    debug!(
        id = %raw.id,
        kind = ?raw.kind,
        message_type = %raw.message_type,
        "processing command"
    );

    let response_json = match raw.message_type.as_str() {
        message_types::SET_MODE => handle_set_mode(raw, current_mode),
        unknown => {
            warn!(command = %unknown, "unknown command type");
            make_unknown_command_error(&raw.id, unknown)
        }
    };

    CommandResult::Response(response_json)
}

/// Handle SET_MODE command
fn handle_set_mode(raw: &RawEnvelope, current_mode: &mut Mode) -> String {
    // Parse the payload
    let payload: SetModePayload = match serde_json::from_value(raw.payload.clone()) {
        Ok(p) => p,
        Err(e) => {
            warn!(error = %e, "failed to parse SET_MODE payload");
            return make_malformed_message_error(&raw.id, &format!("invalid SET_MODE payload: {}", e));
        }
    };

    let target_mode = payload.mode;
    let from_mode = *current_mode;

    info!(
        from = %from_mode,
        to = %target_mode,
        "SET_MODE request"
    );

    // Validate the transition
    if !is_valid_transition(from_mode, target_mode) {
        info!(
            from = %from_mode,
            to = %target_mode,
            "rejecting invalid mode transition"
        );
        return make_invalid_transition_error(&raw.id, from_mode, target_mode);
    }

    // Apply the transition
    *current_mode = target_mode;
    info!(mode = %target_mode, "mode changed successfully");

    // Build success response
    let response = Envelope::response(
        &raw.id,
        message_types::MODE_CHANGED,
        ModeChangedPayload { mode: target_mode },
    );

    serde_json::to_string(&response).unwrap_or_else(|e| {
        make_malformed_message_error(&raw.id, &format!("serialization error: {}", e))
    })
}

// =============================================================================
// Error Response Helpers
// =============================================================================

fn make_invalid_transition_error(request_id: &str, from: Mode, to: Mode) -> String {
    let response = Envelope::error(
        request_id,
        message_types::INVALID_MODE_TRANSITION,
        InvalidModeTransitionPayload { from, to },
    );
    serde_json::to_string(&response).expect("error serialization should not fail")
}

fn make_invalid_mode_error(request_id: &str, provided: &str) -> String {
    let response = Envelope::error(
        request_id,
        message_types::INVALID_MODE,
        InvalidModePayload { provided: provided.to_string() },
    );
    serde_json::to_string(&response).expect("error serialization should not fail")
}

fn make_malformed_message_error(request_id: &str, reason: &str) -> String {
    let response = Envelope::error(
        request_id,
        message_types::MALFORMED_MESSAGE,
        MalformedMessagePayload { reason: reason.to_string() },
    );
    serde_json::to_string(&response).expect("error serialization should not fail")
}

fn make_unknown_command_error(request_id: &str, command: &str) -> String {
    let response = Envelope::error(
        request_id,
        message_types::UNKNOWN_COMMAND,
        UnknownCommandPayload { command: command.to_string() },
    );
    serde_json::to_string(&response).expect("error serialization should not fail")
}

/// Create a malformed message error for parse failures (used by server)
pub fn make_parse_error(request_id: &str, reason: &str) -> String {
    make_malformed_message_error(request_id, reason)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::message::MessageKind;

    fn make_set_mode_command(id: &str, mode: &str) -> RawEnvelope {
        let json = format!(
            r#"{{"id":"{}","kind":"command","type":"SET_MODE","payload":{{"mode":"{}"}}}}"#,
            id, mode
        );
        RawEnvelope::from_line(&json).unwrap()
    }

    #[test]
    fn test_valid_transitions() {
        // From idle
        assert!(is_valid_transition(Mode::Idle, Mode::Dictation));
        assert!(is_valid_transition(Mode::Idle, Mode::Intelligent));
        assert!(is_valid_transition(Mode::Idle, Mode::Agent));
        
        // To idle
        assert!(is_valid_transition(Mode::Dictation, Mode::Idle));
        assert!(is_valid_transition(Mode::Intelligent, Mode::Idle));
        assert!(is_valid_transition(Mode::Agent, Mode::Idle));
        
        // Dictation upgrade
        assert!(is_valid_transition(Mode::Dictation, Mode::Intelligent));
        
        // Same mode
        assert!(is_valid_transition(Mode::Dictation, Mode::Dictation));
    }

    #[test]
    fn test_invalid_transitions() {
        // Cannot go directly between non-idle modes (except dictation->intelligent)
        assert!(!is_valid_transition(Mode::Dictation, Mode::Agent));
        assert!(!is_valid_transition(Mode::Intelligent, Mode::Dictation));
        assert!(!is_valid_transition(Mode::Intelligent, Mode::Agent));
        assert!(!is_valid_transition(Mode::Agent, Mode::Dictation));
        assert!(!is_valid_transition(Mode::Agent, Mode::Intelligent));
    }

    #[test]
    fn test_handle_set_mode_success() {
        let mut current_mode = Mode::Idle;
        let raw = make_set_mode_command("1", "dictation");
        
        let result = handle_command(&raw, &mut current_mode);
        
        assert!(matches!(result, CommandResult::Response(_)));
        assert_eq!(current_mode, Mode::Dictation);
        
        if let CommandResult::Response(json) = result {
            assert!(json.contains("MODE_CHANGED"));
            assert!(json.contains("\"kind\":\"response\""));
            assert!(json.contains("\"mode\":\"dictation\""));
        }
    }

    #[test]
    fn test_handle_set_mode_invalid_transition() {
        let mut current_mode = Mode::Agent;
        let raw = make_set_mode_command("1", "dictation");
        
        let result = handle_command(&raw, &mut current_mode);
        
        // Mode should not change
        assert_eq!(current_mode, Mode::Agent);
        
        if let CommandResult::Response(json) = result {
            assert!(json.contains("INVALID_MODE_TRANSITION"));
            assert!(json.contains("\"kind\":\"error\""));
            assert!(json.contains("\"from\":\"agent\""));
            assert!(json.contains("\"to\":\"dictation\""));
        }
    }

    #[test]
    fn test_handle_unknown_command() {
        let mut current_mode = Mode::Idle;
        let json = r#"{"id":"1","kind":"command","type":"UNKNOWN_THING","payload":{}}"#;
        let raw = RawEnvelope::from_line(json).unwrap();
        
        let result = handle_command(&raw, &mut current_mode);
        
        if let CommandResult::Response(json) = result {
            assert!(json.contains("UNKNOWN_COMMAND"));
            assert!(json.contains("\"kind\":\"error\""));
        }
    }
}
