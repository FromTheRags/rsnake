//! # Fruits Manager
//!
//! This module defines the `fruits_manager` struct,
//! which is responsible for handling fruit objects within a game.
//! It includes methods for spawning, replacing, and interacting with fruits.
//!
//! # Example
//! ```rust
//! use rsnaker::game_logic::fruits_manager::FruitsManager;
//! use rsnaker::graphics::graphic_block::Position;
//! use rsnaker::graphics::sprites::map::Map;
//! use ratatui::layout::Rect;
//! use std::sync::{Arc, RwLock};
//! use std::time::Duration;
//!
//! let case_size = 2;
//! let map_size = Rect::new(0, 0, 200, 10);
//! let map = Arc::new(RwLock::new(Map::new(case_size, map_size)));
//! let number_of_fruits_to_manage = 10;
//! let mut manager = FruitsManager::new(
//!     number_of_fruits_to_manage,
//!     map.clone(),
//!     true,
//!     5f32,
//! );
//!
//! // Simulate eating fruits
//! let position = Position { x: 10, y: 20 };
//! if let Some(eaten) = manager.eat_some_fruits(&position) {
//!     manager.replace_fruits(&eaten);
//! }
//! ```

use crate::graphics::graphic_block::Position;
use crate::graphics::sprites::fruit::{Fruit, FRUITS_SCORES_PROBABILITIES};
use crate::graphics::sprites::map::Map;
use rand::{rng, RngExt};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::widgets::WidgetRef;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Manages fruit objects within the game logic.
/// Map outlive fruits, so 'b lifetime >= 'a (fruits) lifetime
pub struct FruitsManager<'a, 'b: 'a> {
    fruits: Vec<Fruit<'a>>,      // List of fruits currently in the game_logic
    carte: Arc<RwLock<Map<'b>>>, // Reference to the game_logic map
    fruit_timer_enabled: bool,
    fruit_base_lifetime: f32,
}

impl<'a, 'b> FruitsManager<'a, 'b> {
    /// Creates a new `FruitsManager` with a given number of fruits.
    /// # Panics
    /// if guard cannot be got for Map (whenever a previous panic poisoned guard)
    /// # Example
    /// ```
    /// use std::sync::{Arc, RwLock};
    /// use std::time::Duration;
    /// use ratatui::layout::Rect;
    /// use rsnaker::game_logic::fruits_manager::FruitsManager;
    /// use rsnaker::graphics::sprites::map::Map;
    /// let map = Arc::new(RwLock::new(Map::new(2, Rect::new(0,0, 160,10))));
    /// let manager = FruitsManager::new(3, map, true, 5f32);
    /// ```
    #[must_use]
    pub fn new(
        nb: u16,
        carte: Arc<RwLock<Map<'b>>>,
        fruit_timer_enabled: bool,
        fruit_base_lifetime: f32,
    ) -> Self {
        let mut fm = Self {
            fruits: Vec::with_capacity(nb as usize),
            carte,
            fruit_timer_enabled,
            fruit_base_lifetime,
        };
        fm.init(nb);
        fm
    }

