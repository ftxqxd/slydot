use super::{Game, CELL_SIZE, CELL_PADDING, CELL_OFFSET_X, CELL_OFFSET_Y, cell_pos};
use grid::Cell;
use std::collections::VecDeque;
use graphics::Context;
use opengl_graphics::GlGraphics;

#[derive(Clone)]
pub struct Unit {
    pub parts: VecDeque<(i16, i16)>,
    pub len_limit: usize,
    pub selected: bool,
    pub moves: u16,
    pub move_limit: u16,
    pub has_attacked: bool,
    pub team: u16,
    pub attack: Option<u16>,
    pub attacks: Vec<Attack>,
    colour: [f32; 3],
    texture: usize,
}

#[derive(Copy)]
pub enum Attack {
    UnitTargetting {
        range: u16,
        /// When the second parameter is `None`, the unit is attacking itself.
        perform: fn(&mut Unit, Option<&mut Unit>) -> bool,
    },
    GroundTargetting {
        range: u16,
        /// Can the attack target solid tiles?
        full: bool,
        /// Can the attack target empty tiles?
        empty: bool,
        perform: fn(&mut Unit, &mut Game, (i16, i16)) -> bool,
    },
}

impl Clone for Attack {
    fn clone(&self) -> Attack {
        match *self {
            Attack::UnitTargetting { range, perform } => {
                Attack::UnitTargetting {
                    range: range,
                    perform: perform,
                }
            }
            Attack::GroundTargetting { range, full, empty, perform } => {
                Attack::GroundTargetting {
                    range: range,
                    full: full,
                    empty: empty,
                    perform: perform,
                }
            }
        }
    }
}

macro_rules! simple_attack {
    ($damage: expr) => {{
        fn tmp(slf: &mut Unit, unit: Option<&mut Unit>) -> bool {
            match unit {
                Some(unit) => unit.damage($damage),
                None => slf.damage($damage),
            }
            true
        }
        tmp
    }}
}

impl Attack {
    pub fn slice() -> Attack {
        Attack::UnitTargetting {
            range: 1,
            perform: simple_attack!(2),
        }
    }

    pub fn dice() -> Attack {
        fn atk(slf: &mut Unit, unit: Option<&mut Unit>) -> bool {
            if slf.parts.len() < 3 {
                return false
            }

            match unit {
                Some(unit) => unit.damage(3),
                None => slf.damage(3),
            }
            true
        }

        Attack::UnitTargetting {
            range: 1,
            perform: atk,
        }
    }

    pub fn thump() -> Attack {
        Attack::UnitTargetting {
            range: 1,
            perform: simple_attack!(3),
        }
    }

    pub fn one() -> Attack {
        fn add_tile(_: &mut Unit, game: &mut Game, coords: (i16, i16)) -> bool {
            game.grid[coords] = Cell::Floor;
            true
        }
        Attack::GroundTargetting {
            range: 3,
            full: false,
            empty: true,
            perform: add_tile,
        }
    }

    pub fn range(&self) -> u16 {
        match *self {
            Attack::UnitTargetting { range, .. }
            | Attack::GroundTargetting { range, .. } => range,
        }
    }
}

impl Unit {
    pub fn from_char(c: char, coords: (i16, i16)) -> Unit {
        match c {
            'A' => Unit::sample_enemy(coords),
            'B' => Unit::sample_enemy2(coords),

            // XXX remove these later
            '1' => Unit::sample(coords),
            '2' => Unit::sample2(coords),
            _ => panic!("unknown enemy code: {}", c),
        }
    }

    pub fn sample(coords: (i16, i16)) -> Unit {
        Unit {
            parts: { let mut v = VecDeque::new(); v.push_back(coords); v },
            len_limit: 4,
            selected: false,
            attack: None,
            moves: 3,
            move_limit: 3,
            has_attacked: false,
            team: 0,
            attacks: vec![Attack::slice(), Attack::dice()],
            colour: [0.0, 0.7490196078431373, 0.9686274509803922],
            texture: 1,
        }
    }

