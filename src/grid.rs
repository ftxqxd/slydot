use super::{Draw, Game, CELL_SIZE, CELL_PADDING, CELL_OFFSET_X, CELL_OFFSET_Y};
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
    grid: Vec<Cell>,
    width: usize,
}

macro_rules! grid {
    ($($i:ident)*) => {
        vec![$(Cell::$i),*]
    }
}

impl Grid {
    pub fn new(grid: Vec<Cell>, width: usize) -> Grid {
        Grid {
            grid: grid,
            width: width,
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
        }
    }

    pub fn height(&self) -> usize {
        self.grid.len() / self.width
    }

    pub fn is_valid(&self, x: i16, y: i16) -> bool {
        if x < 0 || y < 0 || x >= self.width as i16 || y >= self.height() as i16 { return false }
        self[(x as usize, y as usize)] != Cell::Empty
    }
}

impl Draw for Grid {
    fn draw(&mut self, _: &Game, c: &Context, gl: &mut GlGraphics) {
        use graphics::*;

        for (i, v) in self.grid.iter_mut().enumerate() {
            let (x, y) = (i % self.width, i / self.width);
            match *v {
                Cell::Empty => {},
                Cell::Floor => {
                    rectangle([1.0, 1.0, 1.0, 0.3],
                              [CELL_OFFSET_X + x as f64 * (CELL_SIZE + CELL_PADDING),
                               CELL_OFFSET_Y + y as f64 * (CELL_SIZE + CELL_PADDING),
                               CELL_SIZE, CELL_SIZE],
                              c.transform,
                              gl);
                }
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
