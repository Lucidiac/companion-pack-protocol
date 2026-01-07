//! Shared types used in the gamepack protocol.
//!
//! NOTE: All types here are GAME-AGNOSTIC. No League/TFT/etc specifics.
//! Each gamepack defines its own subpacks and column schemas in config.json.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{Display, EnumString};

// ============================================================================
// TYPE-SAFE ENUMS
// ============================================================================

/// Type of entry in the match timeline.
///
/// Used for filtering and ensuring type safety when storing/retrieving timeline data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
pub enum EntryType {
    /// Discrete game events (kills, objectives, etc.)
    Event,
    /// Polled statistics (KDA, CS, gold, etc.)
    Statistic,
    /// Recordable moments that may trigger clips
    Moment,
}

/// Source of match summary data.
///
/// Indicates whether the final stats came from an official API or were
/// reconstructed from live data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
pub enum SummarySource {
    /// Stats from official game API (most accurate)
    Api,
    /// Stats reconstructed from live data (fallback when API unavailable)
    LiveFallback,
}

// ============================================================================
// GAME EVENTS
// ============================================================================

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
// MOMENTS
// ============================================================================

/// A recordable moment that might trigger a clip.
///
/// Moments are distinct from events - they represent things worth recording,
/// not just things that happened. The daemon checks trigger configuration
/// to decide whether to actually record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Moment {
    /// Moment ID (must match a moment defined in config.json or will be auto-registered)
    pub moment_id: String,
    /// In-game timestamp in seconds
    pub game_time_secs: f64,
    /// Moment-specific data (context for the clip)
    pub data: serde_json::Value,
}

impl Moment {
    /// Create a new moment.
    pub fn new(moment_id: impl Into<String>, game_time_secs: f64, data: serde_json::Value) -> Self {
        Self {
            moment_id: moment_id.into(),
            game_time_secs,
            data,
        }
    }
}

// ============================================================================
// MATCH DATA MESSAGES (Subpack Model)
// ============================================================================

/// Gamepack → Daemon: Write match data.
///
/// These are the three unsolicited messages a gamepack can send to the daemon
/// during gameplay, plus SetComplete to mark matches finished.
///
/// **Data Flow:**
/// - `WriteStatistics` → Timeline (delta) + Summary (UPSERT)
/// - `WriteGameEvents` → Timeline (events)
/// - `WriteMoments` → Timeline (moments) + Trigger check
/// - `SetComplete` → Mark `is_in_progress=0`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MatchDataMessage {
    /// Write statistics to timeline (delta compressed) AND summary table (UPSERT).
    ///
    /// This is the primary way to emit polled game state during gameplay.
    /// The daemon will:
    /// 1. Create match row if it doesn't exist (lazy creation)
    /// 2. Store to timeline with delta compression (only changed fields)
    /// 3. UPSERT to summary table (`p{guid}_{subpack}_match_details`)
    WriteStatistics {
        /// Subpack index (0 = default, 1+ = additional subpacks)
        subpack: u8,
        /// Game's native match ID (used for deduplication and API lookups)
        external_match_id: String,
        /// When the match started (ISO 8601) - only needed on first write
        #[serde(skip_serializing_if = "Option::is_none")]
        played_at: Option<String>,
        /// In-game timestamp in seconds
        game_time_secs: f64,
        /// Stats to write (keys must match columns declared in subpack's schema)
        stats: HashMap<String, serde_json::Value>,
    },

    /// Write game events to timeline.
    ///
    /// Events are discrete occurrences (kills, objectives, etc.).
    /// Saved to `p{guid}_{subpack}_match_timeline` with entry_type='event'.
    WriteGameEvents {
        /// Subpack index (0 = default, 1+ = additional subpacks)
        subpack: u8,
        /// Game's native match ID
        external_match_id: String,
        /// Events to append
        events: Vec<GameEvent>,
    },

    /// Write moments to timeline and check triggers.
    ///
    /// Moments are recordable things that might trigger a clip.
    /// The daemon will:
    /// 1. Store to timeline with entry_type='moment'
    /// 2. Check trigger configuration for each moment
    /// 3. Fire recording if trigger is enabled
    WriteMoments {
        /// Subpack index (0 = default, 1+ = additional subpacks)
        subpack: u8,
        /// Game's native match ID
        external_match_id: String,
        /// Moments to process
        moments: Vec<Moment>,
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
        /// Where the final stats came from
        summary_source: SummarySource,
        /// Optional final stats to overwrite summary table
        #[serde(skip_serializing_if = "Option::is_none")]
        final_stats: Option<HashMap<String, serde_json::Value>>,
    },
}

