use super::Game;
use piston::input::{Button, Key};

pub trait Controller {
    fn handle_press(&mut self, game: &mut Game, button: Button);
    fn is_local_controlled(&self) -> bool;
}

pub struct DummyController;

impl Controller for DummyController {
    fn handle_press(&mut self, _: &mut Game, _: Button) { panic!() }
    fn is_local_controlled(&self) -> bool { panic!() }
}

pub struct LocalController;

impl Controller for LocalController {
    fn handle_press(&mut self, game: &mut Game, button: Button) {
        match button {
            Button::Keyboard(k) => match k {
                Key::Left | Key::Right | Key::Up | Key::Down =>
                    game.for_each_unit(|unit, game| {
                        if unit.is_player(game) && unit.selected { unit.relocate(k, game); }
                    }),
                Key::Tab => {
                    if let Some(idx) = game.selected_idx {
                        if game.units[idx].attack.is_none() {
                            let len = game.units.len();
                            let mut idx = idx;
                            loop {
                                idx += 1;
                                idx %= len;
                                if game.units[idx].is_player(game) { break }
                            }
                            game.select(idx);
                        }
                    }
                },
                Key::Q => { // XXX debugging
                    game.for_each_unit(|unit, _| {
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
            _ => {},
        }
    }

    fn is_local_controlled(&self) -> bool { true }
}
