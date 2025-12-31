//! Unix domain socket server for IPC
//!
//! Uses newline-delimited JSON (JSONL) for all communication.
//! One task per client connection, with heartbeat and streaming support.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use super::handlers::{handle_command, make_parse_error, CommandResult};
use super::message::{
    message_types, DictationPayload, Envelope, HeartbeatPayload, MessageKind, Mode, RawEnvelope,
};

// =============================================================================
// Configuration
// =============================================================================

/// Heartbeat interval
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// Simulated dictation interval (for fake streaming)
const DICTATION_INTERVAL: Duration = Duration::from_millis(500);

/// Simulated dictation phrases
const DICTATION_PHRASES: &[&str] = &[
    "hello",
    "hello this",
    "hello this is",
    "hello this is a",
    "hello this is a test",
];

// =============================================================================
// Server
// =============================================================================

/// IPC Server handling client connections
pub struct Server {
    socket_path: PathBuf,
    listener: Option<UnixListener>,
    state: Arc<RwLock<ServerState>>,
    shutdown_tx: broadcast::Sender<()>,
}

/// Shared server state
struct ServerState {
    /// Current operating mode
    current_mode: Mode,
    /// Server start time for uptime calculation
    start_time: Instant,
}

impl Server {
    /// Create a new IPC server
    pub fn new(socket_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = socket_path.parent() {
            std::fs::create_dir_all(parent).context("failed to create socket directory")?;
        }

        // Remove stale socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(socket_path).context("failed to remove stale socket")?;
        }

        let listener =
            UnixListener::bind(socket_path).context("failed to bind Unix socket")?;

