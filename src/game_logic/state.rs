//! # Game State Management
//!
//! This module defines a `GameState` struct that manages game-related flags
//! such as life, score, and game status.
//! It provides methods for initializing and resetting the game state.
//!
//! # Example
//! ```rust
//! use rsnaker::game_logic::state::{GameState, GameStatus};
//!
//! let mut state = GameState::new(3); // Initialize game_logic with 3 lives
//! assert_eq!(state.life, 3);
//! assert_eq!(state.score, 0);
//! assert_eq!(state.status, GameStatus::Playing);
//!
//! state.reset(); // Reset game_logic state
//! assert_eq!(state.life, 3);
//! assert_eq!(state.score, 0);
//! ```
//!

/// Represents the possible states of the game.
#[derive(Debug, Clone, PartialEq)]
pub enum GameStatus {
    Paused,     // The game is currently paused
    GameOver,   // The game has ended
    ByeBye,     // The game has exited
    Playing,    // The game is in progress
    Restarting, // The game is restarting
    Menu,       // The game is going or displaying the Main Menu
}

/// Manages the game state, including life count, score, and current status.
#[derive(Clone, Debug, PartialEq)]
pub struct GameState {
    pub life: u16,          // Current player life count
    pub score: i32,         // Current player score
    pub status: GameStatus, // Current game status
    life_ini: u16,          // Initial life count (used for reset)
}

impl GameState {
    /// Creates a new `GameState` with the specified initial life count.
    ///
    /// # Arguments
    /// * `life` - The initial number of lives the player starts with.
    ///
    /// # Returns
    /// A new instance of `GameState` with default values.
    #[must_use]
    pub fn new(life: u16) -> Self {
        Self {
            life,
            score: 0,
            status: GameStatus::Playing,
            life_ini: life,
        }
    }

    /// Resets the game state to its initial values.
    pub fn reset(&mut self) {
        self.score = 0;
        self.status = GameStatus::Playing;
        self.life = self.life_ini;
    }
}
