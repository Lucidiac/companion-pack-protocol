//! Main loop runner for gamepacks.

use std::io::{BufRead, Write};
use std::sync::Mutex;

use crate::commands::GamepackCommand;
use crate::handler::GamepackHandler;
use crate::responses::GamepackResponse;
use crate::types::{GameEvent, InitResponse, MatchDataMessage, Moment};
use crate::version::PROTOCOL_VERSION;
use std::collections::HashMap;

/// Global stdout lock for thread-safe message emission.
/// This is used by `emit_match_data` to send unsolicited messages.
static STDOUT_LOCK: Mutex<()> = Mutex::new(());

/// Emit a match data message to the daemon (unsolicited).
///
/// This function can be called from any thread to send match data updates
/// to the daemon. Messages are thread-safe and will be properly interleaved
/// with command responses.
///
/// For convenience, use the typed helpers:
/// - [`emit_statistics`] for WriteStatistics
/// - [`emit_game_events`] for WriteGameEvents
/// - [`emit_moments`] for WriteMoments
///
/// # Example
///
/// ```rust,ignore
/// use companion_pack_protocol::{emit_match_data, MatchDataMessage, SummarySource};
/// use std::collections::HashMap;
///
/// // Mark match as complete
/// emit_match_data(MatchDataMessage::set_complete(
///     0,
///     "match123",
///     SummarySource::Api,
/// ));
/// ```
pub fn emit_match_data(message: MatchDataMessage) {
    let response = GamepackResponse::WriteMatchData { message };

    if let Ok(json) = serde_json::to_string(&response) {
        let _lock = STDOUT_LOCK.lock();
        let mut stdout = std::io::stdout();
        let _ = writeln!(stdout, "{}", json);
        let _ = stdout.flush();
    }
}

/// Emit statistics to the daemon.
///
/// Statistics are polled game state (KDA, CS, gold, etc.) that get:
/// 1. Stored to timeline with delta compression
/// 2. UPSERTed to the summary table
///
/// Call this periodically during gameplay when stats change.
///
/// # Example
///
/// ```rust,ignore
/// use companion_pack_protocol::emit_statistics;
/// use std::collections::HashMap;
/// use serde_json::json;
///
/// let mut stats = HashMap::new();
/// stats.insert("kills".to_string(), json!(5));
/// stats.insert("deaths".to_string(), json!(2));
/// stats.insert("cs".to_string(), json!(150));
///
/// emit_statistics(0, "match123", 1234.5, stats);
/// ```
pub fn emit_statistics(
    subpack: u8,
    external_match_id: impl Into<String>,
    game_time_secs: f64,
    stats: HashMap<String, serde_json::Value>,
) {
    emit_match_data(MatchDataMessage::write_statistics(
        subpack,
        external_match_id,
        game_time_secs,
        stats,
    ));
}

/// Emit game events to the daemon.
///
/// Events are discrete occurrences (kills, objectives, etc.) that get
/// stored to the timeline.
///
/// # Example
///
/// ```rust,ignore
/// use companion_pack_protocol::{emit_game_events, GameEvent};
/// use serde_json::json;
///
/// let events = vec![
///     GameEvent::new("ChampionKill", 120.0, json!({"killer": "Player1", "victim": "Enemy1"})),
///     GameEvent::new("DragonKill", 125.0, json!({"team": "blue", "dragon": "infernal"})),
/// ];
///
/// emit_game_events(0, "match123", events);
/// ```
pub fn emit_game_events(
    subpack: u8,
    external_match_id: impl Into<String>,
    events: Vec<GameEvent>,
) {
    emit_match_data(MatchDataMessage::write_game_events(
        subpack,
        external_match_id,
        events,
    ));
}

/// Emit moments to the daemon.
///
/// Moments are recordable things that might trigger a clip. The daemon will:
/// 1. Store to timeline with entry_type='moment'
/// 2. Check trigger configuration
/// 3. Fire recording if trigger is enabled
///
/// # Example
///
/// ```rust,ignore
/// use companion_pack_protocol::{emit_moments, Moment};
/// use serde_json::json;
///
/// let moments = vec![
///     Moment::new("pentakill", 1500.0, json!({"kills": 5, "time_span_secs": 10.0})),
/// ];
///
/// emit_moments(0, "match123", moments);
/// ```
pub fn emit_moments(
    subpack: u8,
    external_match_id: impl Into<String>,
    moments: Vec<Moment>,
) {
    emit_match_data(MatchDataMessage::write_moments(
        subpack,
        external_match_id,
        moments,
    ));
}

/// Run the gamepack main loop with the provided handler.
///
/// This function handles all stdin/stdout communication with the main daemon.
/// It reads NDJSON commands from stdin, dispatches them to the handler, and
/// writes NDJSON responses to stdout.
///
/// # Example
///
/// ```rust,ignore
/// use companion_pack_protocol::{run_gamepack, GamepackHandler};
///
/// struct MyGameIntegration { /* ... */ }
/// impl GamepackHandler for MyGameIntegration { /* ... */ }
///
/// fn main() {
///     let handler = MyGameIntegration::new();
///     run_gamepack(handler);
/// }
/// ```
///
/// # Protocol
///
/// The function will continue reading commands until:
/// - A `shutdown` command is received
/// - stdin is closed
/// - An unrecoverable error occurs
pub fn run_gamepack<H: GamepackHandler>(mut handler: H) {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) if !l.trim().is_empty() => l,
            Ok(_) => continue, // Skip empty lines
            Err(_) => break,   // stdin closed
        };

        let response = match serde_json::from_str::<GamepackCommand>(&line) {
            Ok(cmd) => dispatch_command(&mut handler, cmd),
            Err(e) => GamepackResponse::error("", format!("Parse error: {}", e)),
        };

        if let Ok(json) = serde_json::to_string(&response) {
            let _ = writeln!(stdout, "{}", json);
            let _ = stdout.flush();
        }

        // Exit after shutdown
        if matches!(response, GamepackResponse::ShutdownComplete { .. }) {
            break;
        }
    }
}