        // Set socket permissions to owner-only (0600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o600))?;
        }

        let (shutdown_tx, _) = broadcast::channel(1);

        let state = Arc::new(RwLock::new(ServerState {
            current_mode: Mode::default(),
            start_time: Instant::now(),
        }));

        info!(?socket_path, "IPC server listening");

        Ok(Self {
            socket_path: socket_path.to_owned(),
            listener: Some(listener),
            state,
            shutdown_tx,
        })
    }

    /// Create server with initial mode
    pub fn with_mode(socket_path: &Path, mode: Mode) -> Result<Self> {
        let server = Self::new(socket_path)?;
        {
            // Set initial mode synchronously since we're in construction
            let state = server.state.clone();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let mut s = state.write().await;
                    s.current_mode = mode;
                });
            });
        }
        Ok(server)
    }

    /// Get current mode
    pub async fn get_mode(&self) -> Mode {
        self.state.read().await.current_mode
    }

    /// Set current mode (for external state machine updates)
    pub async fn set_mode(&self, mode: Mode) {
        let mut state = self.state.write().await;
        if state.current_mode != mode {
            info!(from = %state.current_mode, to = %mode, "mode updated externally");
            state.current_mode = mode;
        }
    }

    /// Run the server, accepting connections
    pub async fn run(&self) -> Result<()> {
        let listener = self.listener.as_ref().context("server not initialized")?;

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    info!("client connected");
                    let state = Arc::clone(&self.state);
                    let mut shutdown_rx = self.shutdown_tx.subscribe();

                    tokio::spawn(async move {
                        tokio::select! {
                            result = Self::handle_client(stream, state) => {
                                if let Err(e) = result {
                                    warn!(?e, "client handler error");
                                }
                                info!("client disconnected");
                            }
                            _ = shutdown_rx.recv() => {
                                debug!("client handler shutting down");
                            }
                        }
                    });
                }
                Err(e) => {
                    error!(?e, "accept error");
                }
            }
        }
    }

    /// Handle a single client connection
    async fn handle_client(stream: UnixStream, state: Arc<RwLock<ServerState>>) -> Result<()> {
        let (reader, writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let writer = Arc::new(tokio::sync::Mutex::new(writer));

        let mut line = String::new();
        let mut heartbeat_timer = interval(HEARTBEAT_INTERVAL);
        let mut dictation_timer = interval(DICTATION_INTERVAL);
        let mut dictation_phrase_index = 0;
        let mut stream_id = 0u64;

        loop {
            tokio::select! {
                // Read incoming commands
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            // EOF - client disconnected
                            return Ok(());
                        }
                        Ok(_) => {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                Self::process_line(trimmed, &state, &writer).await?;
                            }
                            line.clear();
                        }
                        Err(e) => {
                            warn!(?e, "read error");
                            return Err(e.into());
                        }
                    }
                }

                // Send heartbeat
                _ = heartbeat_timer.tick() => {
                    let uptime_ms = {
                        let s = state.read().await;
                        s.start_time.elapsed().as_millis() as u64
                    };

                    let heartbeat = Envelope::event(
                        "sys",
                        message_types::HEARTBEAT,
                        HeartbeatPayload { uptime_ms },
                    );

                    Self::send_message(&writer, &heartbeat).await?;
                    debug!(uptime_ms, "sent heartbeat");
                }

                // Simulate dictation streaming when in dictation mode
                _ = dictation_timer.tick() => {
                    let current_mode = {
                        state.read().await.current_mode
                    };

                    if current_mode == Mode::Dictation {
                        let phrase = DICTATION_PHRASES[dictation_phrase_index];
                        let is_final = dictation_phrase_index == DICTATION_PHRASES.len() - 1;
                        let stream_id_str = format!("stream-{}", stream_id);

                        let message_type = if is_final {
                            message_types::DICTATION_FINAL
                        } else {
                            message_types::DICTATION_PARTIAL
                        };

                        let event = Envelope::event(
                            &stream_id_str,
                            message_type,
                            DictationPayload { text: phrase.to_string() },
                        );

                        Self::send_message(&writer, &event).await?;
                        debug!(text = phrase, is_final, "sent dictation event");

                        if is_final {
                            // Reset for next cycle
                            dictation_phrase_index = 0;
                            stream_id += 1;
                        } else {
                            dictation_phrase_index += 1;
                        }
                    } else {
                        // Reset when not in dictation mode
                        dictation_phrase_index = 0;
                    }
                }
            }
        }
    }

    /// Process a single JSON line
    async fn process_line(
        line: &str,
        state: &Arc<RwLock<ServerState>>,
        writer: &Arc<tokio::sync::Mutex<tokio::net::unix::OwnedWriteHalf>>,
    ) -> Result<()> {
        debug!(line, "received message");

        // Parse the raw envelope
        let raw = match RawEnvelope::from_line(line) {
            Ok(r) => r,
            Err(e) => {
                warn!(?e, "failed to parse message");
                let error_json = make_parse_error("unknown", &format!("parse error: {}", e));
                Self::send_line(writer, &error_json).await?;
                return Ok(());
            }
        };

        // Only process commands
        if raw.kind != MessageKind::Command {
            debug!(kind = ?raw.kind, "ignoring non-command message");
            return Ok(());
        }

        // Get current mode and process command
        let response = {
            let mut s = state.write().await;
            handle_command(&raw, &mut s.current_mode)
        };

        // Send response if any
        if let CommandResult::Response(json) = response {
            Self::send_line(writer, &json).await?;
        }

        Ok(())
    }

    /// Send a serializable message as JSONL
    async fn send_message<T: serde::Serialize>(
        writer: &Arc<tokio::sync::Mutex<tokio::net::unix::OwnedWriteHalf>>,
        message: &T,
    ) -> Result<()> {
        let json = serde_json::to_string(message)?;
        Self::send_line(writer, &json).await
    }

    /// Send a raw JSON string as a line
    async fn send_line(
        writer: &Arc<tokio::sync::Mutex<tokio::net::unix::OwnedWriteHalf>>,
        json: &str,
    ) -> Result<()> {
        let mut w = writer.lock().await;
        w.write_all(json.as_bytes()).await?;
        w.write_all(b"\n").await?;
        w.flush().await?;
        Ok(())
    }

    /// Gracefully shutdown the server
    pub async fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());

        // Remove socket file
        if self.socket_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.socket_path) {
                warn!(?e, "failed to remove socket file");
            }
        }

        info!("IPC server shutdown complete");
    }
}

// For compatibility with existing main.rs
impl Server {
    /// Create a new IPC server with event subscription (compatibility shim)
    pub fn with_events(
        socket_path: &Path,
        _event_rx: broadcast::Receiver<crate::events::StateEvent>,
    ) -> Result<Self> {
        Self::new(socket_path)
    }

    /// Update the current mode in server state (compatibility with main.rs)
    pub async fn set_state(&self, state: crate::state::State) {
        self.set_mode(state.into()).await;
    }
}
