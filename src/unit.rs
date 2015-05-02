use super::Draw;
use std::{mem, ptr};
use piston::input::{Button, Key};
use graphics::Context;
use opengl_graphics::GlGraphics;

const CELL_SIZE: f64 = 25.0;
const CELL_PADDING: f64 = 5.0;

#[derive(Clone, Debug)]
struct Node<T> {
    val: T,
    children: Vec<Node<T>>,
}

impl<T> Node<T> {
    fn new(val: T) -> Node<T> {
        Node {
            val: val,
            children: vec![],
        }
    }

    fn add(&mut self, val: T) {
        unsafe {
            let old = mem::replace(self, mem::uninitialized());
            ptr::write(self, Node {
                val: val,
                children: vec![old],
            });
        }
    }

    fn parent_of_deepest(&mut self) -> (usize, &mut Node<T>) {
        if self.children.len() == 0 { panic!("tried to remove from one-element tree") }
        if self.children.iter().all(|x| x.children.len() == 0) {
            return (0, self)
        }
        let (depth, v) = self.children.iter_mut().filter(|x| x.children.len() != 0)
                             .map(|x| x.parent_of_deepest()).max_by(|x| x.0).unwrap();
        (depth + 1, v)
    }

    /// Removes the deepest value in the tree.
    fn pop(&mut self) -> T {
        self.parent_of_deepest().1.children.pop().unwrap().val
    }

    fn get_head(&self) -> &T {
        &self.val
    }

    fn set_head(&mut self, idx: usize) {
        let mut elt = self.children.remove(idx);
        unsafe {
            let old = mem::replace(self, mem::uninitialized());
            elt.children.push(old);
            ptr::write(self, elt);
        }
    }

    fn into_vec(self) -> Vec<T> {
        if self.children.len() == 0 { return vec![self.val] }
        self.children.into_iter().flat_map(|x| x.into_vec().into_iter()).chain(Some(self.val).into_iter()).collect()
    }
}

impl<T: PartialEq> Node<T> {
    /// Returns the reversed path to get to `val`.
    fn how_to_get_to(&self, val: &T) -> Option<Vec<usize>> {
        if self.val == *val { return Some(vec![]) }
        let route = self.children.iter().enumerate()
                        .filter_map(|(i, v)| v.how_to_get_to(val).map(|v| (i, v)))
                        .min_by(|&(_, ref v)| v.len());
        route.map(|(idx, mut route)| { route.push(idx); route })
    }

    fn get_to(&mut self, val: &T) -> bool {
        let how = self.how_to_get_to(val);
        if how.is_none() { return false }
        let how = how.unwrap();
        for &i in how.iter().rev() {
            self.set_head(i);
        }
        true
    }
}

pub struct Unit {
    tree: Node<(i16, i16)>,
    len: usize,
    len_limit: usize,
    select: Option<u8>,
    select_change: i8,
}

impl Unit {
    pub fn sample() -> Unit {
        Unit {
            tree: Node::new((0, 0)),
            len: 1,
            len_limit: 10,
            select: Some(0),
            select_change: 1,
        }
    }

    pub fn handle_press(&mut self, press: Button) {
        let (dx, dy);
        match press {
            Button::Keyboard(k) => match k {
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
                _ => return,
            },
            _ => return,
        }

        let (headx, heady) = *self.tree.get_head();
        let new = (headx + dx, heady + dy);
        if self.tree.get_to(&new) {
            return
        }

        self.tree.add((headx + dx, heady + dy));
        if self.len == self.len_limit {
            self.tree.pop();
        } else {
            self.len += 1;
        }
    }
}

impl Draw for Unit {
    fn draw(&mut self, c: &Context, gl: &mut GlGraphics) {
        use graphics::*;

        let r = 1.0;
        let g = 0.5;
        let b = 0.0;

        let mut parts: Vec<_> = self.tree.clone().into_vec();
        parts.sort();

        for &(x, y) in &parts {
            let is_head = (x, y) == *self.tree.get_head();
            let extra = if is_head { self.select.unwrap_or(0) as f32 * 0.004 } else { 0.0 };
            for i in 0..3 {
                let colour = [[r * 0.5  + extra, g * 0.5  + extra, b * 0.5  + extra, 1.0],
                              [r * 0.7  + extra, g * 0.7  + extra, b * 0.7  + extra, 1.0],
                              [r        + extra, g        + extra, b        + extra, 1.0]][i];
                let i = i as f64;
                rectangle(colour,
                          [50.0 + x as f64 * (CELL_SIZE + CELL_PADDING) - i,
                           50.0 + y as f64 * (CELL_SIZE + CELL_PADDING) - i,
                           CELL_SIZE, CELL_SIZE],
                          c.transform,
                          gl);
                let colour = [[r * 0.5, g * 0.5, b * 0.5, 1.0],
                              [r * 0.7, g * 0.7, b * 0.7, 1.0],
                              [r      , g      , b      , 1.0]][i as usize];
                // Draw cell connectors
                if self.tree.how_to_get_to(&(x, y + 1)).is_some() {
                    rectangle(colour,
                              [50.0 + x as f64 * (CELL_SIZE + CELL_PADDING) + CELL_SIZE/2.0 - CELL_PADDING/2.0 - i,
                               50.0 + y as f64 * (CELL_SIZE + CELL_PADDING) + CELL_SIZE - i,
                               CELL_PADDING, CELL_PADDING],
                              c.transform,
                              gl);
                }
                if self.tree.how_to_get_to(&(x - 1, y)).is_some() {
                    rectangle(colour,
                              [50.0 + x as f64 * (CELL_SIZE + CELL_PADDING) - CELL_PADDING - i,
                               50.0 + y as f64 * (CELL_SIZE + CELL_PADDING) + CELL_SIZE/2.0 - CELL_PADDING/2.0 - i,
                               CELL_PADDING, CELL_PADDING],
                              c.transform,
                              gl);
                }
            }
        }
        let select_change = &mut self.select_change;
        self.select.as_mut().map(|x| {
            if *x == 44 && *select_change > 0 {
                *select_change = -1;
            } else if *x == 0 && *select_change < 0 {
                *select_change = 1;
            }
            *x = (*x as i8 + *select_change) as u8;
        });
    }
}
