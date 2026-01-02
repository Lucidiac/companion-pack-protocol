//! Trait for implementing gamepack handlers.

use crate::types::{GameEvent, GameStatus, InitResponse, MatchData};

/// Result type for gamepack operations.
pub type GamepackResult<T> = Result<T, GamepackError>;

/// Error type for gamepack operations.
#[derive(Debug, Clone)]
pub struct GamepackError {
    /// Error message
    pub message: String,
    /// Optional error code
    pub code: Option<String>,
}

impl GamepackError {
    /// Create a new error.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
        }
    }

    /// Create an error with a code.
    pub fn with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: Some(code.into()),
        }
    }
}

impl std::fmt::Display for GamepackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = &self.code {
            write!(f, "[{}] {}", code, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for GamepackError {}

impl From<String> for GamepackError {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for GamepackError {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Trait that gamepacks implement for clean integration with the main daemon.
///
/// Implement this trait and pass it to [`run_gamepack`](crate::run_gamepack)
/// to create a fully functional gamepack daemon.
///
/// # Example
///
/// ```rust,ignore
/// use companion_pack_protocol::{GamepackHandler, GamepackResult};
/// use companion_pack_protocol::{InitResponse, GameStatus, GameEvent, MatchData};
///
/// struct MyGameIntegration { /* ... */ }
///
/// impl GamepackHandler for MyGameIntegration {
///     fn init(&mut self) -> GamepackResult<InitResponse> {
///         Ok(InitResponse {
///             game_id: 1,
///             slug: "my-game".to_string(),
///             protocol_version: 1,
///         })
///     }
///
///     fn detect_running(&self) -> bool {
///         // Check if game process is running
///         false
///     }
///
///     fn get_status(&self) -> GameStatus {
///         GameStatus::disconnected()
///     }
///
///     fn poll_events(&mut self) -> Vec<GameEvent> {
///         vec![]
///     }
///
///     fn get_live_data(&self) -> Option<serde_json::Value> {
///         None
///     }
///
///     fn on_session_start(&mut self) -> Option<serde_json::Value> {
///         None
///     }
///
///     fn on_session_end(&mut self, _context: serde_json::Value) -> Option<MatchData> {
///         None
///     }
///
///     fn shutdown(&mut self) {}
/// }
/// ```
pub trait GamepackHandler {
    /// Initialize the integration.
    ///
    /// Called once when the gamepack process starts. Return metadata about
    /// this game integration.
    fn init(&mut self) -> GamepackResult<InitResponse>;

    /// Check if the game client/process is running.
    ///
    /// Called periodically by the daemon to detect when the game launches
    /// or exits.
    fn detect_running(&self) -> bool;

    /// Get current connection/game status.
    ///
    /// Return the current state of the game connection (client connected,
    /// game phase, whether in an active match, etc.).
    fn get_status(&self) -> GameStatus;

    /// Poll for new game events.
    ///
    /// Called frequently (every ~500ms) during active games. Return any
    /// new events since the last poll that should trigger clip capture.
    fn poll_events(&mut self) -> Vec<GameEvent>;

    /// Get live match data.
    ///
    /// Return current in-game statistics for display in the UI (KDA, gold,
    /// objectives, etc.). Return `None` if not in a game.
    fn get_live_data(&self) -> Option<serde_json::Value>;

    /// Called when a game session starts.
    ///
    /// The daemon calls this when transitioning to an in-game state.
    /// Return optional context data that will be passed to `on_session_end`.
    fn on_session_start(&mut self) -> Option<serde_json::Value>;

    /// Called when a game session ends.
    ///
    /// Return the complete match data for storage in the database.
    /// The `context` parameter contains data returned from `on_session_start`.
    fn on_session_end(&mut self, context: serde_json::Value) -> Option<MatchData>;

    /// Called on graceful shutdown.
    ///
    /// Clean up any resources before the process exits.
    fn shutdown(&mut self);

    /// Resolve an icon URL for an event type.
    ///
    /// Called when the UI needs an icon for a discovered event type that
    /// doesn't have one in the seed data. Return `None` if no icon can be
    /// found for this event key.
    ///
    /// Default implementation returns `None`.
    fn resolve_event_icon(&self, _event_key: &str) -> Option<String> {
        None
    }
}