/// Dispatch a command to the appropriate handler method.
fn dispatch_command<H: GamepackHandler>(handler: &mut H, cmd: GamepackCommand) -> GamepackResponse {
    let request_id = cmd.request_id().to_string();

    match cmd {
        GamepackCommand::Init { .. } => match handler.init() {
            Ok(InitResponse {
                game_id,
                slug,
                protocol_version,
            }) => GamepackResponse::Initialized {
                request_id,
                game_id,
                slug,
                // Use the handler's version or fall back to crate version
                protocol_version: if protocol_version > 0 {
                    protocol_version
                } else {
                    PROTOCOL_VERSION
                },
            },
            Err(e) => GamepackResponse::Error {
                request_id,
                message: e.message,
                code: e.code,
            },
        },

        GamepackCommand::DetectRunning { .. } => GamepackResponse::RunningStatus {
            request_id,
            running: handler.detect_running(),
        },

        GamepackCommand::GetStatus { .. } => {
            let status = handler.get_status();
            GamepackResponse::GameStatus {
                request_id,
                connected: status.connected,
                connection_status: status.connection_status,
                game_phase: status.game_phase,
                is_in_game: status.is_in_game,
            }
        }

        GamepackCommand::PollEvents { .. } => {
            let events = handler.poll_events();
            GamepackResponse::Events { request_id, events }
        }

        GamepackCommand::GetLiveData { .. } => {
            let data = handler.get_live_data();
            GamepackResponse::LiveData { request_id, data }
        }

        GamepackCommand::SessionStart { .. } => {
            let context = handler.on_session_start();
            GamepackResponse::SessionStarted { request_id, context }
        }

        GamepackCommand::SessionEnd { context, .. } => {
            let match_data = handler.on_session_end(context);
            GamepackResponse::SessionEnded {
                request_id,
                match_data: match_data.map(|m| serde_json::to_value(m).unwrap_or_default()),
            }
        }

        GamepackCommand::Shutdown { .. } => {
            handler.shutdown();
            GamepackResponse::ShutdownComplete { request_id }
        }

        GamepackCommand::ResolveEventIcon { event_key, .. } => {
            let icon_url = handler.resolve_event_icon(&event_key);
            GamepackResponse::EventIconResolved {
                request_id,
                event_key,
                icon_url,
            }
        }

        GamepackCommand::IsMatchInProgress {
            subpack,
            external_match_id,
            ..
        } => {
            let response = handler.is_match_in_progress(subpack, &external_match_id);
            GamepackResponse::MatchInProgressStatus {
                request_id,
                still_playing: response.still_playing,
                set_complete: response.set_complete,
            }
        }

        GamepackCommand::GetMatchTimeline { .. } => {
            // This command is typically sent FROM the daemon TO provide timeline data,
            // but it can also be used for the gamepack to request its own data back.
            // Default implementation returns empty - daemon handles this.
            GamepackResponse::MatchTimeline {
                request_id,
                found: false,
                entries: vec![],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::GamepackResult;
    use crate::types::{GameEvent, GameStatus, MatchData};

    struct TestHandler {
        initialized: bool,
    }

    impl GamepackHandler for TestHandler {
        fn init(&mut self) -> GamepackResult<InitResponse> {
            self.initialized = true;
            Ok(InitResponse {
                game_id: 99,
                slug: "test".to_string(),
                protocol_version: 1,
            })
        }

        fn detect_running(&self) -> bool {
            true
        }

        fn get_status(&self) -> GameStatus {
            GameStatus::connected("Test connected")
        }

        fn poll_events(&mut self) -> Vec<GameEvent> {
            vec![]
        }

        fn get_live_data(&self) -> Option<serde_json::Value> {
            Some(serde_json::json!({"test": true}))
        }

        fn on_session_start(&mut self) -> Option<serde_json::Value> {
            Some(serde_json::json!({"started": true}))
        }

        fn on_session_end(&mut self, _context: serde_json::Value) -> Option<MatchData> {
            Some(MatchData::new("test", 99, "win", serde_json::json!({})))
        }

        fn shutdown(&mut self) {}
    }

    #[test]
    fn test_dispatch_init() {
        let mut handler = TestHandler { initialized: false };
        let response = dispatch_command(
            &mut handler,
            GamepackCommand::Init {
                request_id: "test_1".to_string(),
            },
        );

        assert!(handler.initialized);
        match response {
            GamepackResponse::Initialized {
                request_id,
                game_id,
                slug,
                ..
            } => {
                assert_eq!(request_id, "test_1");
                assert_eq!(game_id, 99);
                assert_eq!(slug, "test");
            }
            _ => panic!("Expected Initialized response"),
        }
    }

    #[test]
    fn test_dispatch_get_status() {
        let mut handler = TestHandler { initialized: false };
        let response = dispatch_command(
            &mut handler,
            GamepackCommand::GetStatus {
                request_id: "test_2".to_string(),
            },
        );

        match response {
            GamepackResponse::GameStatus {
                request_id,
                connected,
                connection_status,
                ..
            } => {
                assert_eq!(request_id, "test_2");
                assert!(connected);
                assert_eq!(connection_status, "Test connected");
            }
            _ => panic!("Expected GameStatus response"),
        }
    }
}
