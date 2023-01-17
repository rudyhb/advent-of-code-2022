use std::collections::HashSet;
use std::ops::{AddAssign, Sub};
use std::str::FromStr;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input9.txt").unwrap();
    let instructions: Vec<Instruction> = input.lines().map(|l| l.parse().unwrap()).collect();
    let mut grid = Grid::new(2);
    for instruction in instructions.iter() {
        grid.step(instruction);
    }
    println!("tail visits: {}", grid.amount_visited_tail());

    let mut grid = Grid::new(10);
    for instruction in instructions.iter() {
        grid.step(instruction);
    }
    println!(
        "tail visits with 10 segments: {}",
        grid.amount_visited_tail()
    );
}

#[derive(Clone, Default, Eq, PartialEq, Hash)]
struct Coord {
    x: i64,
    y: i64,
}

impl Coord {
    pub fn translate(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.y -= 1,
            Direction::Down => self.y += 1,
            Direction::Left => self.x -= 1,
            Direction::Right => self.x += 1,
        }
    }
    pub fn is_touching(&self, other: &Self) -> bool {
        (self.x - other.x).abs() <= 1 && (self.y - other.y).abs() <= 1
    }
    fn normalize_val(val: &mut i64) {
        if *val != 0 {
            *val = *val / (*val).abs()
        }
    }
    pub fn normalized(mut self) -> Self {
        Self::normalize_val(&mut self.x);
        Self::normalize_val(&mut self.y);
        self
    }
}

impl AddAssign for Coord {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for &Coord {
    type Output = Coord;

    fn sub(self, rhs: Self) -> Self::Output {
        Coord {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

struct Grid {
    segments: Vec<Coord>,
    visited_tail: HashSet<Coord>,
}

impl Grid {
    pub fn new(length: usize) -> Self {
        Self {
            segments: vec![Default::default(); length],
            visited_tail: vec![Default::default()].into_iter().collect(),
        }
    }
    pub fn amount_visited_tail(&self) -> usize {
        self.visited_tail.len()
    }
    pub fn step(&mut self, instruction: &Instruction) {
        for _ in 0..instruction.amount {
            self.segments[0].translate(instruction.direction);
            for i in 1..self.segments.len() {
                let reference = self.segments[i - 1].clone();
                Self::move_segment(&mut self.segments[i], &reference);
            }
            self.update_tail();
        }
    }
    fn move_segment(segment: &mut Coord, reference: &Coord) {
        if segment.is_touching(reference) {
            return;
        }
        *segment += (reference - &*segment).normalized();
    }
    fn update_tail(&mut self) {
        self.visited_tail
            .insert(self.segments.last().unwrap().clone());
    }
}

#[derive(Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl From<char> for Direction {
    fn from(value: char) -> Self {
        match value {
            'U' => Self::Up,
            'D' => Self::Down,
            'L' => Self::Left,
            'R' => Self::Right,
            other => panic!("invalid direction '{}'", other),
        }
    }
}

struct Instruction {
    direction: Direction,
    amount: usize,
}

impl FromStr for Instruction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        Ok(Self {
            direction: Direction::from(parts.next().unwrap().chars().next().unwrap()),
            amount: parts.next().unwrap().parse().unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2";
        let instructions: Vec<Instruction> = input.lines().map(|l| l.parse().unwrap()).collect();
        let mut grid = Grid::new(2);
        for instruction in instructions.iter() {
            grid.step(instruction);
        }
        assert_eq!(13, grid.amount_visited_tail());
    }
    #[test]
    fn test2() {
        let input = "R 5
U 8
L 8
D 3
R 17
D 10
L 25
U 20";
        let instructions: Vec<Instruction> = input.lines().map(|l| l.parse().unwrap()).collect();
        let mut grid = Grid::new(10);
        for instruction in instructions.iter() {
            grid.step(instruction);
        }
        assert_eq!(36, grid.amount_visited_tail());
    }
}
