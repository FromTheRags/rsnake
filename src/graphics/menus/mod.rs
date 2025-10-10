//! This module provides various menu and UI components for the game interface.
//!
//! ## Modules:
//!
//! - `greeting_menu`: Displays the initial welcome screen and main menu.
//! - `layout_utils`: Utility functions for UI layout calculations and rendering.
//! - `multiple_choice_menu`: Implements multi-select menu with a save button.
//! - `parameters_menu`: Handles game parameter configuration.
//! - `selectable_item`: Provides a generic selectable list widget.
//! - `status`: Contains UI elements for different game states (pause, game over, etc.).

pub mod customized_retro_parameter_with_cli;
/// A as generic as possible table for parameter implementation, data are loaded in `parameter_helper`,
/// as long as (control, apply : wip (take a &mut self ) and footer
pub mod generic_retro_parameter_logic;
pub mod generic_retro_parameter_style;
pub mod greeting;
pub mod status;
pub mod utils_layout;
