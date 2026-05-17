//! # Game graphical Modules
//! This module contains the core components of the game logic, it includes the following submodules:
//!
//! - [`graphic_block`]: Handles the graphical rendering of game logic blocks in the terminal.
//! - [`sprites`]: Provides the game sprites, as snake, map, and so on
//! - [`menus`]: Provides utility functions to help with common texts and graphical tasks

pub mod graphic_block;
pub mod sprites;

pub mod menus;
pub mod playing_render;
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