impl MatchDataMessage {
    /// Create a WriteStatistics message.
    pub fn write_statistics(
        subpack: u8,
        external_match_id: impl Into<String>,
        game_time_secs: f64,
        stats: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self::WriteStatistics {
            subpack,
            external_match_id: external_match_id.into(),
            played_at: None,
            game_time_secs,
            stats,
        }
    }

    /// Create a WriteStatistics message with played_at timestamp.
    pub fn write_statistics_with_time(
        subpack: u8,
        external_match_id: impl Into<String>,
        played_at: impl Into<String>,
        game_time_secs: f64,
        stats: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self::WriteStatistics {
            subpack,
            external_match_id: external_match_id.into(),
            played_at: Some(played_at.into()),
            game_time_secs,
            stats,
        }
    }

    /// Create a WriteGameEvents message.
    pub fn write_game_events(
        subpack: u8,
        external_match_id: impl Into<String>,
        events: Vec<GameEvent>,
    ) -> Self {
        Self::WriteGameEvents {
            subpack,
            external_match_id: external_match_id.into(),
            events,
        }
    }

    /// Create a WriteMoments message.
    pub fn write_moments(
        subpack: u8,
        external_match_id: impl Into<String>,
        moments: Vec<Moment>,
    ) -> Self {
        Self::WriteMoments {
            subpack,
            external_match_id: external_match_id.into(),
            moments,
        }
    }

    /// Create a SetComplete message.
    pub fn set_complete(
        subpack: u8,
        external_match_id: impl Into<String>,
        summary_source: SummarySource,
    ) -> Self {
        Self::SetComplete {
            subpack,
            external_match_id: external_match_id.into(),
            summary_source,
            final_stats: None,
        }
    }

