//! # companion-pack-protocol
//!
//! Protocol types and helpers for building League Companion gamepacks.
//!
//! This crate provides everything needed to implement a gamepack daemon that
//! communicates with the main League Companion application via NDJSON over
//! stdin/stdout.
//!
//! ## Quick Start
//!
//! 1. Implement the [`GamepackHandler`] trait for your game integration
//! 2. Call [`run_gamepack`] with your handler to start the main loop
//!
//! ```rust,ignore
//! use companion_pack_protocol::{run_gamepack, GamepackHandler, GamepackResult};
//! use companion_pack_protocol::{InitResponse, GameStatus, GameEvent, MatchData};
//!
//! struct MyGameIntegration { /* ... */ }
//!
//! impl GamepackHandler for MyGameIntegration {
//!     fn init(&mut self) -> GamepackResult<InitResponse> {
//!         Ok(InitResponse {
//!             game_id: 99,
//!             slug: "my-game".to_string(),
//!             protocol_version: 1,
//!         })
//!     }
//!
//!     fn detect_running(&self) -> bool { false }
//!     fn get_status(&self) -> GameStatus { GameStatus::disconnected() }
//!     fn poll_events(&mut self) -> Vec<GameEvent> { vec![] }
//!     fn get_live_data(&self) -> Option<serde_json::Value> { None }
//!     fn on_session_start(&mut self) -> Option<serde_json::Value> { None }
//!     fn on_session_end(&mut self, _: serde_json::Value) -> Option<MatchData> { None }
//!     fn shutdown(&mut self) {}
//! }
//!
//! fn main() {
//!     run_gamepack(MyGameIntegration { /* ... */ });
//! }
//! ```
//!
//! ## Protocol
//!
//! Communication uses newline-delimited JSON (NDJSON):
//!
//! - Commands flow from daemon to gamepack via stdin
//! - Responses flow from gamepack to daemon via stdout
//! - Each command has a `request_id` for correlation
//!
//! See [`GamepackCommand`] and [`GamepackResponse`] for the full protocol.

pub mod commands;
pub mod handler;
pub mod responses;
pub mod runner;
pub mod types;
pub mod version;

// Re-export main types at crate root for convenience
pub use commands::GamepackCommand;
pub use handler::{GamepackError, GamepackHandler, GamepackResult};
pub use responses::GamepackResponse;
pub use runner::run_gamepack;
pub use types::{GameEvent, GameStatus, InitResponse, MatchData};
pub use version::PROTOCOL_VERSION;
