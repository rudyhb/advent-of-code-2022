use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Add, Sub};

use lazy_static::lazy_static;
use log::{debug, info};

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input17.txt").unwrap();
    for i in 0usize..5 {
        let rock = Rock::new(RockKind::from(i), 0);
        println!("{:?} parts: {:?}", rock, rock.filled_positions());
    }
    let mut tunnel = Tunnel::new(input.chars().map(|c| Direction::from(c)).collect());
    // tunnel.debug = true;
    tunnel.run(2022);
    println!(
        "tower height after 2022 rocks: {}",
        tunnel.highest_position()
    );

    let tunnel = Tunnel::new(input.chars().map(|c| Direction::from(c)).collect());
    let n = 1000000000000;
    println!(
        "tower height after {} rocks: {}",
        n,
        tunnel.run_cached_get_height(n)
    );
}

struct Tunnel {
    settled_rocks: BTreeMap<i64, Vec<Rock>>,
    kind: usize,
    jets: Vec<Direction>,
    jet_index: usize,
    debug: bool,
}

impl Tunnel {
    pub fn new(jets: Vec<Direction>) -> Self {
        Self {
            settled_rocks: Default::default(),
            kind: 0,
            jets,
            jet_index: 0,
            debug: false,
        }
    }
    pub fn run_cached_get_height(mut self, amount: usize) -> i64 {
        let mut heights_cache: HashMap<u64, i64> = HashMap::new();
        let mut hashes = vec![];

        for i in 0..amount {
            let hash = self.get_state_hash();
            if heights_cache.contains_key(&hash) {
                let start = hashes.iter().position(|&h| h == hash).unwrap();
                info!(
                    "found a repeated state at step {}: same as from step {}",
                    i, start
                );
                debug!("\n{}", self);
                let amount = amount - start;
                let times = (amount / (hashes.len() - start)) as i64;
                let j = amount % (hashes.len() - start) + start;
                let start_height = heights_cache.get(&hash).copied().unwrap();
                let end_height = self.highest_position();
                info!("start={}, end={}, j={}", start, hashes.len() - 1, j);
                info!(
                    "times={}, start_h={}, end_h={}, j_h={}",
                    times,
                    start_height,
                    end_height,
                    heights_cache.get(&hashes[j]).copied().unwrap()
                );
                return start_height
                    + times * (end_height - start_height)
                    + (heights_cache.get(&hashes[j]).copied().unwrap() - start_height);
            }
            debug!("step {}:\n{}", i, self);
            hashes.push(hash);
            heights_cache.insert(hash, self.highest_position());
            self.next_rock();
        }

        self.highest_position()
    }
    fn get_state_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.kind.hash(&mut hasher);
        self.jet_index.hash(&mut hasher);
        self.get_floor_shape().hash(&mut hasher);
        hasher.finish()
    }
    fn get_floor_shape(&self) -> Vec<i64> {
        if self.settled_rocks.is_empty() {
            return vec![0; (Coord::MAX_X + 1) as usize];
        }
        let max_y = self.highest_position();
        let shape: Vec<_> = (0..=Coord::MAX_X)
            .map(|x| {
                (1..=max_y)
                    .rev()
                    .filter_map(|y| {
                        let c = Coord::new(x, y);
                        self.get_rocks_in_range(y, y)
                            .flat_map(|r| r.filled_positions())
                            .filter(|coord| &c == coord)
                            .map(|coord| coord.y)
                            .next()
                    })
                    .next()
                    .unwrap_or_default()
            })
            .collect();
        let min = shape.iter().copied().min().unwrap();
        shape.into_iter().map(|s| s - min).collect()
    }
    pub fn run(&mut self, amount: usize) {
        for _ in 0..amount {
            self.next_rock();
            if self.debug {
                println!("{:?}", self.settled_rocks);
                println!("{}", self);
                let _ = std::io::stdin().read_line(&mut String::new());
            }
        }
    }
    fn next_jet(&mut self) -> Direction {
        let dir = self.jets[self.jet_index];
        self.jet_index = (self.jet_index + 1) % self.jets.len();
        dir
    }
    pub fn highest_position(&self) -> i64 {
        self.settled_rocks
            .last_key_value()
            .map(|(i, _)| *i)
            .unwrap_or_default()
    }
    pub fn next_rock(&mut self) {
        let kind = self.kind;
        self.kind = (self.kind + 1) % RockKind::LEN;
        let mut rock = Rock::new(RockKind::from(kind), self.highest_position());
        loop {
            debug!("rock position: {:?}", rock.position);
            rock.try_move(self.next_jet(), self);
            if !rock.try_move(Direction::Down, self) {
                break;
            }
        }
        debug!("deposited rock: {:?}", rock.position);
        self.settled_rocks
            .entry(rock.position.y)
            .or_default()
            .push(rock);
    }
    pub fn get_rocks_in_range(&self, min_y: i64, max_y: i64) -> impl Iterator<Item = &Rock> {
        (min_y..=max_y)
            .filter_map(|y| self.settled_rocks.get(&y))
            .flat_map(|lines| lines)
    }
}

#[derive(Clone, Debug)]
struct Rock {
    position: Coord,
    kind: RockKind,
}