    pub fn sample2(coords: (i16, i16)) -> Unit {
        Unit {
            parts: { let mut v = VecDeque::new(); v.push_back(coords); v },
            len_limit: 4,
            selected: false,
            attack: None,
            moves: 3,
            move_limit: 3,
            has_attacked: false,
            team: 0,
            attacks: vec![Attack::slice(), Attack::dice()],
            colour: [0.5647058823529412, 0.9882352941176471, 0.0],
            texture: 2,
        }
    }

    pub fn sample_enemy(coords: (i16, i16)) -> Unit {
        Unit {
            parts: { let mut v = VecDeque::new(); v.push_back(coords); v },
            len_limit: 5,
            selected: false,
            attack: None,
            moves: 1,
            move_limit: 1,
            has_attacked: false,
            team: 1,
            attacks: vec![Attack::thump()],
            colour: [0.9725490196078431, 0.0, 0.06666666666666667],
            texture: 3,
        }
    }

    pub fn sample_enemy2(coords: (i16, i16)) -> Unit {
        Unit {
            parts: { let mut v = VecDeque::new(); v.push_back(coords); v },
            len_limit: 5,
            selected: false,
            attack: None,
            moves: 1,
            move_limit: 1,
            has_attacked: false,
            team: 1,
            attacks: vec![Attack::thump()],
            colour: [0.5647058823529412, 1.0, 0.9882352941176471],
            texture: 2,
        }
    }

    pub fn is_player(&self, game: &Game) -> bool {
        game.current_team == self.team
    }

    // Why does `move` have to be a keyword? :(
    pub fn relocate(&mut self, game: &mut Game, dx: i16, dy: i16) {
        if self.attack.is_some() { self.move_target(game, dx, dy); return }
        if self.moves == 0 { return }

        let (headx, heady) = self.parts[0];
        let new = (headx + dx, heady + dy);
        if !game.is_valid(new.0, new.1) { return }
        if !game.done {
            game.save_with(self.clone());
            game.done = true;
        }
        if let Some(idx) = self.parts.iter().position(|x| *x == new) {
            let val = self.parts.remove(idx).unwrap();
            self.parts.push_front(val);
            self.moves -= 1;
            self.highlight(game);
            return
        }

        self.parts.push_front((headx + dx, heady + dy));
        if self.parts.len() > self.len_limit {
            self.shorten();
        }
        self.moves -= 1;
        self.highlight(game);
    }

    fn shorten(&mut self) {
        self.parts.pop_back();
    }

    pub fn damage(&mut self, amount: u16) {
        for _ in 0..amount {
            self.shorten();
        }
    }

    pub fn occupies(&self, x: i16, y: i16) -> bool {
        self.parts.iter().any(|&p| p == (x, y))
    }

    pub fn highlight(&self, game: &mut Game) {
        if self.parts.len() == 0 { return }
        game.clear_highlight();
        if let Some(attack) = self.attack {
            let attack = self.attacks[attack as usize];
            self._attack_highlight(game, attack.range(), self.parts[0].0, self.parts[0].1);
            return
        }
        self._highlight(game, self.moves, self.parts[0].0, self.parts[0].1);
        if self.moves == 0 {
            game.grid.player_pos = None;
        } else {
            game.grid.player_pos = Some(self.parts[0]);
        }
    }

    fn _highlight(&self, game: &mut Game, moves: u16, x: i16, y: i16) {
        if !game.is_valid(x, y) { return }
        let pos = x as usize + y as usize*game.grid.width;
        if game.grid.highlight[pos] >= moves + 1 { return }
        game.grid.highlight[pos] = moves + 1;
        if moves == 0 { return }
        self._highlight(game, moves - 1, x + 1, y);
        self._highlight(game, moves - 1, x - 1, y);
        self._highlight(game, moves - 1, x, y + 1);
        self._highlight(game, moves - 1, x, y - 1);
    }

