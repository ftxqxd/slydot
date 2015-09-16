use std::collections::VecDeque;
use std::mem;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use piston::input::Button;
use graphics::Context;
use opengl_graphics::GlGraphics;

use super::{Unit, Grid, Controller};
use controller::{LocalController, DummyController};

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
        let mut f = File::open(Path::new("levels/test.sunrise")).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        let (grid, units) = Grid::from_string(&s);
        Game {
            grid: grid,
            units: units,
            frame: 0,
            selected_idx: None,
            teams: vec![
                Team { name: "Player".into(), controller: Box::new(LocalController) },
                Team { name: "Enemy".into(), controller: Box::new(LocalController) },
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

    pub fn for_grid<F>(&mut self, f: F) where F: FnOnce(&mut Grid, &mut Game) {
        let mut grid = mem::replace(&mut self.grid, Grid::dummy());
        f(&mut grid, self);
        self.grid = grid;
    }

    pub fn for_current_team<F>(&mut self, f: F) where F: FnOnce(&mut Team, &mut Game) {
        let cur = self.current_team as usize;
        let mut team = mem::replace(&mut self.teams[cur],
                                    Team { name: "Dummy".into(),
                                           controller: Box::new(DummyController) });
        f(&mut team, self);
        self.teams[cur] = team;
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
        self.for_each_unit(|unit, _| {
            unit.moves = unit.move_limit;
            unit.has_attacked = false;
            unit.attack = None;
        });
        let idx = self.units.iter().position(|x| x.team == team_idx).unwrap();
        self.select(idx);
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
            // TODO: handle pause, etc.
            args => {
                self.for_current_team(|team, game| {
                    team.controller.handle_press(game, args);
                });
            },
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
    }
}

pub struct Team {
    pub name: String,
    controller: Box<Controller>,
}
