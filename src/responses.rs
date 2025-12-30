//! Responses sent from gamepacks to the main daemon.

use serde::{Deserialize, Serialize};

use crate::types::GameEvent;

/// Responses from a gamepack to the main daemon.
///
/// Each response includes the `request_id` from the corresponding command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GamepackResponse {
    /// Initialization complete.
    Initialized {
        request_id: String,
        /// Unique identifier for this game
        game_id: i32,
        /// URL-friendly slug (e.g., "league", "valorant")
        slug: String,
        /// Protocol version this pack implements
        protocol_version: u32,
    },

    /// Game running status.
    RunningStatus {
        request_id: String,
        /// Whether the game process is running
        running: bool,
    },

    /// Current game status.
    GameStatus {
        request_id: String,
        /// Whether connected to the game's API/client
        connected: bool,
        /// Human-readable connection status
        connection_status: String,
        /// Current game phase (e.g., "Lobby", "InProgress", "PostGame")
        game_phase: Option<String>,
        /// Whether the player is actively in a game
        is_in_game: bool,
    },

    /// Polled events.
    Events {
        request_id: String,
        /// New game events since last poll
        events: Vec<GameEvent>,
    },

    /// Live match data.
    LiveData {
        request_id: String,
        /// Game-specific live match data (stats, scores, etc.)
        data: Option<serde_json::Value>,
    },

    /// Session started acknowledgment.
    SessionStarted {
        request_id: String,
        /// Optional context data to pass to session_end
        context: Option<serde_json::Value>,
    },

    /// Session ended with match data.
    SessionEnded {
        request_id: String,
        /// Complete match data for database storage
        match_data: Option<serde_json::Value>,
    },

    /// Error response.
    Error {
        request_id: String,
        /// Human-readable error message
        message: String,
        /// Optional error code for programmatic handling
        code: Option<String>,
    },

    /// Shutdown complete.
    ShutdownComplete { request_id: String },

    /// Event icon resolved.
    EventIconResolved {
        request_id: String,
        /// The event key that was requested
        event_key: String,
        /// The resolved icon URL, or None if no icon could be found
        icon_url: Option<String>,
    },
}

impl GamepackResponse {
    /// Get the request_id from any response variant.
    pub fn request_id(&self) -> &str {
        match self {
            Self::Initialized { request_id, .. } => request_id,
            Self::RunningStatus { request_id, .. } => request_id,
            Self::GameStatus { request_id, .. } => request_id,
            Self::Events { request_id, .. } => request_id,
            Self::LiveData { request_id, .. } => request_id,
            Self::SessionStarted { request_id, .. } => request_id,
            Self::SessionEnded { request_id, .. } => request_id,
            Self::Error { request_id, .. } => request_id,
            Self::ShutdownComplete { request_id, .. } => request_id,
            Self::EventIconResolved { request_id, .. } => request_id,
        }
    }

    /// Create an error response.
    pub fn error(request_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error {
            request_id: request_id.into(),
            message: message.into(),
            code: None,
        }
    }

    /// Create an error response with a code.
    pub fn error_with_code(
        request_id: impl Into<String>,
        message: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self::Error {
            request_id: request_id.into(),
            message: message.into(),
            code: Some(code.into()),
        }
    }
}
