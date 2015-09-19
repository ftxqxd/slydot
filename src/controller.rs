use super::{Game, CELL_SIZE, CELL_PADDING, CELL_OFFSET_X, CELL_OFFSET_Y};
use piston::input::{Button, Key, MouseButton};

pub trait Controller {
    fn handle_press(&mut self, game: &mut Game, button: Button);
    fn handle_mouse(&mut self, game: &mut Game);
    fn is_local_controlled(&self) -> bool;
}

pub struct DummyController;

impl Controller for DummyController {
    fn handle_press(&mut self, _: &mut Game, _: Button) { panic!() }
    fn handle_mouse(&mut self, _: &mut Game) { panic!() }
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
                    let idx = game.current_team;
                    let len = game.teams.len() as u16;
                    game.select_team((idx + 1) % len);
                },
                _ => {},
            },
            Button::Mouse(MouseButton::Left) => {
                let (x, y) = coords_to_tile(game.mouse);

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

                // Selecting
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

    fn is_local_controlled(&self) -> bool { true }
}
