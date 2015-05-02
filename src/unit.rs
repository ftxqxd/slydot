use super::{Draw, Game, CELL_SIZE, CELL_PADDING, CELL_OFFSET_X, CELL_OFFSET_Y};
use std::collections::VecDeque;
use piston::input::{Button, Key};
use graphics::Context;
use opengl_graphics::GlGraphics;

pub struct Unit {
    parts: VecDeque<(i16, i16)>,
    len_limit: usize,
    select: Option<u8>,
    select_change: i8,
}

impl Unit {
    pub fn sample() -> Unit {
        Unit {
            parts: { let mut v = VecDeque::new(); v.push_back((0, 1)); v },
            len_limit: 10,
            select: Some(0),
            select_change: 1,
        }
    }

    // Why does `move` have to be a keyword? :(
    fn relocate(&mut self, key: Key, game: &mut Game) {
        let (dx, dy);
        match key {
            Key::Up => {
                dx = 0;
                dy = -1;
            },
            Key::Down => {
                dx = 0;
                dy = 1;
            },
            Key::Left => {
                dx = -1;
                dy = 0;
            },
            Key::Right => {
                dx = 1;
                dy = 0;
            },
            _ => panic!("Unit::relocate didn’t receive a direction key"),
        }

        let (headx, heady) = self.parts[0];
        let new = (headx + dx, heady + dy);
        if !game.grid.is_valid(new.0, new.1) { return }
        if let Some(idx) = self.parts.iter().position(|x| *x == new) {
            let val = self.parts.remove(idx).unwrap();
            self.parts.push_front(val);
            return
        }

        self.parts.push_front((headx + dx, heady + dy));
        if self.parts.len() == self.len_limit {
            self.shorten();
        }
    }

    fn shorten(&mut self) {
        self.parts.pop_back();
    }

    pub fn handle_press(&mut self, press: Button, game: &mut Game) {
        match press {
            Button::Keyboard(k) => match k {
                Key::Left | Key::Right | Key::Up | Key::Down => self.relocate(k, game),
                _ => {},
            },
            _ => {},
        }
    }
}

impl Draw for Unit {
    fn draw(&mut self, c: &Context, gl: &mut GlGraphics) {
        use graphics::*;

        let r = 1.0;
        let g = 0.5;
        let b = 0.0;

        let mut parts: Vec<_> = self.parts.iter().collect();
        parts.sort();

        for &&(x, y) in &parts {
            let is_head = (x, y) == self.parts[0];
            let extra = if is_head { self.select.unwrap_or(0) as f32 * 0.004 } else { 0.0 };
            for i in 0..3 {
                let colour = [[r * 0.5 + extra, g * 0.5 + extra, b * 0.5 + extra, 1.0],
                              [r * 0.7 + extra, g * 0.7 + extra, b * 0.7 + extra, 1.0],
                              [r       + extra, g       + extra, b       + extra, 1.0]][i];
                let i = i as f64;
                rectangle(colour,
                          [CELL_OFFSET_X + x as f64 * (CELL_SIZE + CELL_PADDING) - i,
                           CELL_OFFSET_Y + y as f64 * (CELL_SIZE + CELL_PADDING) - i,
                           CELL_SIZE, CELL_SIZE],
                          c.transform,
                          gl);
                let colour = [[r * 0.5, g * 0.5, b * 0.5, 1.0],
                              [r * 0.7, g * 0.7, b * 0.7, 1.0],
                              [r      , g      , b      , 1.0]][i as usize];
                // Draw cell connectors
                if self.parts.iter().find(|a| **a == (x, y + 1)).is_some() {
                    rectangle(colour,
                              [CELL_OFFSET_X + x as f64 * (CELL_SIZE + CELL_PADDING) + CELL_SIZE/2.0 - CELL_PADDING/2.0 - i,
                               CELL_OFFSET_Y + y as f64 * (CELL_SIZE + CELL_PADDING) + CELL_SIZE - i,
                               CELL_PADDING, CELL_PADDING],
                              c.transform,
                              gl);
                }
                if self.parts.iter().find(|a| **a == (x - 1, y)).is_some() {
                    rectangle(colour,
                              [CELL_OFFSET_X + x as f64 * (CELL_SIZE + CELL_PADDING) - CELL_PADDING - i,
                               CELL_OFFSET_Y + y as f64 * (CELL_SIZE + CELL_PADDING) + CELL_SIZE/2.0 - CELL_PADDING/2.0 - i,
                               CELL_PADDING, CELL_PADDING],
                              c.transform,
                              gl);
                }
            }
        }
        let select_change = &mut self.select_change;
        self.select.as_mut().map(|x| {
            if *x == 44 && *select_change > 0 {
                *select_change = -1;
            } else if *x == 0 && *select_change < 0 {
                *select_change = 1;
            }
            *x = (*x as i8 + *select_change) as u8;
        });
    }
}
