//! Slydot: The Sunrise Event – an original game written in Rust.
//!
//! Any similarity between this game and *Spybot: The Nightfall Incident* is completely
//! coincidental.

extern crate piston;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;

use std::collections::VecDeque;
use graphics::Context;
use opengl_graphics::{GlGraphics, OpenGL};
use sdl2_window::Sdl2Window;
use piston::window::WindowSettings;
use piston::event::*;

mod unit;
use unit::Unit;

mod grid;
use grid::Grid;

pub const CELL_SIZE: f64 = 30.0;
pub const CELL_PADDING: f64 = 5.0;
pub const CELL_OFFSET_X: f64 = 50.0;
pub const CELL_OFFSET_Y: f64 = 50.0;

pub struct Game {
    grid: Grid,
    player_units: VecDeque<Unit>,
    enemy_units: VecDeque<Unit>,
}

impl Game {
    pub fn sample() -> Game {
        let mut deque = VecDeque::new();
        deque.push_back(Unit::sample());
        Game {
            grid: Grid::sample(),
            player_units: deque,
            enemy_units: VecDeque::new(),
        }
    }

    pub fn each_player_unit<F>(&mut self, mut f: F) where F: FnMut(&mut Unit, &mut Game) {
        for _ in 0..self.player_units.len() {
            let mut unit = self.player_units.pop_front().unwrap();
            f(&mut unit, self);
            self.player_units.push_back(unit);
        }
    }
}

pub trait Draw {
    fn draw(&mut self, &Context, &mut GlGraphics);
}

fn main() {
    let opengl = OpenGL::_3_2;
    let window = Sdl2Window::new(
        opengl,
        WindowSettings::new("sunrise", [640, 480])
        .exit_on_esc(true)
    );

    let ref mut gl = GlGraphics::new(opengl);
    let mut game = Game::sample();
    for e in window.events() {
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, gl| {
                use graphics::*;
                clear([0.125, 0.75, 0.1875, 1.0], gl);
                game.grid.draw(&c, gl);
                for unit in &mut game.player_units {
                    unit.draw(&c, gl);
                }
            });
        }

        if let Some(args) = e.press_args() {
            game.each_player_unit(|unit, game| {
                unit.handle_press(args, game);
            });
        }
    }
}