    /// Create a SetComplete message with final stats.
    pub fn set_complete_with_stats(
        subpack: u8,
        external_match_id: impl Into<String>,
        summary_source: SummarySource,
        final_stats: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self::SetComplete {
            subpack,
            external_match_id: external_match_id.into(),
            summary_source,
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
    /// Entry type (event, statistic, or moment)
    pub entry_type: EntryType,
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
            entry_type: EntryType::Event,
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
            entry_type: EntryType::Statistic,
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
            entry_type: EntryType::Moment,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::str::FromStr;

    // ========================================================================
    // EntryType Tests
    // ========================================================================

    #[test]
    fn entry_type_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&EntryType::Event).unwrap(),
            "\"event\""
        );
        assert_eq!(
            serde_json::to_string(&EntryType::Statistic).unwrap(),
            "\"statistic\""
        );
        assert_eq!(
            serde_json::to_string(&EntryType::Moment).unwrap(),
            "\"moment\""
        );
    }

    #[test]
    fn entry_type_deserializes_from_snake_case() {
        assert_eq!(
            serde_json::from_str::<EntryType>("\"event\"").unwrap(),
            EntryType::Event
        );
        assert_eq!(
            serde_json::from_str::<EntryType>("\"statistic\"").unwrap(),
            EntryType::Statistic
        );
        assert_eq!(
            serde_json::from_str::<EntryType>("\"moment\"").unwrap(),
            EntryType::Moment
        );
    }

    #[test]
    fn entry_type_display_is_snake_case() {
        assert_eq!(EntryType::Event.to_string(), "event");
        assert_eq!(EntryType::Statistic.to_string(), "statistic");
        assert_eq!(EntryType::Moment.to_string(), "moment");
    }

    #[test]
    fn entry_type_from_str_case_insensitive() {
        assert_eq!(EntryType::from_str("EVENT").unwrap(), EntryType::Event);
        assert_eq!(EntryType::from_str("event").unwrap(), EntryType::Event);
        assert_eq!(EntryType::from_str("Event").unwrap(), EntryType::Event);
    }

    #[test]
    fn entry_type_round_trips() {
        for entry_type in [EntryType::Event, EntryType::Statistic, EntryType::Moment] {
            let json = serde_json::to_string(&entry_type).unwrap();
            let back: EntryType = serde_json::from_str(&json).unwrap();
            assert_eq!(entry_type, back);
        }
    }

    // ========================================================================
    // SummarySource Tests
    // ========================================================================

    #[test]
    fn summary_source_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&SummarySource::Api).unwrap(),
            "\"api\""
        );
        assert_eq!(
            serde_json::to_string(&SummarySource::LiveFallback).unwrap(),
            "\"live_fallback\""
        );
    }

    #[test]
    fn summary_source_deserializes_from_snake_case() {
        assert_eq!(
            serde_json::from_str::<SummarySource>("\"api\"").unwrap(),
            SummarySource::Api
        );
        assert_eq!(
            serde_json::from_str::<SummarySource>("\"live_fallback\"").unwrap(),
            SummarySource::LiveFallback
        );
    }

    #[test]
    fn summary_source_display_is_snake_case() {
        assert_eq!(SummarySource::Api.to_string(), "api");
        assert_eq!(SummarySource::LiveFallback.to_string(), "live_fallback");
    }

    #[test]
    fn summary_source_round_trips() {
        for source in [SummarySource::Api, SummarySource::LiveFallback] {
            let json = serde_json::to_string(&source).unwrap();
            let back: SummarySource = serde_json::from_str(&json).unwrap();
            assert_eq!(source, back);
        }
    }

    // ========================================================================
    // GameEvent Tests
    // ========================================================================

    #[test]
    fn game_event_new_creates_with_defaults() {
        let event = GameEvent::new("ChampionKill", 100.5, json!({"killer": "Player1"}));

        assert_eq!(event.event_type, "ChampionKill");
        assert_eq!(event.timestamp_secs, 100.5);
        assert_eq!(event.data, json!({"killer": "Player1"}));
        assert_eq!(event.pre_capture_secs, None);
        assert_eq!(event.post_capture_secs, None);
    }

    #[test]
    fn game_event_with_capture_times() {
        let event = GameEvent::new("DragonKill", 500.0, json!({}))
            .with_pre_capture(15.0)
            .with_post_capture(10.0);

        assert_eq!(event.pre_capture_secs, Some(15.0));
        assert_eq!(event.post_capture_secs, Some(10.0));
    }

    #[test]
    fn game_event_serializes_correctly() {
        let event = GameEvent::new("ChampionKill", 100.5, json!({"killer": "Player1"}));
        let json = serde_json::to_string(&event).unwrap();

        assert!(json.contains("\"event_type\":\"ChampionKill\""));
        assert!(json.contains("\"timestamp_secs\":100.5"));
        assert!(!json.contains("pre_capture_secs")); // None should be skipped
    }

    #[test]
    fn game_event_round_trips() {
        let event = GameEvent::new("ChampionKill", 100.5, json!({"killer": "Player1"}))
            .with_pre_capture(10.0);
        let json = serde_json::to_string(&event).unwrap();
        let back: GameEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.event_type, back.event_type);
        assert!((event.timestamp_secs - back.timestamp_secs).abs() < 0.001);
        assert_eq!(event.pre_capture_secs, back.pre_capture_secs);
    }

    // ========================================================================
    // Moment Tests
    // ========================================================================

    #[test]
    fn moment_new_creates_correctly() {
        let moment = Moment::new("pentakill", 1500.0, json!({"kills": 5}));

        assert_eq!(moment.moment_id, "pentakill");
        assert_eq!(moment.game_time_secs, 1500.0);
        assert_eq!(moment.data, json!({"kills": 5}));
    }

    #[test]
    fn moment_round_trips() {
        let moment = Moment::new("death", 250.0, json!({"killer": "Enemy1"}));
        let json = serde_json::to_string(&moment).unwrap();
        let back: Moment = serde_json::from_str(&json).unwrap();

        assert_eq!(moment.moment_id, back.moment_id);
        assert!((moment.game_time_secs - back.game_time_secs).abs() < 0.001);
        assert_eq!(moment.data, back.data);
    }

    // ========================================================================
    // MatchDataMessage Tests
    // ========================================================================

    #[test]
    fn write_statistics_serializes_with_type_tag() {
        let msg = MatchDataMessage::write_statistics(
            0,
            "match123",
            100.0,
            [("kills".to_string(), json!(5))].into_iter().collect(),
        );
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"write_statistics\""));
        assert!(json.contains("\"subpack\":0"));
        assert!(json.contains("\"external_match_id\":\"match123\""));
        assert!(json.contains("\"game_time_secs\":100"));
    }

    #[test]
    fn write_game_events_serializes_with_type_tag() {
        let events = vec![GameEvent::new("ChampionKill", 100.0, json!({}))];
        let msg = MatchDataMessage::write_game_events(0, "match123", events);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"write_game_events\""));
        assert!(json.contains("\"events\""));
    }

    #[test]
    fn write_moments_serializes_with_type_tag() {
        let moments = vec![Moment::new("pentakill", 1500.0, json!({}))];
        let msg = MatchDataMessage::write_moments(0, "match123", moments);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"write_moments\""));
        assert!(json.contains("\"moments\""));
    }

    #[test]
    fn set_complete_serializes_with_type_tag() {
        let msg = MatchDataMessage::set_complete(0, "match123", SummarySource::Api);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"set_complete\""));
        assert!(json.contains("\"summary_source\":\"api\""));
    }

    #[test]
    fn match_data_message_round_trips_all_variants() {
        let messages: Vec<MatchDataMessage> = vec![
            MatchDataMessage::write_statistics(0, "m1", 100.0, HashMap::new()),
            MatchDataMessage::write_game_events(
                0,
                "m1",
                vec![GameEvent::new("Kill", 50.0, json!({}))],
            ),
            MatchDataMessage::write_moments(0, "m1", vec![Moment::new("death", 75.0, json!({}))]),
            MatchDataMessage::set_complete(0, "m1", SummarySource::Api),
            MatchDataMessage::set_complete_with_stats(
                0,
                "m1",
                SummarySource::LiveFallback,
                [("kills".to_string(), json!(10))].into_iter().collect(),
            ),
        ];

        for msg in messages {
            let json = serde_json::to_string(&msg).unwrap();
            let back: MatchDataMessage = serde_json::from_str(&json).unwrap();
            // Round-trip should produce equivalent JSON
            let json2 = serde_json::to_string(&back).unwrap();
            assert_eq!(json, json2);
        }
    }

    // ========================================================================
    // TimelineEntry Tests
    // ========================================================================

    #[test]
    fn timeline_entry_event_creates_correctly() {
        let entry = TimelineEntry::event(
            "ChampionKill",
            100.0,
            "2024-01-15T10:30:00Z",
            json!({"killer": "Player1"}),
        );

        assert_eq!(entry.entry_type, EntryType::Event);
        assert_eq!(entry.entry_key, "ChampionKill");
        assert_eq!(entry.trigger_fired, None);
    }

    #[test]
    fn timeline_entry_statistic_creates_correctly() {
        let entry = TimelineEntry::statistic(
            100.0,
            "2024-01-15T10:30:00Z",
            json!({"kills": 5}),
        );

        assert_eq!(entry.entry_type, EntryType::Statistic);
        assert_eq!(entry.entry_key, "stats");
        assert_eq!(entry.trigger_fired, None);
    }

    #[test]
    fn timeline_entry_moment_creates_correctly() {
        let entry = TimelineEntry::moment(
            "pentakill",
            1500.0,
            "2024-01-15T10:55:00Z",
            json!({"kills": 5}),
            true,
        );

        assert_eq!(entry.entry_type, EntryType::Moment);
        assert_eq!(entry.entry_key, "pentakill");
        assert_eq!(entry.trigger_fired, Some(true));
    }

    #[test]
    fn timeline_entry_round_trips() {
        let entry = TimelineEntry::event(
            "ChampionKill",
            100.0,
            "2024-01-15T10:30:00Z",
            json!({"killer": "Player1", "victim": "Enemy1"}),
        );
        let json = serde_json::to_string(&entry).unwrap();
        let back: TimelineEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.entry_type, back.entry_type);
        assert_eq!(entry.entry_key, back.entry_key);
        assert_eq!(entry.data, back.data);
    }

    // ========================================================================
    // IsMatchInProgressResponse Tests
    // ========================================================================

    #[test]
    fn is_match_in_progress_response_still_playing() {
        let response = IsMatchInProgressResponse::still_playing();

        assert!(response.still_playing);
        assert!(response.set_complete.is_none());
    }

    #[test]
    fn is_match_in_progress_response_ended() {
        let response = IsMatchInProgressResponse::ended();

        assert!(!response.still_playing);
        assert!(response.set_complete.is_none());
    }

    #[test]
    fn is_match_in_progress_response_ended_with_stats() {
        let set_complete = MatchDataMessage::set_complete(0, "match123", SummarySource::Api);
        let response = IsMatchInProgressResponse::ended_with_stats(set_complete);

        assert!(!response.still_playing);
        assert!(response.set_complete.is_some());
    }

    // ========================================================================
    // GameStatus Tests
    // ========================================================================

    #[test]
    fn game_status_disconnected() {
        let status = GameStatus::disconnected();

        assert!(!status.connected);
        assert_eq!(status.connection_status, "Not connected");
        assert!(status.game_phase.is_none());
        assert!(!status.is_in_game);
    }

    #[test]
    fn game_status_connected_with_phase() {
        let status = GameStatus::connected("Connected to client")
            .with_phase("InProgress")
            .in_game(true);

        assert!(status.connected);
        assert_eq!(status.connection_status, "Connected to client");
        assert_eq!(status.game_phase, Some("InProgress".to_string()));
        assert!(status.is_in_game);
    }

    // ========================================================================
    // MatchData Tests
    // ========================================================================

    #[test]
    fn match_data_new_creates_correctly() {
        let data = MatchData::new("league", 1, "win", json!({"kills": 10}));

        assert_eq!(data.game_slug, "league");
        assert_eq!(data.game_id, 1);
        assert_eq!(data.result, "win");
        assert_eq!(data.details, json!({"kills": 10}));
    }
}
