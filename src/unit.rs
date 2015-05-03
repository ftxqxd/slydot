use super::{Game, CELL_SIZE, CELL_PADDING, CELL_OFFSET_X, CELL_OFFSET_Y, cell_pos};
use std::collections::VecDeque;
use std::path::Path;
use piston::input::Key;
use graphics::Context;
use opengl_graphics::{GlGraphics, Texture};

pub struct Unit {
    parts: VecDeque<(i16, i16)>,
    pub len_limit: usize,
    pub selected: bool,
    pub moves: u16,
    pub enemy: bool,
    colour: [f32; 3],
    texture: Texture,
}

impl Unit {
    pub fn sample() -> Unit {
        let tex = Texture::from_path(&Path::new("./assets/lightning.png")).unwrap();
        Unit {
            parts: { let mut v = VecDeque::new(); v.push_back((0, 1)); v },
            len_limit: 4,
            selected: false,
            moves: 10,
            enemy: false,
            colour: [0.0, 0.5647058823529412, 0.9882352941176471],
            texture: tex,
        }
    }

    pub fn sample2() -> Unit {
        let tex = Texture::from_path(&Path::new("./assets/lightning.png")).unwrap();
        Unit {
            parts: { let mut v = VecDeque::new(); v.push_back((2, 0)); v },
            len_limit: 4,
            selected: false,
            moves: 10,
            enemy: false,
            colour: [0.5647058823529412, 0.9882352941176471, 0.0],
            texture: tex,
        }
    }

    pub fn sample_enemy() -> Unit {
        let tex = Texture::from_path(&Path::new("./assets/lightning.png")).unwrap();
        Unit {
            parts: { let mut v = VecDeque::new(); v.push_back((8, 4)); v },
            len_limit: 4,
            selected: false,
            moves: 0,
            enemy: true,
            colour: [0.5647058823529412, 0.0, 0.9882352941176471],
            texture: tex,
        }
    }

    // Why does `move` have to be a keyword? :(
    pub fn relocate(&mut self, key: Key, game: &mut Game) {
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

        if self.moves == 0 { return }

        let (headx, heady) = self.parts[0];
        let new = (headx + dx, heady + dy);
        if !game.is_valid(new.0, new.1) { return }
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

    pub fn occupies(&self, x: i16, y: i16) -> bool {
        self.parts.iter().any(|&p| p == (x, y))
    }

    pub fn highlight(&self, game: &mut Game) {
        game.grid.highlight.iter_mut().map(|x| *x = 0).count();
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

    pub fn draw(&mut self, game: &Game, c: &Context, gl: &mut GlGraphics) {
        use graphics::*;

        let (r, g, b) = (self.colour[0], self.colour[1], self.colour[2]);
        let mut parts: Vec<_> = self.parts.iter().collect();
        parts.sort();

        for &&(x, y) in &parts {
            let is_last = |coords|
                self.len_limit > 1
                && self.moves > 0
                && self.selected
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
        Image::new().rect(rect).draw(&self.texture, default_draw_state(), c.transform, gl);
        let border = [rect[0], rect[1], rect[2] + 3.5, rect[3] + 3.5];
        if self.selected && !self.enemy {
            Rectangle::new_border([1.0, 1.0, 1.0, 1.0 - (game.frame % 40) as f32 / 39.0], 1.0)
                .draw(border, default_draw_state(), c.transform, gl);
        }
    }
}
