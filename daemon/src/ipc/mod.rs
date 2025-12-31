//! IPC module for daemon-UI communication
//!
//! Uses newline-delimited JSON (JSONL) with a unified message envelope.
//! See `message.rs` for the envelope format and message types.

mod handlers;
mod message;
mod server;

// Re-export the public API
pub use message::{
    message_types, DictationPayload, Envelope, HeartbeatPayload, InvalidModePayload,
    InvalidModeTransitionPayload, MalformedMessagePayload, MessageKind, Mode,
    ModeChangedPayload, RawEnvelope, SetModePayload, UnknownCommandPayload,
};
pub use server::Server;