    fn _attack_highlight(&self, game: &mut Game, moves: u16, x: i16, y: i16) {
        let mut good = true;
        let mut allow_empty = false;
        if let Some(attack) = self.attack {
            let atk = self.attacks[attack as usize];
            match atk {
                Attack::GroundTargetting { full, empty, .. } => {
                    if !game.grid.is_in_bounds(x, y) {
                        good = false
                    } else {
                        if (full || game.grid[(x, y)] != Cell::Floor)
                        && (empty || game.grid[(x, y)] != Cell::Empty) {
                            allow_empty = true;
                        } else {
                            good = false;
                        }
                    }
                },
                _ => {},
            }
        }
        if good && (allow_empty || game.grid.is_valid(x, y)) {
            let pos = x as usize + y as usize*game.grid.width;
            if game.grid.attack_hi[pos] >= moves + 1 { return }
            game.grid.attack_hi[pos] = moves + 1;
        }
        if moves == 0 { return }
        self._attack_highlight(game, moves - 1, x + 1, y);
        self._attack_highlight(game, moves - 1, x - 1, y);
        self._attack_highlight(game, moves - 1, x, y + 1);
        self._attack_highlight(game, moves - 1, x, y - 1);
    }

    pub fn attack(&mut self, game: &mut Game, attack: u16) {
        if self.has_attacked { return }
        game.clear_highlight();
        self.attack = Some(attack);
        game.grid.attack_loc = Some(self.parts[0]);
        self.highlight(game);
    }

    fn move_target(&mut self, game: &mut Game, dx: i16, dy: i16) {
        // move_target should only be called when attack_loc & self.attack are Some
        let attack_loc = game.grid.attack_loc.as_mut().unwrap();
        let (tx, ty) = *attack_loc;

        let width = game.grid.width as i16;
        if let Some((_, (x, y))) = game.grid.attack_hi.iter().cloned().enumerate()
                                       .filter_map(|(idx, v)| {
            if v == 0 { return None }
            let idx = idx as i16;
            let (x, y) = (idx % width, idx / width);
            if dx == 1 && x <= tx
            || dx == -1 && x >= tx
            || dy == 1 && y <= ty
            || dy == -1 && y >= ty { return None }
            let (r0, r1);
            if dy == 0 {
                r0 = (y - ty).abs();
                r1 = (x - tx).abs();
            } else {
                r0 = (x - tx).abs();
                r1 = (y - ty).abs();
            }
            Some(((r0, r1), (x, y)))
        }).min() {
            *attack_loc = (x, y);
        }
    }

    pub fn fire(&mut self, game: &mut Game) {
        debug_assert!(self.attack.is_some());
        if let Some(atk) = self.attack {
            let coords = game.grid.attack_loc.unwrap();
            match self.attacks[atk as usize] {
                Attack::UnitTargetting { perform, .. } => {
                    if let Some((idx, _)) = game.units.iter_mut().find(|&(_, ref unit)| {
                        for &ucoords in &unit.parts {
                            if coords == ucoords { return true }
                        }
                        false
                    }) {
                        let mut target_is_kill = false;
                        {
                            let target = &mut game.units[idx];
                            if perform(self, Some(target)) {
                                if target.parts.len() == 0 {
                                    target_is_kill = true;
                                }
                                self.moves = 0;
                                self.has_attacked = true;
                            }
                        }
                        if target_is_kill {
                            // no
                            game.units.remove(&idx);
                        }
                    } else {
                        if self.parts.iter().find(|&&x| x == coords).is_some() {
                            if perform(self, None) {
                                self.moves = 0;
                                self.has_attacked = true;
                            }
                            // deleting self if self.parts.len() == 0 is done in game.rs, fn fire
                        }
                    }
                },
                Attack::GroundTargetting { perform, full, empty, .. } => {
                    if (full || game.grid[coords] != Cell::Floor)
                    && (empty || game.grid[coords] != Cell::Empty) {
                        if perform(self, game, coords) {
                            self.moves = 0;
                            self.has_attacked = true;
                        }
                    }
                },
            }
            self.leave_attack(game);
        }
    }

