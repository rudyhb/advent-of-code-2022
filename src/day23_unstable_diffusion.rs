use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Add;
use std::str::FromStr;

use lazy_static::lazy_static;
use log::{debug, info};

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input23.txt").unwrap();

    let mut ground: Ground = input.parse().unwrap();
    ground.run(10);
    println!(
        "empty ground tiles after 10 rounds: {}",
        ground.empty_spaces()
    );

    let mut ground: Ground = input.parse().unwrap();
    println!("elves stop after {} rounds", ground.run_to_end());
}

struct Ground {
    elves: HashSet<Coord>,
    step: usize,
}

impl Ground {
    pub fn run(&mut self, max_steps: u32) {
        info!("start:\n{}", self);
        for i in 0..max_steps {
            if !self.next() {
                break;
            }
            info!("after round {}:\n{}", i + 1, self);
        }
        info!("end:\n{}", self);
    }
    pub fn run_to_end(&mut self) -> u32 {
        info!("start:\n{}", self);
        for i in 0.. {
            if !self.next() {
                info!("end:\n{}", self);
                return i + 1;
            }
            info!("after round {}:\n{}", i + 1, self);
        }
        panic!("cannot finish run");
    }
    pub fn empty_spaces(&self) -> usize {
        let range = self.range();
        let dx = (range.1 - range.0 + 1) as usize;
        let dy = (range.3 - range.2 + 1) as usize;
        dx * dy
            - (range.0..=range.1)
                .flat_map(|x| {
                    (range.2..=range.3).filter(move |&y| {
                        let c = Coord::new(x, y);
                        self.elves.contains(&c)
                    })
                })
                .count()
    }
    fn range(&self) -> (i32, i32, i32, i32) {
        let first = self.elves.iter().next().unwrap();
        self.elves.iter().fold(
            (first.x, first.x, first.y, first.y),
            |(mut min_x, mut max_x, mut min_y, mut max_y), next| {
                min_x = min_x.min(next.x);
                max_x = max_x.max(next.x);
                min_y = min_y.min(next.y);
                max_y = max_y.max(next.y);

                (min_x, max_x, min_y, max_y)
            },
        )
    }
    fn next(&mut self) -> bool {
        let directions = self.get_directions_in_order();
        let (proposed, mapping) = self
            .elves
            .iter()
            .map(|c| {
                (
                    c.clone(),
                    self.get_elf_proposed_next_position(c, &directions),
                )
            })
            .fold::<(HashMap<Coord, u32>, HashMap<Coord, Coord>), _>(
                (Default::default(), Default::default()),
                |(mut proposed, mut mapping), (c, next)| {
                    *proposed.entry(next.clone()).or_default() += 1;
                    mapping.insert(c, next);
                    (proposed, mapping)
                },
            );

        let next: HashSet<_> = self
            .elves
            .iter()
            .map(|c| {
                let next = mapping.get(c).unwrap();
                if proposed.get(&next).unwrap() < &2 {
                    next.clone()
                } else {
                    c.clone()
                }
            })
            .collect();
        assert_eq!(next.len(), self.elves.len());
        if next.iter().all(|c| self.elves.contains(c)) {
            // no one moved
            false
        } else {
            self.elves = next;
            true
        }
    }
    fn get_elf_proposed_next_position(&self, coord: &Coord, directions: &[Direction]) -> Coord {
        if Self::get_neighbors().all(|n| !self.elves.contains(&(coord + &n))) {
            debug!("{:?} is staying still", coord);
            return coord.clone();
        }
        directions
            .iter()
            .copied()
            .filter(|&d| {
                Self::get_neighbors_on_side(d).all(|n| !self.elves.contains(&(coord + &n)))
            })
            .next()
            .map(|d| {
                let next = coord + &Coord::from(d);
                debug!("{:?} is proposing {:?} {:?}", coord, d, next);
                next
            })
            .unwrap_or_else(|| {
                debug!("{:?} has nowhere to go :(", coord);
                coord.clone()
            })
    }
    fn get_neighbors() -> impl Iterator<Item = Coord> {
        (-1..=1).flat_map(move |x| {
            (-1..=1)
                .filter(move |&y| y != 0 || x != 0)
                .map(move |y| Coord::new(x, y))
        })
    }
    fn get_neighbors_on_side(direction: Direction) -> impl Iterator<Item = Coord> {
        let direction_coord = Coord::from(direction);
        Self::get_neighbors()
            .filter(move |c| {
                if c.x < 0 {
                    direction != Direction::East
                } else if c.x > 0 {
                    direction != Direction::West
                } else {
                    true
                }
            })
            .filter(move |c| {
                if c.y < 0 {
                    direction != Direction::South
                } else if c.y > 0 {
                    direction != Direction::North
                } else {
                    true
                }
            })
            .filter(move |c| {
                if c.x == 0 || c.y == 0 {
                    &direction_coord == c
                } else {
                    true
                }
            })
    }
    fn get_directions_in_order(&mut self) -> Vec<Direction> {
        lazy_static! {
            static ref DIRECTIONS: [Direction; 4] = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East
            ];
        }
        let i = self.step;
        self.step += 1;
        (0..4).map(|j| DIRECTIONS[(i + j) % 4]).collect()
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Direction {
    North = 1,
    South = 2,
    East = 4,
    West = 8,
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct Coord {
    x: i32,
    y: i32,
}

impl Debug for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl Coord {
    pub fn new(x: i32, y: i32) -> Self {
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

impl From<Direction> for Coord {
    fn from(value: Direction) -> Self {
        match value {
            Direction::North => Coord::new(0, -1),
            Direction::South => Coord::new(0, 1),
            Direction::East => Coord::new(1, 0),
            Direction::West => Coord::new(-1, 0),
        }
    }
}

impl Display for Ground {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let range = self.range();
        writeln!(f, "{}v / {}>", range.2, range.0)?;
        for y in range.2..=range.3 {
            writeln!(
                f,
                "{}",
                (range.0..=range.1)
                    .map(|x| {
                        let c = Coord::new(x, y);
                        if self.elves.contains(&c) {
                            '#'
                        } else {
                            '.'
                        }
                    })
                    .collect::<String>()
            )?
        }
        Ok(())
    }
}

impl FromStr for Ground {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            elves: s
                .lines()
                .enumerate()
                .flat_map(move |(y, line)| {
                    line.chars()
                        .enumerate()
                        .filter(|(_, c)| *c == '#')
                        .map(move |(x, _)| Coord::new(x as i32, y as i32))
                })
                .collect(),
            step: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "....#..
..###.#
#...#.#
.#...##
#.###..
##.#.##
.#..#..";

        let mut ground: Ground = input.parse().unwrap();
        ground.run(10);
        assert_eq!(
            format!("{}", ground)
                .lines()
                .into_iter()
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n"),
            "\
......#.....
..........#.
.#.#..#.....
.....#......
..#.....#..#
#......##...
....##......
.#........#.
...#.#..#...
............
...#..#..#.."
        );
        assert_eq!(110, ground.empty_spaces());

        let mut ground: Ground = input.parse().unwrap();
        assert_eq!(20, ground.run_to_end());
    }
}
