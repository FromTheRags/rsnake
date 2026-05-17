//! This module contains the core components of the game logic, it includes the following submodules:
//!
//! - [`fruit`]: Represents the fruit element, including its position, score, and image.
//! - [`map`]: Manages the game logic map, layout, and boundaries.
//! - [`snake_body`]: Tracks and manages the snake's body and its movement.
pub mod fruit;

pub mod map;

pub mod snake_body;

/* Just to have some fun with moving letter
pub fn left(&mut self) {
        let mut current: u16 = self.body[0].x;
        self.body[0].x -= self.CASE_SIZE;
        let mut previous: u16 = current;
        for i in 1..self.body.len() {
            current = self.body[i].x;
            self.body[i].x = previous;
            previous = current;
        }
    }
    pub fn right(&mut self) {
        let mut current: u16 = self.body[0].x;
        self.body[0].x += self.CASE_SIZE;
        let mut previous: u16 = current;
        for i in 1..self.body.len() {
            current = self.body[i].x;
            self.body[i].x = previous;
            previous = current;
        }
    }
    pub fn up(&mut self) {
        let mut current: u16 = self.body[0].y;
        self.body[0].y -= self.CASE_SIZE / 2;
        let mut previous: u16 = current;
        for i in 1..self.body.len() {
            current = self.body[i].y;
            self.body[i].y = previous;
            previous = current;
        }
    }
    pub fn down(&mut self) {
        let mut current: u16 = self.body[0].y;
        self.body[0].y += self.CASE_SIZE / 2;
        let mut previous: u16 = current;
        for i in 1..self.body.len() {
            current = self.body[i].y;
            self.body[i].y = previous;
            previous = current;
        }
    }
 */
