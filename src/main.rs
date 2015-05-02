//! Slydot: The Sunrise Event – an original game written in Rust.
//!
//! Any similarity between this game and *Spybot: The Nightfall Incident* is completely
//! coincidental.

#![feature(core)]

extern crate piston;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;

use graphics::Context;
use opengl_graphics::{GlGraphics, OpenGL};
use sdl2_window::Sdl2Window;
use piston::window::WindowSettings;
use piston::event::*;

pub trait Draw {
    fn draw(&mut self, &Context, &mut GlGraphics);
}

mod unit;
use unit::Unit;

fn main() {
    let opengl = OpenGL::_3_2;
    let window = Sdl2Window::new(
        opengl,
        WindowSettings::new("sunrise", [640, 480])
        .exit_on_esc(true)
    );

    let ref mut gl = GlGraphics::new(opengl);
    let mut unit = Unit::sample();
    for e in window.events() {
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, gl| {
                use graphics::*;
                clear([0.125, 0.75, 0.1875, 1.0], gl);
                unit.draw(&c, gl);
            });
        }

        if let Some(args) = e.press_args() {
            unit.handle_press(args);
        }
    }
}
