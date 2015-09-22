use super::{Game, CELL_SIZE, CELL_PADDING, CELL_OFFSET_X, CELL_OFFSET_Y};
use piston::input::{Button, Key, MouseButton};

pub trait Controller {
    fn handle_press(&mut self, game: &mut Game, button: Button);
    fn handle_mouse(&mut self, game: &mut Game);
    fn handle_frame(&mut self, game: &mut Game);
    fn is_local_controlled(&self) -> bool;
}

pub struct DummyController;

impl Controller for DummyController {
    fn handle_press(&mut self, _: &mut Game, _: Button) { panic!() }
    fn handle_mouse(&mut self, _: &mut Game) { panic!() }
    fn handle_frame(&mut self, _: &mut Game) { panic!() }
    fn is_local_controlled(&self) -> bool { panic!() }
}

pub struct LocalController;

fn coords_to_tile((mut x, mut y): (f64, f64)) -> (i16, i16) {
    x -= CELL_OFFSET_X;
    y -= CELL_OFFSET_Y;
    x /= CELL_SIZE + CELL_PADDING;
    y /= CELL_SIZE + CELL_PADDING;
    (x as i16, y as i16)
}

impl Controller for LocalController {
    fn handle_press(&mut self, game: &mut Game, button: Button) {
        match button {
            Button::Keyboard(k) => match k {
                Key::Left | Key::Right | Key::Up | Key::Down =>
                    game.for_each_unit(|unit, game, _| {
                        if unit.is_player(game) && unit.selected {
                            let (dx, dy);
                            match k {
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
                                _ => unreachable!(),
                            }
                            unit.relocate(game, dx, dy);
                        }
                    }),
                Key::Tab => {
                    game.select_next();
                },
                Key::Q => { // XXX debugging
                    game.for_each_unit(|unit, _, _| {
                        unit.moves = unit.move_limit;
                        unit.has_attacked = false;
                    });
                    let idx = game.selected_idx.unwrap_or(0);
                    game.select(idx);
                },
                Key::D1 => {
                    let idx = game.selected_idx.unwrap_or(0);
                    game.attack(idx, 0);
                },
                Key::D2 => {
                    let idx = game.selected_idx.unwrap_or(0);
                    game.attack(idx, 1);
                },
                Key::Return => {
                    if let Some(idx) = game.selected_idx {
                        game.fire(idx);
                    }
                },
                Key::Space => {
                    game.next_team();
                },
                Key::U => {
                    game.undo();
                },
                _ => {},
            },
            Button::Mouse(MouseButton::Left) => {
                let (x, y) = coords_to_tile(game.mouse);

                // Selecting
                if game.grid.attack_loc.is_none() {
                    let mut idx = None;
                    game.for_each_unit(|unit, game, i| {
                        if unit.parts[0] == (x, y) && unit.team == game.current_team {
                            idx = Some(i);
                        }
                    });
                    if let Some(idx) = idx {
                        game.select(idx);
                        return
                    }
                }

                let mut success = false;
                // Moving or attacking
                if let Some(idx) = game.selected_idx {
                    let mut attack = false;
                    game.for_unit(idx, |unit, game| {
                        if unit.attack.is_some() {
                            let width = game.grid.width as i16;
                            for (idx, &v) in game.grid.attack_hi.iter().enumerate() {
                                let idx = idx as i16;
                                if v > 0 && (x, y) == (idx % width, idx / width) {
                                    attack = true;
                                    break
                                }
                            }
                        } else {
                            let (hx, hy) = unit.parts[0];
                            if [(hx + 1, hy), (hx - 1, hy),
                                (hx, hy + 1), (hx, hy - 1)].contains(&(x, y)) {
                                unit.relocate(game, x - hx, y - hy);
                                success = true;
                            }
                        }
                    });
                    if attack {
                        game.grid.attack_loc = Some((x, y));
                        game.fire(idx);
                        success = true;
                    }
                }

                if success { return }
            },
            _ => {},
        }
    }

