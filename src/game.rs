use std::collections::VecDeque;
use std::mem;
use piston::input::{Button, Key};
use graphics::Context;
use opengl_graphics::GlGraphics;

use super::{Unit, Grid};

pub struct Game {
    pub grid: Grid,
    pub units: VecDeque<Unit>,
    pub frame: u64,
    pub selected_idx: Option<usize>,
    pub teams: Vec<Team>,
    pub current_team: u16,
}

impl Game {
    pub fn sample() -> Game {
        let mut deque = VecDeque::new();
        deque.push_back(Unit::sample());
        deque.push_back(Unit::sample2());
        deque.push_back(Unit::sample_enemy());
        deque.push_back(Unit::sample_enemy2());
        Game {
            grid: Grid::sample(),
            units: deque,
            frame: 0,
            selected_idx: None,
            teams: vec![
                Team { name: "Player".into(), local: true },
                Team { name: "Enemy".into(), local: false },
            ],
            current_team: 0,
        }
    }

    pub fn for_unit<F>(&mut self, idx: usize, f: F) where F: FnOnce(&mut Unit, &mut Game) {
        let mut unit = self.units.remove(idx).unwrap();
        f(&mut unit, self);
        //self.units.insert(idx, unit); XXX restore when stable
        let mut temp = VecDeque::new();
        for _ in 0..idx {
            temp.push_back(self.units.pop_front().unwrap());
        }
        self.units.push_front(unit);
        while let Some(u) = temp.pop_back() {
            self.units.push_front(u);
        }
    }

    pub fn for_each_unit<F>(&mut self, mut f: F) where F: FnMut(&mut Unit, &mut Game) {
        for _ in 0..self.units.len() {
            let mut unit = self.units.pop_front().unwrap();
            f(&mut unit, self);
            self.units.push_back(unit);
        }
    }

    pub fn for_grid<F>(&mut self, mut f: F) where F: FnMut(&mut Grid, &mut Game) {
        let mut grid = mem::replace(&mut self.grid, Grid::new(vec![], 0));
        f(&mut grid, self);
        self.grid = grid;
    }

    pub fn is_valid(&self, x: i16, y: i16) -> bool {
        self.grid.is_valid(x, y)
            && self.units.iter().all(|a| !a.occupies(x, y))
    }

    pub fn select(&mut self, unit_idx: usize) {
        self.deselect();
        self.selected_idx = Some(unit_idx);
        self.units[unit_idx].selected = true;
        self.for_unit(unit_idx, |unit, game| {
            unit.highlight(game);
        });
    }

    pub fn select_team(&mut self, team_idx: u16) {
        self.current_team = team_idx;
    }

    pub fn attack(&mut self, unit_idx: usize, attack: u16) {
        self.for_unit(unit_idx, |unit, game| {
            if unit.attack.is_some() {
                unit.leave_attack(game);
            } else {
                unit.attack(game, attack);
            }
        });
    }

    pub fn attack_next(&mut self, unit_idx: usize) {
        self.for_unit(unit_idx, |unit, game| {
            unit.attack_next(game);
        });
    }

    pub fn fire(&mut self, unit_idx: usize) {
        self.for_unit(unit_idx, |unit, game| {
            unit.fire(game);
        });
    }

    pub fn deselect(&mut self) {
        if let Some(idx) = self.selected_idx {
            self.units[idx].selected = false;
        }
        self.selected_idx = None;
    }

    pub fn clear_highlight(&mut self) {
        self.grid.highlight.iter_mut().map(|x| *x = 0).count();
        self.grid.attack_hi.iter_mut().map(|x| *x = 0).count();
        self.grid.player_pos = None;
    }
    
    pub fn handle_press(&mut self, args: Button) {
        match args {
            Button::Keyboard(k) => match k {
                Key::Left | Key::Right | Key::Up | Key::Down =>
                    self.for_each_unit(|unit, game| {
                        if unit.is_player(game) && unit.selected { unit.relocate(k, game); }
                    }),
                Key::Tab => {
                    if let Some(idx) = self.selected_idx {
                        if self.units[idx].attack.is_some() {
                            self.attack_next(idx);
                        } else {
                            let len = self.units.len();
                            let mut idx = idx;
                            loop {
                                idx += 1;
                                idx %= len;
                                if self.units[idx].is_player(self) { break }
                            }
                            self.select(idx);
                        }
                    }
                },
                Key::Q => { // XXX debugging
                    self.for_each_unit(|unit, _| {
                        unit.moves = unit.move_limit;
                        unit.has_attacked = false;
                    });
                    let idx = self.selected_idx.unwrap_or(0);
                    self.select(idx);
                },
                Key::D1 => {
                    let idx = self.selected_idx.unwrap_or(0);
                    self.attack(idx, 0);
                },
                Key::Return => {
                    if let Some(idx) = self.selected_idx {
                        self.fire(idx);
                    }
                },
                _ => {},
            },
            _ => {},
        }
    }

    pub fn draw(&mut self, c: &Context, gl: &mut GlGraphics) {
        use graphics::*;
        clear([0.0, 0.0, 0.0, 1.0], gl);
        self.for_grid(|grid, game| {
            grid.draw(game, &c, gl);
        });
        self.for_each_unit(|unit, game| {
            unit.draw(game, &c, gl);
        });
        self.for_grid(|grid, game| {
            grid.draw_overlay(game, &c, gl);
        });
        self.for_each_unit(|unit, game| {
            unit.draw_late(game, &c, gl);
        });
    }
}

#[derive(Clone)]
pub struct Team {
    pub name: String,
    local: bool, // TODO: use a controller field (AI/local/network) for this instead
}

impl Team {
    /// Returns true if this team is controlled by a local human player.
    pub fn is_local_controlled(&self) -> bool {
        self.local
    }
}
