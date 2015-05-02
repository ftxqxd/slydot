//! Slydot: The Sunrise Event – an original game written in Rust.
//!
//! Any similarity between this game and *Spybot: The Nightfall Incident* is completely
//! coincidental.

extern crate piston;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;

use opengl_graphics::{GlGraphics, OpenGL};
use sdl2_window::Sdl2Window;
use piston::window::WindowSettings;
use piston::event::*;

pub mod game;
pub use game::{Game, Draw};

pub mod unit;
pub use unit::Unit;

pub mod grid;
pub use grid::Grid;

pub const CELL_SIZE: f64 = 28.0;
pub const CELL_PADDING: f64 = 4.0;
pub const CELL_OFFSET_X: f64 = 50.0;
pub const CELL_OFFSET_Y: f64 = 50.0;

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
                clear([0.0, 0.0, 0.0, 1.0], gl);
                game.for_grid(|grid, game| {
                    grid.draw(game, &c, gl);
                });
                game.for_each_player_unit(|unit, game| {
                    unit.draw(game, &c, gl);
                });
                game.for_each_enemy_unit(|unit, game| {
                    unit.draw(game, &c, gl);
                });
            });
            game.frame += 1;
        }

        if let Some(args) = e.press_args() {
            game.for_each_player_unit(|unit, game| {
                unit.handle_press(args, game);
            });
            game.for_each_enemy_unit(|unit, game| {
                unit.handle_press(args, game);
            });
        }
    }
}
