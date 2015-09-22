use vec_map::VecMap;
use std::mem;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use piston::input::Button;
use graphics::Context;
use opengl_graphics::{GlGraphics, Texture};

use super::{Unit, Grid, Controller};
use controller::{LocalController, DummyController, AiController};

pub struct Game {
    pub grid: Grid,
    pub units: VecMap<Unit>,
    pub frame: u64,
    pub mouse: (f64, f64),
    pub selected_idx: Option<usize>,
    pub teams: Vec<Team>,
    pub current_team: u16,
    pub textures: Vec<Texture>,
    pub done: bool,
    pub undo: Vec<UndoState>,
    curr_units: Vec<usize>,
}

pub struct UndoState {
    grid: Grid,
    units: VecMap<Unit>,
    selected_idx: Option<usize>,
}

impl Game {
    pub fn save(&mut self) {
        self.undo.push(UndoState {
            grid: self.grid.clone(),
            units: self.units.clone(),
            selected_idx: self.selected_idx,
        });
    }

    pub fn save_with(&mut self, unit: Unit) {
        let idx = *self.curr_units.last().unwrap();
        let mut units = self.units.clone();
        units.insert(idx, unit);
        self.undo.push(UndoState {
            grid: self.grid.clone(),
            units: units,
            selected_idx: self.selected_idx,
        });
    }

    pub fn undo(&mut self) {
        if let Some(UndoState { grid, units, selected_idx }) = self.undo.pop() {
            self.grid = grid;
            self.units = units;
            self.selected_idx = selected_idx;
            self.done = false;
        }
    }

    pub fn sample() -> Game {
        let mut f = File::open(Path::new("levels/test.sunrise")).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        let (grid, units) = Grid::from_string(&s);

        Game {
            grid: grid,
            units: units,
            frame: 0,
            mouse: (0.0, 0.0),
            selected_idx: None,
            teams: vec![
                Team { name: "Player".into(), controller: Box::new(LocalController) },
                Team { name: "Enemy".into(), controller: Box::new(AiController::new()) },
            ],
            current_team: 0,
            textures: vec![
                Texture::from_memory_alpha(&[], 0, 0).unwrap(),
                Texture::from_path(&Path::new("./assets/hack2.png")).unwrap(),
                Texture::from_path(&Path::new("./assets/lightning.png")).unwrap(),
                Texture::from_path(&Path::new("./assets/warden.png")).unwrap(),
                Texture::from_path(&Path::new("./assets/crosshair.png")).unwrap(),
            ],
            done: false,
            curr_units: vec![],
            undo: vec![],
        }
    }

    pub fn for_unit<F>(&mut self, idx: usize, f: F) where F: FnOnce(&mut Unit, &mut Game) {
        let mut unit = self.units.remove(&idx).unwrap();
        self.curr_units.push(idx);
        f(&mut unit, self);
        self.curr_units.pop();
        self.units.insert(idx, unit);
    }

    pub fn for_each_unit<F>(&mut self, mut f: F) where F: FnMut(&mut Unit, &mut Game, usize) {
        let iter = self.units.keys().collect::<Vec<_>>().into_iter();
        for idx in iter {
            let mut unit = self.units.remove(&idx).unwrap();
            self.curr_units.push(idx);
            f(&mut unit, self, idx);
            self.curr_units.pop();
            self.units.insert(idx, unit);
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
            && self.units.values().all(|a| !a.occupies(x, y))
    }

    pub fn select(&mut self, unit_idx: usize) {
        self.deselect();
        self.selected_idx = Some(unit_idx);
        self.units[unit_idx].selected = true;
        self.for_unit(unit_idx, |unit, game| {
            unit.highlight(game);
        });
        self.done = false;
    }

    pub fn select_team(&mut self, team_idx: u16) {
        self.undo.clear();
        self.for_each_unit(|unit, _, _| {
            unit.moves = unit.move_limit;
            unit.has_attacked = false;
            unit.attack = None;
        });
        let idx = self.units.iter().find(|&(_, ref x)| x.team == team_idx).unwrap().0;
        self.select(idx);
        self.current_team = team_idx;
        self.done = false;
    }

    pub fn next_team(&mut self) {
        let idx = self.current_team;
        let len = self.teams.len() as u16;
        if let Some(idx) = self.selected_idx {
            self.for_unit(idx, |unit, game| {
                unit.leave_attack(game);
            });
        }
        self.select_team((idx + 1) % len);
    }

    pub fn attack(&mut self, unit_idx: usize, attack: u16) {
        self.for_unit(unit_idx, |unit, game| {
            if unit.attack.is_some() {
                game.undo.pop();
                unit.leave_attack(game);
            } else {
                game.save_with(unit.clone());
                unit.attack(game, attack);
            }
        });
    }

    pub fn fire(&mut self, unit_idx: usize) {
        self.for_unit(unit_idx, |unit, game| {
            unit.fire(game);
        });
        if self.units[unit_idx].parts.len() == 0 {
            self.units.remove(&unit_idx);
            let idx = self.units.iter().find(|&(_, ref x)| x.is_player(self)).map(|(i, _)| i);
            if let Some(idx) = idx {
                self.select(idx);
                self.for_unit(idx, |unit, game| {
                    unit.highlight(game);
                });
            }
        }
    }

    pub fn select_next(&mut self) {
        if let Some(idx) = self.selected_idx {
            if self.units[idx].attack.is_none() {
                let mut keys: Vec<_> = self.units.keys().collect();
                keys.sort();
                let mut idx = keys.iter().position(|x| *x == idx).unwrap();
                loop {
                    idx += 1;
                    idx %= keys.len();
                    if self.units[keys[idx]].is_player(self) { break }
                }
                self.select(keys[idx] as usize);
            }
        }
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

    pub fn handle_mouse(&mut self, x: f64, y: f64) {
        self.mouse = (x, y);
        self.for_current_team(|team, game| {
            team.controller.handle_mouse(game);
        });
    }

    pub fn handle_frame(&mut self) {
        self.frame += 1;
        self.for_current_team(|team, game| {
            team.controller.handle_frame(game);
        });
    }

    pub fn draw(&mut self, c: &Context, gl: &mut GlGraphics) {
        use graphics::*;
        clear([0.0, 0.0, 0.0, 1.0], gl);
        self.for_grid(|grid, game| {
            grid.draw(game, &c, gl);
        });
        self.for_each_unit(|unit, game, _| {
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
