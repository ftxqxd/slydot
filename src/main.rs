//! Slydot: The Sunrise Event
//! =========================
//!
//! An original game written in Rust.
//!
//! Any similarity between this game and *Spybot: The Nightfall Incident* is purely coincidental.

extern crate piston;
extern crate piston_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate vec_map;

use piston::input::*;
use piston::window::WindowSettings;
use opengl_graphics::*;
use piston_window::PistonWindow;

pub mod game;
pub use game::Game;

pub mod unit;
pub use unit::Unit;

pub mod grid;
pub use grid::Grid;

pub mod controller;
pub use controller::Controller;

pub const CELL_SIZE: f64 = 28.0;
pub const CELL_PADDING: f64 = 4.0;
pub const CELL_OFFSET_X: f64 = 50.0;
pub const CELL_OFFSET_Y: f64 = 50.0;

pub fn cell_pos(a: i16) -> f64 {
    CELL_OFFSET_X + a as f64 * (CELL_SIZE + CELL_PADDING)
}

fn main() {
    let opengl = OpenGL::V3_2;
    let window: PistonWindow =
        WindowSettings::new("sunrise", [640, 480])
        .opengl(opengl)
        .build()
        .unwrap();

    let ref mut gl = GlGraphics::new(opengl);
    let mut game = Game::sample();
    let idx = game.units.iter().find(|&(_, ref x)| x.team == 0).unwrap().0;
    game.select(idx);
    game.save();
    for e in window {
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, gl| {
                game.draw(&c, gl);
            });
            game.handle_frame();
        }

        if let Some(a) = e.mouse_cursor_args() {
            game.handle_mouse(a[0], a[1]);
        }
        if let Some(b) = e.press_args() {
            game.handle_press(b);
        }
    }
}