    pub fn leave_attack(&mut self, game: &mut Game) {
        self.attack = None;
        game.grid.attack_loc = None;
        self.highlight(game);
    }

    pub fn draw(&mut self, game: &Game, c: &Context, gl: &mut GlGraphics) {
        self._draw(game, c, gl);
    }

    fn _draw(&mut self, game: &Game, c: &Context, gl: &mut GlGraphics) {
        use graphics::*;

        let (r, g, b) = (self.colour[0], self.colour[1], self.colour[2]);
        let mut parts: Vec<_> = self.parts.iter().collect();
        parts.sort();

        for &&(x, y) in &parts {
            let is_last = |coords|
                self.len_limit > 1
                && self.moves > 0
                && self.selected
                && self.attack.is_none()
                && self.parts.len() == self.len_limit && coords == self.parts[self.parts.len() - 1];
            let alpha = if is_last((x, y)) { (game.frame / 3 % 2) as f32 } else { 1.0 };
            for i in -1..3 {
                let i = i as f64;
                let mut colour = [[r * 0.57, g * 0.57, b * 0.57, alpha],
                                  [r * 0.57, g * 0.57, b * 0.57, alpha],
                                  [r * 0.57, g * 0.57, b * 0.57, alpha],
                                  [r       , g       , b       , alpha]][(i + 1.0) as usize];
                let mut extra = 0.0;
                if i == 2.0 { extra = 1.0; }
                let rect = [cell_pos(x) - i + extra,
                            cell_pos(y) - i + extra,
                            CELL_SIZE - extra, CELL_SIZE - extra];
                rectangle(colour,
                          rect,
                          c.transform,
                          gl);
                // Draw cell connectors
                if let Some(&a) = self.parts.iter().find(|&&a| a == (x, y + 1)) {
                    let alpha = if is_last((x, y)) || is_last(a) { (game.frame / 3 % 2) as f32 } else { 1.0 };
                    colour[3] = alpha;
                    rectangle(colour,
                              [cell_pos(x) + CELL_SIZE/2.0 - CELL_PADDING/2.0 - i,
                               cell_pos(y) + CELL_SIZE - i,
                               CELL_PADDING + extra, CELL_PADDING + extra],
                              c.transform,
                              gl);
                }
                if let Some(&a) = self.parts.iter().find(|&&a| a == (x - 1, y)) {
                    let alpha = if is_last((x, y)) || is_last(a) { (game.frame / 3 % 2) as f32 } else { 1.0 };
                    colour[3] = alpha;
                    rectangle(colour,
                              [cell_pos(x) - CELL_PADDING - i,
                               cell_pos(y) + CELL_SIZE/2.0 - CELL_PADDING/2.0 - i,
                               CELL_PADDING + extra, CELL_PADDING + extra],
                              c.transform,
                              gl);
                }
            }
        }
        // Draw icon + glow
        let (x, y) = self.parts[0];
        let rect = [CELL_OFFSET_X + x as f64 * (CELL_SIZE + CELL_PADDING) - 1.0,
                    CELL_OFFSET_Y + y as f64 * (CELL_SIZE + CELL_PADDING) - 1.0,
                    CELL_SIZE - 1.0, CELL_SIZE - 1.0];
        Image::new().rect(rect)
              .draw(&game.textures[self.texture], default_draw_state(), c.transform, gl);
        let border = [rect[0], rect[1], rect[2] + 3.5, rect[3] + 2.5];
        if self.selected {
            Rectangle::new_border([1.0, 1.0, 1.0, 1.0 - (game.frame % 40) as f32 / 39.0], 1.0)
                .draw(border, default_draw_state(), c.transform, gl);
        }
    }
}