    /// Spawns a fruit at a random position on the map.
    fn spawn_random(carte: &Map, fruit_timer_enabled: bool, fruit_base_lifetime: f32) -> Fruit<'a> {
        let position = Self::generate_position_rounded_by_cs(carte);
        let random_value: u16 = rng().random_range(1..=100);
        let mut cumulative_probability = 0;
        for &(image, score, probability, size_effect) in FRUITS_SCORES_PROBABILITIES {
            cumulative_probability += probability;
            if random_value <= cumulative_probability {
                return Fruit::new(
                    score,
                    size_effect,
                    position,
                    image,
                    fruit_base_lifetime,
                    fruit_timer_enabled,
                );
            }
        }
        // Default fallback fruit
        let (image, score, _, size_effect) = FRUITS_SCORES_PROBABILITIES[0];
        Fruit::new(
            score,
            size_effect,
            position,
            image,
            fruit_base_lifetime,
            fruit_timer_enabled,
        )
    }

    /// Replaces eaten fruits with new random ones and ensures balance.
    /// # Panics
    /// if guard cannot be got for Map (whenever a previous panic poisoned guard)
    pub fn replace_fruits(&mut self, fruits_to_remove: &[Fruit<'a>]) {
        if fruits_to_remove.is_empty() {
            return;
        }
        let initial_count = self.fruits.len();
        self.fruits
            .retain(|fruit| !fruits_to_remove.contains(fruit));

        self.spawn_missing_fruits(initial_count - self.fruits.len());
    }

    /// Removes expired fruits and replaces them with fresh ones.
    pub fn update(&mut self) {
        if !self.fruit_timer_enabled {
            return;
        }
        let now = Instant::now();
        let initial_count = self.fruits.len();
        // Filter
        self.fruits.retain(|fruit| !fruit.is_expired(now));

        self.spawn_missing_fruits(initial_count - self.fruits.len());
    }

    /// Refills the fruit vector to its expected capacity by spawning new fruits.
    fn spawn_missing_fruits(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        // Lock the map once for all spawns
        let carte_guard = self.carte.read().unwrap();
        // Optimize memory allocation to prevent multiple costly reallocations
        self.fruits.reserve(count);

        for _ in 0..count {
            self.fruits.push(Self::spawn_random(
                &carte_guard,
                self.fruit_timer_enabled,
                self.fruit_base_lifetime,
            ));
        }
    }

    /// Pauses or resumes all fruit timers.
    pub fn set_timer_paused(&mut self, paused: bool) {
        if !self.fruit_timer_enabled {
            return;
        }

        let now = Instant::now();
        for fruit in &mut self.fruits {
            fruit.set_timer_paused(paused, now);
        }
    }

    /// Returns a list of fruits at the given position, copying them to avoid lock contention.
    #[must_use]
    pub fn eat_some_fruits(&self, position: &Position) -> Option<Vec<Fruit<'a>>> {
        let now = Instant::now();
        let eaten: Vec<Fruit<'a>> = self
            .fruits
            .iter()
            .filter(|x| x.is_at_position(position) && !x.is_expired(now))
            .cloned()
            .collect();
        if eaten.is_empty() { None } else { Some(eaten) }
    }

    /// Generates a random valid position for spawning fruits.
    fn generate_position_rounded_by_cs(carte: &Map) -> Position {
        let mut rng = rng();
        let cs = carte.get_case_size();
        let csy = 1;
        let width = carte.area().width;
        let height = carte.area().height;
        //saturating cs to be sure to have the size for the counter (at right position)
        let mut max_index_x = (width / cs).saturating_sub(cs);
        let mut max_index_y = (height / csy).saturating_sub(csy);
        // Ensure a valid range for generation
        if max_index_x <= 1 {
            max_index_x = 2;
        }
        if max_index_y <= 1 {
            max_index_y = 2;
        }
        Position {
            x: rng.random_range(1..max_index_x) * cs,
            y: rng.random_range(1..max_index_y) * csy,
        }
    }

    pub(crate) fn reset_to_terminal_size(&mut self) {
        //change the position of all fruits to avoid no eatable/unreachable fruits
        for f in &mut self.fruits {
            f.set_position(Self::generate_position_rounded_by_cs(
                &self.carte.read().unwrap(),
            ));
        }
    }

    pub(crate) fn reset(&mut self) {
        let len = u16::try_from(self.fruits.len()).unwrap();
        self.fruits.clear();
        self.init(len);
    }

    /// Returns all current fruits.
    #[must_use]
    pub fn get_fruits(&self) -> &Vec<Fruit<'a>> {
        &self.fruits
    }

    fn init(&mut self, nb: u16) {
        for _ in 0..nb {
            self.fruits.push(Self::spawn_random(
                &self.carte.read().unwrap(),
                self.fruit_timer_enabled,
                self.fruit_base_lifetime,
            ));
        }
    }
}

/// Implements `WidgetRef` for rendering fruits on the screen.
impl<'a> WidgetRef for FruitsManager<'a, 'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        for fruit in &self.fruits {
            fruit.render_ref(area, buf);
        }
    }
}

/// Implements `Widget` for compatibility with older versions.
impl<'a> Widget for FruitsManager<'a, 'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

impl<'a> Widget for &FruitsManager<'a, 'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

/// Test part:
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Mock definitions
    /// Not really need it there,but for example, on how to share resources for test
    fn mock_map() -> Arc<RwLock<Map<'static>>> {
        Arc::new(RwLock::new(Map::new(2, Rect::new(0, 0, 160, 12))))
    }

    fn dummy_position() -> Position {
        Position { x: 10, y: 10 }
    }

    #[test]
    fn test_new_creates_correct_number_of_fruits() {
        let map = mock_map();
        let manager = FruitsManager::new(5, map, true, 5f32);
        assert_eq!(manager.fruits.len(), 5);
    }

    #[test]
    fn test_replace_fruits_removes_and_adds_new() {
        let map = mock_map();
        let mut manager = FruitsManager::new(3, Arc::clone(&map), true, 5f32);
        let fruits_to_remove = vec![manager.fruits[0].clone()];
        manager.replace_fruits(&fruits_to_remove);
        assert_eq!(manager.fruits.len(), 3);
        assert!(!manager.fruits.contains(&fruits_to_remove[0]));
    }

    #[test]
    fn test_eat_some_fruits_returns_correct_fruit() {
        let map = mock_map();
        let mut manager = FruitsManager::new(3, Arc::clone(&map), true, 5f32);
        let fruit = Fruit::new(10, 1, dummy_position(), "🍎", 5f32, true);
        manager.fruits[0] = fruit.clone();
        let result = manager.eat_some_fruits(&dummy_position());
        assert!(result.is_some());
        assert!(result.unwrap().contains(&fruit));
    }

    #[test]
    fn test_eat_some_fruits_returns_none_if_no_fruit() {
        let map = mock_map();
        let manager = FruitsManager::new(3, Arc::clone(&map), true, 5f32);
        let result = manager.eat_some_fruits(&Position { x: 999, y: 999 });
        assert!(result.is_none());
    }

    #[test]
    fn test_update_replaces_expired_fruit() {
        let map = mock_map();
        let mut manager = FruitsManager::new(1, Arc::clone(&map), true, 5f32);
        manager.fruits[0] = Fruit::new(10, 1, dummy_position(), "🍎", 0f32, true);

        manager.update();

        assert_eq!(manager.fruits.len(), 1);
        assert!(!manager.fruits[0].is_expired(Instant::now()));
    }
}
