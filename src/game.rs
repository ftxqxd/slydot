use std::collections::VecDeque;
use std::mem;
use piston::input::{Button, Key};
use graphics::Context;
use opengl_graphics::GlGraphics;

use super::{Unit, Grid};

pub struct Game {
    pub grid: Grid,
    pub player_units: VecDeque<Unit>,
    pub enemy_units: VecDeque<Unit>,
    pub frame: u64,
    pub selected_idx: Option<usize>,
}

impl Game {
    pub fn sample() -> Game {
        let mut deque = VecDeque::new();
        deque.push_back(Unit::sample());
        deque.push_back(Unit::sample2());
        let mut deque2 = VecDeque::new();
        deque2.push_back(Unit::sample_enemy());
        Game {
            grid: Grid::sample(),
            player_units: deque,
            enemy_units: deque2,
            frame: 0,
            selected_idx: None,
        }
    }

    pub fn for_player_unit<F>(&mut self, idx: usize, f: F) where F: FnOnce(&mut Unit, &mut Game) {
        let mut unit = self.player_units.remove(idx).unwrap();
        f(&mut unit, self);
        self.player_units.insert(idx, unit);
    }

    pub fn for_each_player_unit<F>(&mut self, mut f: F) where F: FnMut(&mut Unit, &mut Game) {
        for _ in 0..self.player_units.len() {
            let mut unit = self.player_units.pop_front().unwrap();
            f(&mut unit, self);
            self.player_units.push_back(unit);
        }
    }

    pub fn for_each_enemy_unit<F>(&mut self, mut f: F) where F: FnMut(&mut Unit, &mut Game) {
        for _ in 0..self.enemy_units.len() {
            let mut unit = self.enemy_units.pop_front().unwrap();
            f(&mut unit, self);
            self.enemy_units.push_back(unit);
        }
    }

    pub fn for_grid<F>(&mut self, mut f: F) where F: FnMut(&mut Grid, &mut Game) {
        let mut grid = mem::replace(&mut self.grid, Grid::new(vec![], 0));
        f(&mut grid, self);
        self.grid = grid;
    }

    pub fn is_valid(&self, x: i16, y: i16) -> bool {
        self.grid.is_valid(x, y)
            && self.player_units.iter().all(|a| !a.occupies(x, y))
            && self.enemy_units.iter().all(|a| !a.occupies(x, y))
    }

    pub fn select(&mut self, unit_idx: usize) {
        self.deselect();
        self.selected_idx = Some(unit_idx);
        self.player_units[unit_idx].selected = true;
        self.for_player_unit(unit_idx, |unit, game| {
            unit.highlight(game);
        });
    }

    pub fn deselect(&mut self) {
        if let Some(idx) = self.selected_idx {
            self.player_units[idx].selected = false;
        }
        self.selected_idx = None;
    }
    
    pub fn handle_press(&mut self, args: Button) {
        let mut new_idx = None;
        match args {
            Button::Keyboard(k) => match k {
                Key::Left | Key::Right | Key::Up | Key::Down =>
                    self.for_each_player_unit(|unit, game| {
                        if unit.selected { unit.relocate(k, game); }
                    }),
                Key::Tab => if let Some(idx) = self.selected_idx.as_mut() {
                    new_idx = Some((*idx + 1) % self.player_units.len());
                },
                Key::Q => { // debugging
                    self.for_each_player_unit(|unit, _| {
                        unit.moves = 10;
                    });
                    let idx = self.selected_idx.unwrap_or(0);
                    self.select(idx);
                },
                _ => {},
            },
            _ => {},
        }
        if let Some(new_idx) = new_idx {
            self.select(new_idx);
        }
    }

    pub fn draw(&mut self, c: &Context, gl: &mut GlGraphics) {
        use graphics::*;
        clear([0.0, 0.0, 0.0, 1.0], gl);
        self.for_grid(|grid, game| {
            grid.draw(game, &c, gl);
        });
        self.for_each_player_unit(|unit, game| {
            unit.draw(game, &c, gl);
        });
        self.for_each_enemy_unit(|unit, game| {
            unit.draw(game, &c, gl);
        });
        self.for_grid(|grid, game| {
            grid.draw_overlay(game, &c, gl);
        });
    }
}