    fn handle_mouse(&mut self, game: &mut Game) {
        let (x, y) = coords_to_tile(game.mouse);
        if let Some(idx) = game.selected_idx {
            game.for_unit(idx, |unit, game| {
                if unit.attack.is_some() {
                    let width = game.grid.width as i16;
                    for (idx, &v) in game.grid.attack_hi.iter().enumerate() {
                        let idx = idx as i16;
                        if v > 0 && (x, y) == (idx % width, idx / width) {
                            game.grid.attack_loc = Some((x, y));
                            break
                        }
                    }
                }
            });
        }
    }

    fn handle_frame(&mut self, _: &mut Game) {}

    fn is_local_controlled(&self) -> bool { true }
}

pub struct AiController {
    delay: u16,
    /// Vector of the postitions of enemy cells & the index of the unit they are part of
    enemy_positions: Vec<(i16, i16, usize)>,
    /// Path to the desired location in reverse order.
    path: Option<Vec<(i16, i16)>>,
}

impl AiController {
    pub fn new() -> AiController {
        AiController {
            delay: 9,
            enemy_positions: vec![],
            path: None,
        }
    }

    fn update_enemy_positions(&mut self, game: &mut Game) {
        self.enemy_positions.clear();
        game.for_each_unit(|unit, game, idx| {
            if unit.is_player(game) { return }
            for &(x, y) in &unit.parts {
                self.enemy_positions.push((x, y, idx));
            }
        });
    }
}

impl Controller for AiController {
    fn handle_press(&mut self, _: &mut Game, _: Button) {}

    fn handle_mouse(&mut self, _: &mut Game) {}

    fn handle_frame(&mut self, game: &mut Game) {
        if self.enemy_positions.is_empty() {
            self.update_enemy_positions(game);
        }

        if self.delay > 0 {
            self.delay -= 1;
            return
        }
        self.delay = 9;
        if let Some(curr) = game.selected_idx {
            if game.grid.attack_loc.is_some() {
                game.fire(curr);
                self.update_enemy_positions(game);
                return
            }

            if game.units[curr].has_attacked {
                let mut all = true;
                game.for_each_unit(|unit, game, _| {
                    if unit.is_player(game) && !unit.has_attacked {
                        all = false;
                    }
                });
                if all {
                    self.enemy_positions = vec![];
                    game.next_team();
                } else {
                    game.select_next();
                }
                return
            }

            let (ux, uy) = game.units[curr].parts[0];

            let range = game.units[curr].attacks.iter().map(|x| x.range()).max().unwrap() as i16;

            let mut empty = false;
            if let Some(ref mut path) = self.path {
                if let Some((dx, dy)) = path.pop() {
                    game.for_unit(curr, |unit, game| {
                        unit.relocate(game, dx, dy);
                    });
                } else {
                    empty = true; // work around #6393
                    if let Some((_, x, y)) = self.enemy_positions.iter().filter_map(|&(x, y, idx)| {
                        let dist = (ux - x).abs() + (uy - y).abs();
                        if dist > range { return None }
                        let health = game.units[idx].parts.len();
                        Some((health, x, y))
                    }).max() {
                        game.attack(curr, 0); // TODO: allow multiple AI attacks?
                        game.grid.attack_loc = Some((x, y));
                    } else {
                        let unit = &mut game.units[curr];
                        unit.has_attacked = true;
                        unit.moves = 0;
                    }
                }
            } else {
                // First, find the target tile
                let width = game.grid.width as i16;
                let (_, mut x, mut y) = game.grid.highlight.iter().enumerate().filter_map(|(i, &v)| {
                    let i = i as i16;
                    if v == 0 { return None }
                    let (ox, oy) = (i % width, i / width);
                    let min_dist = self.enemy_positions.iter().map(|&(x, y, _)| {
                        ((ox - x).abs() + (oy - y).abs()).saturating_sub(range)
                    }).min().unwrap();
                    Some((min_dist, ox, oy))
                }).min().unwrap();

                // Next, find the path to the target tile
                let mut path = Vec::new();
                while (x, y) != (ux, uy) {
                    for &(dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)].iter() {
                        if game.grid.highlight[(x + dx + width*(y + dy)) as usize]
                                == game.grid.highlight[(x + width*y) as usize] + 1 {
                            x += dx;
                            y += dy;
                            path.push((-dx, -dy));
                            break
                        }
                    }
                }
                self.path = Some(path);
            }
            if empty { self.path = None }
        }
    }

    fn is_local_controlled(&self) -> bool { false }
}
