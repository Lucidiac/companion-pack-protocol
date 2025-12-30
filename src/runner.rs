//! Main loop runner for gamepacks.

use std::io::{BufRead, Write};

use crate::commands::GamepackCommand;
use crate::handler::GamepackHandler;
use crate::responses::GamepackResponse;
use crate::types::InitResponse;
use crate::version::PROTOCOL_VERSION;

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
