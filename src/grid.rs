use super::{Game, CELL_SIZE, CELL_PADDING, cell_pos};
use std::ops::{Index, IndexMut};
use graphics::Context;
use opengl_graphics::GlGraphics;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Floor,
}

#[derive(Clone, Debug)]
pub struct Grid {
    pub grid: Vec<Cell>,
    pub width: usize,
    pub highlight: Vec<u16>,
    /// `None` when the player has no moves left or when no unit is selected.
    pub player_pos: Option<(i16, i16)>,
}

macro_rules! grid {
    ($($i:ident)*) => {
        vec![$(Cell::$i),*]
    }
}

impl Grid {
    pub fn new(grid: Vec<Cell>, width: usize) -> Grid {
        let len = grid.len();
        Grid {
            grid: grid,
            width: width,
            highlight: vec![0; len],
            player_pos: None,
        }
    }

    pub fn sample() -> Grid {
        Grid {
            grid: grid![Empty Empty Floor Floor Floor Floor Floor Empty Empty
                        Floor Floor Floor Floor Floor Floor Floor Floor Floor
                        Floor Floor Floor Floor Floor Floor Floor Floor Floor
                        Floor Floor Floor Floor Floor Floor Floor Floor Floor
                        Floor Floor Floor Floor Floor Floor Floor Floor Floor
                        Empty Empty Floor Floor Floor Floor Floor Empty Empty],
            width: 9,
            highlight: vec![0; 9 * 6],
            player_pos: None,
        }
    }

    pub fn height(&self) -> usize {
        self.grid.len() / self.width
    }

    pub fn is_valid(&self, x: i16, y: i16) -> bool {
        if x < 0 || y < 0 || x >= self.width as i16 || y >= self.height() as i16 { return false }
        self[(x as usize, y as usize)] != Cell::Empty
    }

    pub fn draw(&mut self, _: &Game, c: &Context, gl: &mut GlGraphics) {
        use graphics::*;

        for (i, (v, &hi)) in self.grid.iter_mut().zip(self.highlight.iter()).enumerate() {
            let (x, y) = (i % self.width, i / self.width);
            match *v {
                Cell::Empty => {},
                Cell::Floor => {
                    let alpha = if hi != 0 { 0.6 } else { 0.3 };
                    rectangle([1.0, 1.0, 1.0, alpha],
                              [cell_pos(x as i16),
                               cell_pos(y as i16),
                               CELL_SIZE, CELL_SIZE],
                              c.transform,
                              gl);
                }
            }
        }
    }

    pub fn draw_overlay(&mut self, _: &Game, c: &Context, gl: &mut GlGraphics) {
        use graphics::*;

        if let Some((px, py)) = self.player_pos {
            let colour = [1.0, 1.0, 1.0, 0.8];
            if self.is_valid(px + 1, py) {
                polygon(colour,
                        &[[cell_pos(px + 1), cell_pos(py)],
                          [cell_pos(px + 1) + CELL_SIZE/2.0, cell_pos(py) + CELL_SIZE/2.0],
                          [cell_pos(px + 1), cell_pos(py) + CELL_SIZE]],
                        c.transform,
                        gl);
            }
            if self.is_valid(px - 1, py) {
                polygon(colour,
                        &[[cell_pos(px) - CELL_PADDING, cell_pos(py)],
                          [cell_pos(px) - CELL_PADDING - CELL_SIZE/2.0, cell_pos(py) + CELL_SIZE/2.0],
                          [cell_pos(px) - CELL_PADDING, cell_pos(py) + CELL_SIZE]],
                        c.transform,
                        gl);
            }
            if self.is_valid(px, py + 1) {
                polygon(colour,
                        &[[cell_pos(px), cell_pos(py) + CELL_SIZE + CELL_PADDING],
                          [cell_pos(px) + CELL_SIZE/2.0, cell_pos(py) + CELL_SIZE * 1.5 + CELL_PADDING],
                          [cell_pos(px) + CELL_SIZE, cell_pos(py) + CELL_SIZE + CELL_PADDING]],
                        c.transform,
                        gl);
            }
            if self.is_valid(px, py - 1) {
                polygon(colour,
                        &[[cell_pos(px), cell_pos(py) - CELL_PADDING],
                          [cell_pos(px) + CELL_SIZE/2.0, cell_pos(py) - CELL_PADDING - CELL_SIZE/2.0],
                          [cell_pos(px) + CELL_SIZE, cell_pos(py) - CELL_PADDING]],
                        c.transform,
                        gl);
            }
        }
    }
}

impl Index<(usize, usize)> for Grid {
    type Output = Cell;
    fn index(&self, (x, y): (usize, usize)) -> &Cell {
        let width = self.width;
        &self.grid[x + width*y]
    }
}

impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Cell {
        let width = self.width;
        &mut self.grid[x + width*y]
    }
}
