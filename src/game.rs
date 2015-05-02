use std::collections::VecDeque;
use std::mem;
use graphics::Context;
use opengl_graphics::GlGraphics;

use super::{Unit, Grid};

pub struct Game {
    pub grid: Grid,
    pub player_units: VecDeque<Unit>,
    pub enemy_units: VecDeque<Unit>,
    pub frame: u64,
}

impl Game {
    pub fn sample() -> Game {
        let mut deque = VecDeque::new();
        deque.push_back(Unit::sample());
        let mut deque2 = VecDeque::new();
        deque2.push_back(Unit::sample_enemy());
        Game {
            grid: Grid::sample(),
            player_units: deque,
            enemy_units: deque2,
            frame: 0,
        }
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
}

pub trait Draw {
    fn draw(&mut self, &Game, &Context, &mut GlGraphics);
}
