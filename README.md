# companion-pack-protocol

Protocol types and helpers for building League Companion gamepacks.

## Overview

This crate provides the core protocol types and a runtime helper for building
gamepack daemons that communicate with the main League Companion application.

## Quick Start

```rust
use companion_pack_protocol::{run_gamepack, GamepackHandler, GamepackResult};
use companion_pack_protocol::{InitResponse, GameStatus, GameEvent, MatchData};

struct MyGameIntegration {
    // Your game-specific state
}

impl GamepackHandler for MyGameIntegration {
    fn init(&mut self) -> GamepackResult<InitResponse> {
        Ok(InitResponse {
            game_id: 99,
            slug: "my-game".to_string(),
            protocol_version: 1,
        })
    }

    fn detect_running(&self) -> bool {
        // Check if the game process is running
        false
    }

    fn get_status(&self) -> GameStatus {
        GameStatus {
            connected: false,
            connection_status: "Not connected".to_string(),
            game_phase: None,
            is_in_game: false,
        }
    }

    fn poll_events(&mut self) -> Vec<GameEvent> {
        // Return any new game events
        vec![]
    }

    fn get_live_data(&self) -> Option<serde_json::Value> {
        // Return live match data if in-game
        None
    }

    fn on_session_start(&mut self) -> Option<serde_json::Value> {
        // Called when a game session starts
        None
    }

    fn on_session_end(&mut self, _context: serde_json::Value) -> Option<MatchData> {
        // Called when a game session ends, return match data
        None
    }

    fn shutdown(&mut self) {
        // Cleanup on shutdown
    }
}

fn main() {
    let handler = MyGameIntegration { };
    run_gamepack(handler);
}
```

## Protocol

Communication happens over stdin/stdout using newline-delimited JSON (NDJSON).

### Commands (daemon -> gamepack)

- `init` - Initialize the integration
- `detect_running` - Check if game is running
- `get_status` - Get connection status
- `poll_events` - Poll for new game events
- `get_live_data` - Get live match data
- `session_start` - Game session started
- `session_end` - Game session ended
- `shutdown` - Graceful shutdown

### Responses (gamepack -> daemon)

Each command has a corresponding response type. All responses include a `request_id`
field for correlation.

## License

MIT
