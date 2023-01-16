use std::collections::{HashMap, HashSet};
use std::str::FromStr;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input8.txt").unwrap();
    let forest: Forest = input.parse().unwrap();
    println!("trees visible from outside: {}", forest.trees_visible());
    println!("highest scenic score: {}", forest.highest_scenic_score());
}

struct Forest {
    grid: HashMap<Coord, Tree>,
    x_len: usize,
    y_len: usize,
}

#[derive(Clone)]
struct ReversibleRange {
    range_from: usize,
    range_to: usize,
    reverse: bool,
}

impl Iterator for ReversibleRange {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.reverse {
            if self.range_from < self.range_to {
                let val = self.range_from;
                self.range_from += 1;
                Some(val)
            } else {
                None
            }
        } else {
            if self.range_to > self.range_from {
                self.range_to -= 1;
                Some(self.range_to)
            } else {
                None
            }
        }
    }
}

impl Forest {
    pub fn trees_visible(&self) -> usize {
        let mut visible = HashSet::new();
        self.trees_visible_traverse(true, true, &mut visible);
        self.trees_visible_traverse(false, true, &mut visible);
        self.trees_visible_traverse(true, false, &mut visible);
        self.trees_visible_traverse(false, false, &mut visible);
        visible.len()
    }
    fn trees_visible_traverse(&self, is_x: bool, low_to_high: bool, visible: &mut HashSet<Coord>) {
        let (max_first, max_second) = if is_x {
            (self.x_len, self.y_len)
        } else {
            (self.y_len, self.x_len)
        };
        let iter = ReversibleRange {
            range_from: 0,
            range_to: max_second,
            reverse: !low_to_high,
        };
        for first in 0..max_first {
            let mut iter = iter.clone();
            let mut coord = if is_x {
                Coord::new(first, iter.next().unwrap())
            } else {
                Coord::new(iter.next().unwrap(), first)
            };
            visible.insert(coord.clone());
            let mut max = self.grid.get(&coord).unwrap().0;
            for second in iter {
                if is_x {
                    coord.y = second;
                } else {
                    coord.x = second;
                }
                let val = self.grid.get(&coord).unwrap().0;
                if val > max {
                    max = val;
                    visible.insert(coord.clone());
                }
            }
        }
    }
    fn trees_visible_traverse_from(&self, coord: &Coord, on_x: bool, positive_dir: bool) -> usize {
        let iter: ReversibleRange = if on_x {
            if positive_dir {
                ReversibleRange {
                    range_from: coord.x + 1,
                    range_to: self.x_len,
                    reverse: false,
                }
            } else {
                ReversibleRange {
                    range_from: 0,
                    range_to: coord.x,
                    reverse: true,
                }
            }
        } else {
            if positive_dir {
                ReversibleRange {
                    range_from: coord.y + 1,
                    range_to: self.y_len,
                    reverse: false,
                }
            } else {
                ReversibleRange {
                    range_from: 0,
                    range_to: coord.y,
                    reverse: true,
                }
            }
        };
        let height = self.grid.get(coord).unwrap().0;
        let mut n = 0;
        for i in iter {
            n += 1;
            let c = if on_x {
                Coord::new(i, coord.y)
            } else {
                Coord::new(coord.x, i)
            };
            if self.grid.get(&c).unwrap().0 >= height {
                break;
            }
        }
        n
    }
    fn get_tree_score(&self, coord: &Coord) -> usize {
        self.trees_visible_traverse_from(coord, true, true)
            * self.trees_visible_traverse_from(coord, true, false)
            * self.trees_visible_traverse_from(coord, false, true)
            * self.trees_visible_traverse_from(coord, false, false)
    }
    pub fn highest_scenic_score(&self) -> usize {
        (0..self.x_len)
            .flat_map(|x| (0..self.y_len).map(move |y| (x, y)))
            .map(|(x, y)| {
                let coord = Coord::new(x, y);
                self.get_tree_score(&coord)
            })
            .max()
            .unwrap()
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
struct Coord {
    x: usize,
    y: usize,
}

impl Coord {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

struct Tree(u8);

impl Tree {
    pub fn new(height: u8) -> Self {
        Self(height)
    }
}

impl FromStr for Forest {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grid: HashMap<Coord, Tree> = s
            .lines()
            .enumerate()
            .flat_map(|(y, line)| {
                line.chars()
                    .enumerate()
                    .map(move |(x, c)| (Coord::new(x, y), Tree::new(c.to_digit(10).unwrap() as u8)))
            })
            .collect();

        let x_len = grid.keys().map(|c| c.x).max().unwrap() + 1;
        let y_len = grid.keys().map(|c| c.y).max().unwrap() + 1;

        Ok(Self { grid, x_len, y_len })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "30373
25512
65332
33549
35390";
        let forest: Forest = input.parse().unwrap();
        assert_eq!(21, forest.trees_visible());
        assert_eq!(8, forest.highest_scenic_score());
    }
}