impl Rock {
    pub fn new(kind: RockKind, floor: i64) -> Self {
        Self {
            position: Coord::new(2, floor + 3 + kind.height()),
            kind,
        }
    }
    pub fn try_move(&mut self, direction: Direction, tunnel: &Tunnel) -> bool {
        let mut s = self.clone();
        s.position = &self.position + &(Coord::from(direction));
        let filled = s.filled_positions();
        if filled
            .iter()
            .any(|c| c.y <= 0 || c.x > Coord::MAX_X || c.x < 0)
        {
            return false;
        }
        let max_y = filled.iter().map(|c| c.y).max().unwrap() + 3;
        let min_y = filled.iter().map(|c| c.y).min().unwrap();
        if !tunnel
            .get_rocks_in_range(min_y, max_y)
            .any(|rock| filled.iter().any(|p| rock.is_filled(p)))
        {
            *self = s;
            true
        } else {
            false
        }
    }
    pub fn is_filled(&self, coord: &Coord) -> bool {
        self.kind.is_filled(&(coord - &self.position))
    }
    fn filled_positions(&self) -> Vec<Coord> {
        self.kind
            .filled_positions_from_top_left()
            .map(|coord| &self.position + coord)
            .collect()
    }
}

#[derive(Copy, Clone)]
enum Direction {
    Left,
    Right,
    Down,
}

impl From<Direction> for Coord {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Left => Coord::new(-1, 0),
            Direction::Right => Coord::new(1, 0),
            Direction::Down => Coord::new(0, -1),
        }
    }
}

impl From<char> for Direction {
    fn from(value: char) -> Self {
        match value {
            '<' => Self::Left,
            '>' => Self::Right,
            other => panic!("invalid direction '{}'", other),
        }
    }
}

#[derive(Clone, Debug)]
enum RockKind {
    Flat,
    Cross,
    L,
    I,
    Square,
}

impl RockKind {
    pub const LEN: usize = 5;
    pub fn from(i: usize) -> Self {
        match i {
            0 => Self::Flat,
            1 => Self::Cross,
            2 => Self::L,
            3 => Self::I,
            4 => Self::Square,
            _ => panic!("out of bounds for RockKind"),
        }
    }
    fn set(&self) -> &HashSet<Coord> {
        lazy_static! {
            static ref FLAT: HashSet<Coord> = HashSet::from_iter(
                vec![
                    Coord::new(0, 0),
                    Coord::new(1, 0),
                    Coord::new(2, 0),
                    Coord::new(3, 0),
                ]
                .into_iter()
            );
            static ref CROSS: HashSet<Coord> = HashSet::from_iter(vec![
                Coord::new(1, 0),
                Coord::new(0, -1),
                Coord::new(1, -1),
                Coord::new(2, -1),
                Coord::new(1, -2),
            ]);
            static ref L: HashSet<Coord> = HashSet::from_iter(vec![
                Coord::new(2, 0),
                Coord::new(2, -1),
                Coord::new(2, -2),
                Coord::new(1, -2),
                Coord::new(0, -2),
            ]);
            static ref I: HashSet<Coord> = HashSet::from_iter(vec![
                Coord::new(0, 0),
                Coord::new(0, -1),
                Coord::new(0, -2),
                Coord::new(0, -3),
            ]);
            static ref SQUARE: HashSet<Coord> = HashSet::from_iter(vec![
                Coord::new(0, 0),
                Coord::new(1, 0),
                Coord::new(0, -1),
                Coord::new(1, -1),
            ]);
        }
        match self {
            RockKind::Flat => &FLAT,
            RockKind::Cross => &CROSS,
            RockKind::L => &L,
            RockKind::I => &I,
            RockKind::Square => &SQUARE,
        }
    }
    pub fn filled_positions_from_top_left(&self) -> impl Iterator<Item = &Coord> + '_ {
        self.set().iter()
    }
    pub fn is_filled(&self, coord: &Coord) -> bool {
        self.set().contains(coord)
    }
    pub fn height(&self) -> i64 {
        match self {
            RockKind::Flat => 1,
            RockKind::Cross => 3,
            RockKind::L => 3,
            RockKind::I => 4,
            RockKind::Square => 2,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Coord {
    y: i64,
    x: i64,
}

impl Coord {
    const MAX_X: i64 = 6;
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}

impl Add for &Coord {
    type Output = Coord;

    fn add(self, rhs: Self) -> Self::Output {
        Coord {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
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

impl Display for Tunnel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in (1..=self.highest_position()).rev() {
            writeln!(
                f,
                "|{}|",
                (0..=Coord::MAX_X)
                    .map(|x| {
                        let coord = Coord::new(x, y);
                        if self
                            .get_rocks_in_range(coord.y, coord.y + 3)
                            .any(|r| r.is_filled(&coord))
                        {
                            '#'
                        } else {
                            '.'
                        }
                    })
                    .collect::<String>()
            )?
        }
        write!(f, "{}", "+-------+")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = ">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>";

        let mut tunnel = Tunnel::new(input.chars().map(|c| Direction::from(c)).collect());
        tunnel.run(10);
        let res = "\
|....#..|
|....#..|
|....##.|
|##..##.|
|######.|
|.###...|
|..#....|
|.####..|
|....##.|
|....##.|
|....#..|
|..#.#..|
|..#.#..|
|#####..|
|..###..|
|...#...|
|..####.|
+-------+";
        println!("result:\n{}\n\nactual:\n{}", res, tunnel);
        assert_eq!(res, format!("{}", tunnel));

        let mut tunnel = Tunnel::new(input.chars().map(|c| Direction::from(c)).collect());
        tunnel.run(2022);
        assert_eq!(3068, tunnel.highest_position());

        let test = |i: usize| {
            let tunnel = Tunnel::new(input.chars().map(|c| Direction::from(c)).collect());
            let a = tunnel.run_cached_get_height(i);
            let mut tunnel = Tunnel::new(input.chars().map(|c| Direction::from(c)).collect());
            tunnel.run(i);
            let b = tunnel.highest_position();
            assert_eq!(b, a, "i={}", i)
        };

        test(21);
        test(150);
        test(300);

        let tunnel = Tunnel::new(input.chars().map(|c| Direction::from(c)).collect());
        assert_eq!(1514285714288, tunnel.run_cached_get_height(1000000000000));
    }
}
