//! Shared types used in the gamepack protocol.
//!
//! NOTE: All types here are GAME-AGNOSTIC. No League/TFT/etc specifics.
//! Each gamepack defines its own subpacks and column schemas in config.json.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

// ============================================================================
// MATCH DATA MESSAGES (Subpack Model)
// ============================================================================

/// Gamepack → Daemon: Write match data.
///
/// These messages allow gamepacks to emit match data during gameplay.
/// Each message includes a `subpack` field for multi-game packs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MatchDataMessage {
    /// Create or update match with stats (creates match if doesn't exist).
    ///
    /// The daemon will:
    /// 1. Create match row if it doesn't exist (lazy creation)
    /// 2. UPSERT stats to the summary table (`p{guid}_{subpack}_match_details`)
    WriteStats {
        /// Subpack index (0 = default, 1+ = additional subpacks)
        subpack: u8,
        /// Game's native match ID (used for deduplication and API lookups)
        external_match_id: String,
        /// When the match started (ISO 8601)
        #[serde(skip_serializing_if = "Option::is_none")]
        played_at: Option<String>,
        /// Match duration in seconds
        #[serde(skip_serializing_if = "Option::is_none")]
        duration_secs: Option<i32>,
        /// Match result: "win" | "loss" | "draw"
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<String>,
        /// Stats to write (keys must match columns declared in subpack's schema)
        stats: HashMap<String, serde_json::Value>,
    },

    /// Append events to match timeline.
    ///
    /// Events are saved to `p{guid}_{subpack}_match_timeline` with entry_type='event'.
    WriteEvents {
        /// Subpack index (0 = default, 1+ = additional subpacks)
        subpack: u8,
        /// Game's native match ID
        external_match_id: String,
        /// Events to append
        events: Vec<GameEvent>,
    },

    /// Mark match as complete (sets is_in_progress=0).
    ///
    /// Call this when:
    /// - Game ends naturally (gamepack detects end state)
    /// - Responding to `IsMatchInProgress` with still_playing=false
    SetComplete {
        /// Subpack index (0 = default, 1+ = additional subpacks)
        subpack: u8,
        /// Game's native match ID
        external_match_id: String,
        /// Where the final stats came from: "api" | "live_fallback"
        summary_source: String,
        /// Optional final stats to overwrite summary table
        #[serde(skip_serializing_if = "Option::is_none")]
        final_stats: Option<HashMap<String, serde_json::Value>>,
    },
}

impl MatchDataMessage {
    /// Create a WriteStats message.
    pub fn write_stats(
        subpack: u8,
        external_match_id: impl Into<String>,
        stats: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self::WriteStats {
            subpack,
            external_match_id: external_match_id.into(),
            played_at: None,
            duration_secs: None,
            result: None,
            stats,
        }
    }

    /// Create a WriteEvents message.
    pub fn write_events(
        subpack: u8,
        external_match_id: impl Into<String>,
        events: Vec<GameEvent>,
    ) -> Self {
        Self::WriteEvents {
            subpack,
            external_match_id: external_match_id.into(),
            events,
        }
    }

    /// Create a SetComplete message.
    pub fn set_complete(
        subpack: u8,
        external_match_id: impl Into<String>,
        summary_source: impl Into<String>,
    ) -> Self {
        Self::SetComplete {
            subpack,
            external_match_id: external_match_id.into(),
            summary_source: summary_source.into(),
            final_stats: None,
        }
    }

    /// Create a SetComplete message with final stats.
    pub fn set_complete_with_stats(
        subpack: u8,
        external_match_id: impl Into<String>,
        summary_source: impl Into<String>,
        final_stats: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self::SetComplete {
            subpack,
            external_match_id: external_match_id.into(),
            summary_source: summary_source.into(),
            final_stats: Some(final_stats),
        }
    }
}

// ============================================================================
// STALE MATCH RECOVERY
// ============================================================================

/// Daemon → Gamepack: Check if a match is still in progress.
///
/// Sent when the daemon needs to recover stale matches (e.g., after crash).
/// The gamepack should check if the game is actually still running.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsMatchInProgressRequest {
    /// Subpack index
    pub subpack: u8,
    /// Game's native match ID
    pub external_match_id: String,
}

/// Gamepack → Daemon: Response to IsMatchInProgress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsMatchInProgressResponse {
    /// Whether the game is actually still running
    pub still_playing: bool,
    /// If !still_playing, optionally provide SetComplete message with final stats
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_complete: Option<MatchDataMessage>,
}

impl IsMatchInProgressResponse {
    /// Create a response indicating the game is still playing.
    pub fn still_playing() -> Self {
        Self {
            still_playing: true,
            set_complete: None,
        }
    }

    /// Create a response indicating the game ended.
    pub fn ended() -> Self {
        Self {
            still_playing: false,
            set_complete: None,
        }
    }

    /// Create a response with final stats to apply.
    pub fn ended_with_stats(set_complete: MatchDataMessage) -> Self {
        Self {
            still_playing: false,
            set_complete: Some(set_complete),
        }
    }
}

// ============================================================================
// TIMELINE DATA
// ============================================================================

/// A single entry in the match timeline.
///
/// The timeline contains all match data (events, statistics, moments) in
/// chronological order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    /// Entry type: "event" | "statistic" | "moment"
    pub entry_type: String,
    /// Entry key: event type, "stats", or moment ID
    pub entry_key: String,
    /// In-game timestamp in seconds
    pub game_time_secs: f64,
    /// Wall clock time (ISO 8601)
    pub captured_at: String,
    /// Type-specific payload
    pub data: serde_json::Value,
    /// Only for moments: whether recording was triggered
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_fired: Option<bool>,
}

impl TimelineEntry {
    /// Create an event entry.
    pub fn event(
        event_type: impl Into<String>,
        game_time_secs: f64,
        captured_at: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            entry_type: "event".to_string(),
            entry_key: event_type.into(),
            game_time_secs,
            captured_at: captured_at.into(),
            data,
            trigger_fired: None,
        }
    }

    /// Create a statistic entry (delta).
    pub fn statistic(
        game_time_secs: f64,
        captured_at: impl Into<String>,
        changed_fields: serde_json::Value,
    ) -> Self {
        Self {
            entry_type: "statistic".to_string(),
            entry_key: "stats".to_string(),
            game_time_secs,
            captured_at: captured_at.into(),
            data: changed_fields,
            trigger_fired: None,
        }
    }

    /// Create a moment entry.
    pub fn moment(
        moment_id: impl Into<String>,
        game_time_secs: f64,
        captured_at: impl Into<String>,
        data: serde_json::Value,
        trigger_fired: bool,
    ) -> Self {
        Self {
            entry_type: "moment".to_string(),
            entry_key: moment_id.into(),
            game_time_secs,
            captured_at: captured_at.into(),
            data,
            trigger_fired: Some(trigger_fired),
        }
    }
}

/// Daemon → Gamepack: Request match timeline data.
///
/// Used for recovery when a gamepack needs to reconstruct match state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMatchTimelineRequest {
    /// Subpack index
    pub subpack: u8,
    /// Game's native match ID
    pub external_match_id: String,
    /// Filter by entry types (None = all types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_types: Option<Vec<String>>,
    /// Max entries to return (latest N)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Daemon → Gamepack: Response with match timeline data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMatchTimelineResponse {
    /// Whether the match was found
    pub found: bool,
    /// Timeline entries (empty if not found)
    pub entries: Vec<TimelineEntry>,
}
