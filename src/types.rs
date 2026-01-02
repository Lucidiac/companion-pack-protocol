//! Shared types used in the gamepack protocol.

use serde::{Deserialize, Serialize};

/// A game event that can trigger clip capture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    /// Event type identifier (e.g., "ChampionKill", "DragonKill")
    pub event_type: String,

    /// Timestamp in seconds from game start
    pub timestamp_secs: f64,

    /// Game-specific event data
    pub data: serde_json::Value,

    /// Seconds to capture before the event (overrides default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_capture_secs: Option<f64>,

    /// Seconds to capture after the event (overrides default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_capture_secs: Option<f64>,
}

impl GameEvent {
    /// Create a new game event with default capture times.
    pub fn new(event_type: impl Into<String>, timestamp_secs: f64, data: serde_json::Value) -> Self {
        Self {
            event_type: event_type.into(),
            timestamp_secs,
            data,
            pre_capture_secs: None,
            post_capture_secs: None,
        }
    }

    /// Set custom pre-capture duration.
    pub fn with_pre_capture(mut self, secs: f64) -> Self {
        self.pre_capture_secs = Some(secs);
        self
    }

    /// Set custom post-capture duration.
    pub fn with_post_capture(mut self, secs: f64) -> Self {
        self.post_capture_secs = Some(secs);
        self
    }
}

/// Response from the `init` command.
#[derive(Debug, Clone)]
pub struct InitResponse {
    /// Unique identifier for this game
    pub game_id: i32,
    /// URL-friendly slug (e.g., "league", "valorant")
    pub slug: String,
    /// Protocol version this pack implements
    pub protocol_version: u32,
}

/// Current game status returned by `get_status`.
#[derive(Debug, Clone, Default)]
pub struct GameStatus {
    /// Whether connected to the game's API/client
    pub connected: bool,
    /// Human-readable connection status
    pub connection_status: String,
    /// Current game phase (e.g., "Lobby", "InProgress", "PostGame")
    pub game_phase: Option<String>,
    /// Whether the player is actively in a game
    pub is_in_game: bool,
}

impl GameStatus {
    /// Create a disconnected status.
    pub fn disconnected() -> Self {
        Self {
            connected: false,
            connection_status: "Not connected".to_string(),
            game_phase: None,
            is_in_game: false,
        }
    }

    /// Create a connected status.
    pub fn connected(status: impl Into<String>) -> Self {
        Self {
            connected: true,
            connection_status: status.into(),
            game_phase: None,
            is_in_game: false,
        }
    }

    /// Set the game phase.
    pub fn with_phase(mut self, phase: impl Into<String>) -> Self {
        self.game_phase = Some(phase.into());
        self
    }

    /// Set whether in-game.
    pub fn in_game(mut self, in_game: bool) -> Self {
        self.is_in_game = in_game;
        self
    }
}

/// Match data returned when a game session ends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchData {
    /// Game slug (e.g., "league")
    pub game_slug: String,
    /// Game ID
    pub game_id: i32,
    /// Match result ("win", "loss", "remake")
    pub result: String,
    /// Game-specific match details
    pub details: serde_json::Value,
}

impl MatchData {
    /// Create new match data.
    pub fn new(
        game_slug: impl Into<String>,
        game_id: i32,
        result: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            game_slug: game_slug.into(),
            game_id,
            result: result.into(),
            details,
        }
    }
}
