//! # Game logic Modules
//!
//! This module organizes various components of the game logic, including:
//!
//! - `playing_logic`: handles high level game logic during gameplay.
//! - `playing_thread_manager`: manages the main game thread and timing.
//! - `state`: maintains the game's state, including score and status.
//! - `fruits_manager`: handles fruit-related logic and management.
//! - `game_options`: provides configuration options for the game with CLI integration.
//!
//! These modules collectively contribute to the core mechanics of the game logic.
//!

pub mod fruits_manager;
pub mod game_options;
pub mod high_score;
pub mod playing_logic;
pub mod playing_thread_manager;
pub mod state;
