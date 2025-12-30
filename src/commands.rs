//! Commands sent from the main daemon to gamepacks.

use serde::{Deserialize, Serialize};

/// Commands sent from the main daemon to a gamepack.
///
/// Each command includes a `request_id` for correlating responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GamepackCommand {
    /// Initialize the integration.
    /// Expected response: `Initialized`
    Init { request_id: String },

    /// Check if the game client/process is running.
    /// Expected response: `RunningStatus`
    DetectRunning { request_id: String },

    /// Get current connection and game status.
    /// Expected response: `GameStatus`
    GetStatus { request_id: String },

    /// Poll for new game events (triggers for clip capture).
    /// Expected response: `Events`
    PollEvents { request_id: String },

    /// Get live match data for display in the UI.
    /// Expected response: `LiveData`
    GetLiveData { request_id: String },

    /// Notification that a game session has started.
    /// Expected response: `SessionStarted`
    SessionStart { request_id: String },

    /// Notification that a game session has ended.
    /// Expected response: `SessionEnded`
    SessionEnd {
        request_id: String,
        /// Context data captured at session start
        context: serde_json::Value,
    },

    /// Request graceful shutdown.
    /// Expected response: `ShutdownComplete`
    Shutdown { request_id: String },

    /// Request an icon URL for an event type.
    /// Used for discovered events that don't have icons in the seed data.
    /// Expected response: `EventIconResolved`
    ResolveEventIcon {
        request_id: String,
        /// The event key to resolve an icon for (e.g., "ChampionKill", "DragonKill")
        event_key: String,
    },
}

impl GamepackCommand {
    /// Get the request_id from any command variant.
    pub fn request_id(&self) -> &str {
        match self {
            Self::Init { request_id } => request_id,
            Self::DetectRunning { request_id } => request_id,
            Self::GetStatus { request_id } => request_id,
            Self::PollEvents { request_id } => request_id,
            Self::GetLiveData { request_id } => request_id,
            Self::SessionStart { request_id } => request_id,
            Self::SessionEnd { request_id, .. } => request_id,
            Self::Shutdown { request_id } => request_id,
            Self::ResolveEventIcon { request_id, .. } => request_id,
        }
    }
}
