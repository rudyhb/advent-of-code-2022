use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use anyhow::{bail, Context};
use itertools::Itertools;
use log::debug;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input22.txt").unwrap();
    let map: Map = input.parse().unwrap();
    debug!("map:\n{:?}", map);
    println!("password: {}", get_password(&map));
    let map = use_custom_edges(map);
    println!("as a cube - password: {}", get_password(&map));
}

fn use_custom_edges(map: Map) -> Map {
    map.with_edges(
        [
            EdgeDescription::new(
                Direction::Up,
                ReversibleRange::new(51, 100),
                1,
                ReversibleRange::new(151, 200),
                1,
                Direction::Right,
            ),
            EdgeDescription::new(
                Direction::Up,
                ReversibleRange::new(101, 150),
                1,
                ReversibleRange::new(1, 50),
                200,
                Direction::Up,
            ),
            EdgeDescription::new(
                Direction::Right,
                ReversibleRange::new(1, 50),
                150,
                ReversibleRange::new(150, 101),
                100,
                Direction::Left,
            ),
            EdgeDescription::new(
                Direction::Down,
                ReversibleRange::new(101, 150),
                50,
                ReversibleRange::new(51, 100),
                100,
                Direction::Left,
            ),
            EdgeDescription::new(
                Direction::Down,
                ReversibleRange::new(51, 100),
                150,
                ReversibleRange::new(151, 200),
                50,
                Direction::Left,
            ),
            EdgeDescription::new(
                Direction::Left,
                ReversibleRange::new(101, 150),
                1,
                ReversibleRange::new(50, 1),
                51,
                Direction::Right,
            ),
            EdgeDescription::new(
                Direction::Up,
                ReversibleRange::new(1, 50),
                101,
                ReversibleRange::new(51, 100),
                51,
                Direction::Right,
            ),
        ]
        .into_iter(),
    )
}

fn get_password(map: &Map) -> u64 {
    let mut position = map.get_initial_position();
    let mut path = Path::new(map);
    path.add(&position);
    debug!("start:\n{}", path);
    for instruction in map.instructions.iter() {
        position = position.next(instruction, map);
        path.add(&position);
        debug!("{:?}:\n{}", instruction, path);
    }
    println!("{}", path);
    println!("final position: {:?}", position);
    position.get_password()
}

struct Path<'a> {
    positions: HashMap<Coord, Position>,
    map: &'a Map,
}

impl<'a> Path<'a> {
    pub fn new(map: &'a Map) -> Self {
        Self {
            positions: Default::default(),
            map,
        }
    }
    pub fn add(&mut self, position: &Position) {
        self.positions
            .insert(position.position.clone(), position.clone());
    }
}

impl<'a> Display for Path<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let max_y = self
            .map
            .edge_connections
            .values()
            .map(|(_, c)| c.y)
            .max()
            .unwrap();
        let max_x = self
            .map
            .edge_connections
            .values()
            .map(|(_, c)| c.x)
            .max()
            .unwrap();
        write!(
            f,
            "{}",
            (1..=max_y)
                .map(|y| {
                    (1..=max_x)
                        .map(|x| {
                            let c = Coord::new(x, y);
                            if let Some(p) = self.positions.get(&c) {
                                char::from(p.direction)
                            } else if let Some(s) = self.map.board.get(&c) {
                                char::from(*s)
                            } else {
                                ' '
                            }
                        })
                        .collect::<String>()
                })
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[derive(Clone, Debug)]
struct Position {
    position: Coord,
    direction: Direction,
}

impl Position {
    pub fn get_password(&self) -> u64 {
        1000 * self.position.y + 4 * self.position.x + (self.direction as u64)
    }
    pub fn next(&self, instruction: &Instruction, map: &Map) -> Self {
        let mut result = self.clone();
        match instruction {
            Instruction::Move(amount) => {
                debug!("starting at {:?}", result.position);
                for _ in 0..*amount {
                    if !result.try_move_forward(map) {
                        debug!("stopped at wall");
                        break;
                    }
                    debug!("{:?}", result.position);
                }
            }
            Instruction::TurnLeft => {
                result.direction = result.direction.turn_left();
            }
            Instruction::TurnRight => {
                result.direction = result.direction.turn_right();
            }
        }
        result
    }
    fn try_move_forward(&mut self, map: &Map) -> bool {
        let next = map
            .board
            .get_key_value(&(self.position.move_towards(self.direction)))
            .map(|c| (self.direction, c))
            .unwrap_or_else(|| {
                // wrap around
                let (new_direction, key) = map
                    .edge_connections
                    .get(&(
                        self.direction,
                        self.direction.perpendicular_component(&self.position),
                    ))
                    .expect("current position is outside edges");
                (
                    *new_direction,
                    map.board
                        .get_key_value(key)
                        .with_context(|| {
                            format!("cannot find key ({:?}) in board for {:?}", key, self)
                        })
                        .unwrap(),
                )
            });
        match next.1 .1 {
            Space::Open => {
                self.position = next.1 .0.clone();
                self.direction = next.0;
                true
            }
            Space::Wall => false,
        }
    }
}

#[derive(Debug)]
struct Map {
    board: HashMap<Coord, Space>,
    instructions: Vec<Instruction>,
    edge_connections: HashMap<(Direction, u64), (Direction, Coord)>,
}

struct EdgeDescription {
    pub side: Direction,
    pub input_range_inclusive: ReversibleRange,
    pub input_other_coordinate: u64,
    pub output_range_inclusive: ReversibleRange,
    pub output_other_coordinate: u64,
    pub output_direction: Direction,
}

impl EdgeDescription {
    pub fn new(
        side: Direction,
        input_range_inclusive: ReversibleRange,
        input_other_coordinate: u64,
        output_range_inclusive: ReversibleRange,
        output_other_coordinate: u64,
        output_direction: Direction,
    ) -> Self {
        Self {
            side,
            input_range_inclusive,
            input_other_coordinate,
            output_range_inclusive,
            output_other_coordinate,
            output_direction,
        }
    }
}

struct ReversibleRange {
    start: u64,
    end: u64,
    reverse: bool,
}

impl ReversibleRange {
    pub fn new(start: u64, end: u64) -> Self {
        Self {
            start,
            end,
            reverse: start > end,
        }
    }
}

impl Iterator for ReversibleRange {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reverse {
            if self.start >= self.end {
                let res = self.start;
                self.start -= 1;
                Some(res)
            } else {
                None
            }
        } else {
            if self.start <= self.end {
                let res = self.start;
                self.start += 1;
                Some(res)
            } else {
                None
            }
        }
    }
}

impl Map {
    fn with_edges(&self, new_edges: impl Iterator<Item = EdgeDescription>) -> Self {
        Self {
            board: self.board.clone(),
            instructions: self.instructions.clone(),
            edge_connections: new_edges
                .flat_map(|edge| {
                    edge.input_range_inclusive
                        .into_iter()
                        .zip_eq(edge.output_range_inclusive.into_iter())
                        .flat_map(move |(input, output)| {
                            [
                                (
                                    (edge.side, input),
                                    (
                                        edge.output_direction,
                                        match edge.output_direction {
                                            Direction::Right | Direction::Left => {
                                                Coord::new(edge.output_other_coordinate, output)
                                            }
                                            Direction::Up | Direction::Down => {
                                                Coord::new(output, edge.output_other_coordinate)
                                            }
                                        },
                                    ),
                                ),
                                (
                                    (edge.output_direction.opposite(), output),
                                    (
                                        edge.side.opposite(),
                                        match edge.side {
                                            Direction::Right | Direction::Left => {
                                                Coord::new(edge.input_other_coordinate, input)
                                            }
                                            Direction::Up | Direction::Down => {
                                                Coord::new(input, edge.input_other_coordinate)
                                            }
                                        },
                                    ),
                                ),
                            ]
                        })
                })
                .collect(),
        }
    }
    fn make_edges(board: &HashMap<Coord, Space>) -> HashMap<(Direction, u64), (Direction, Coord)> {
        let mut keys_x: Vec<_> = board.keys().cloned().collect();
        keys_x.sort_by_key(|c| c.x);
        let mut keys_y = keys_x.clone();
        keys_y.sort_by_key(|c| c.y);
        [
            keys_y
                .iter()
                .group_by(|c| c.y)
                .into_iter()
                .map(|(key, group)| {
                    (
                        (Direction::Left, key),
                        (Direction::Left, group.max_by_key(|c| c.x).cloned().unwrap()),
                    )
                })
                .collect::<Vec<_>>(),
            keys_y
                .iter()
                .group_by(|c| c.y)
                .into_iter()
                .map(|(key, group)| {
                    (
                        (Direction::Right, key),
                        (
                            Direction::Right,
                            group.min_by_key(|c| c.x).cloned().unwrap(),
                        ),
                    )
                })
                .collect::<Vec<_>>(),
            keys_x
                .iter()
                .group_by(|c| c.x)
                .into_iter()
                .map(|(key, group)| {
                    (
                        (Direction::Down, key),
                        (Direction::Down, group.min_by_key(|c| c.y).cloned().unwrap()),
                    )
                })
                .collect::<Vec<_>>(),
            keys_x
                .iter()
                .group_by(|c| c.x)
                .into_iter()
                .map(|(key, group)| {
                    (
                        (Direction::Up, key),
                        (Direction::Up, group.max_by_key(|c| c.y).cloned().unwrap()),
                    )
                })
                .collect::<Vec<_>>(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
    pub fn get_initial_position(&self) -> Position {
        Position {
            position: self
                .board
                .iter()
                .filter(|(_, s)| s == &&Space::Open)
                .map(|(c, _)| c)
                .min_by(|a, b| {
                    let cmp = a.y.cmp(&b.y);
                    if cmp == Ordering::Equal {
                        a.x.cmp(&b.x)
                    } else {
                        cmp
                    }
                })
                .cloned()
                .unwrap(),
            direction: Direction::Right,
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum Space {
    Open,
    Wall,
}

impl Debug for Space {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", char::from(*self))
    }
}

impl From<Space> for char {
    fn from(value: Space) -> Self {
        match value {
            Space::Open => '.',
            Space::Wall => '#',
        }
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
struct Coord {
    x: u64,
    y: u64,
}

impl Coord {
    pub fn new(x: u64, y: u64) -> Self {
        Self { x, y }
    }
    pub fn move_towards(&self, direction: Direction) -> Self {
        let mut result = self.clone();
        match direction {
            Direction::Right => result.x += 1,
            Direction::Down => result.y += 1,
            Direction::Left => result.x = result.x.saturating_sub(1),
            Direction::Up => result.y = result.y.saturating_sub(1),
        }
        result
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum Direction {
    Right = 0,
    Down,
    Left,
    Up,
}

impl From<Direction> for char {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Right => '>',
            Direction::Down => 'v',
            Direction::Left => '<',
            Direction::Up => '^',
        }
    }
}

impl Direction {
    pub fn turn_right(self) -> Self {
        self.turn_left().opposite()
    }
    pub fn turn_left(self) -> Self {
        match self {
            Direction::Right => Direction::Up,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
            Direction::Up => Direction::Left,
        }
    }
    pub fn opposite(self) -> Self {
        match self {
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Up => Direction::Down,
        }
    }
    pub fn perpendicular_component(self, coord: &Coord) -> u64 {
        match self {
            Direction::Right | Direction::Left => coord.y,
            Direction::Down | Direction::Up => coord.x,
        }
    }
}

#[derive(Debug, Clone)]
enum Instruction {
    Move(u64),
    TurnLeft,
    TurnRight,
}

impl FromStr for Map {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let board: HashMap<Coord, Space> = lines
            .by_ref()
            .take_while(|l| !l.is_empty())
            .enumerate()
            .flat_map(|(row, line)| {
                line.chars().enumerate().filter_map(move |(col, c)| {
                    Some((
                        Coord::new(col as u64 + 1, row as u64 + 1),
                        match c {
                            '.' => Space::Open,
                            '#' => Space::Wall,
                            ' ' => return None,
                            other => panic!("invalid character '{}'", other),
                        },
                    ))
                })
            })
            .collect();
        fn parse_num(s: &[char]) -> anyhow::Result<Instruction> {
            Ok(Instruction::Move(
                s.iter().copied().collect::<String>().parse()?,
            ))
        }
        let (mut instructions, leftover) = lines
            .next()
            .context("cannot find instructions")?
            .chars()
            .fold(Ok((vec![], vec![])), |previous, c| {
                let (mut result, mut leftover) = previous?;
                if c.is_numeric() {
                    leftover.push(c);
                } else {
                    if !leftover.is_empty() {
                        result.push(parse_num(&leftover)?);
                        leftover.clear();
                    }
                    result.push(match c {
                        'L' => Instruction::TurnLeft,
                        'R' => Instruction::TurnRight,
                        other => bail!("invalid instruction: '{}'", other),
                    });
                }
                Ok((result, leftover))
            })?;
        if !leftover.is_empty() {
            instructions.push(parse_num(&leftover)?);
        }

        Ok(Self {
            edge_connections: Self::make_edges(&board),
            board,
            instructions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "        ...#
        .#..
        #...
        ....
...#.......#
........#...
..#....#....
..........#.
        ...#....
        .....#..
        .#......
        ......#.

10R5L5R10L4R5L5";

        let map: Map = input.parse().unwrap();
        assert_eq!(6032, get_password(&map));

        let map = map.with_edges(
            [
                EdgeDescription::new(
                    Direction::Right,
                    ReversibleRange::new(5, 8),
                    12,
                    ReversibleRange::new(16, 13),
                    9,
                    Direction::Down,
                ),
                EdgeDescription::new(
                    Direction::Right,
                    ReversibleRange::new(1, 4),
                    12,
                    ReversibleRange::new(12, 9),
                    16,
                    Direction::Left,
                ),
                EdgeDescription::new(
                    Direction::Down,
                    ReversibleRange::new(13, 16),
                    12,
                    ReversibleRange::new(8, 5),
                    1,
                    Direction::Right,
                ),
                EdgeDescription::new(
                    Direction::Down,
                    ReversibleRange::new(9, 12),
                    12,
                    ReversibleRange::new(4, 1),
                    8,
                    Direction::Up,
                ),
                EdgeDescription::new(
                    Direction::Left,
                    ReversibleRange::new(9, 12),
                    9,
                    ReversibleRange::new(8, 5),
                    8,
                    Direction::Up,
                ),
                EdgeDescription::new(
                    Direction::Up,
                    ReversibleRange::new(5, 8),
                    1,
                    ReversibleRange::new(4, 1),
                    5,
                    Direction::Down,
                ),
                EdgeDescription::new(
                    Direction::Left,
                    ReversibleRange::new(1, 4),
                    9,
                    ReversibleRange::new(5, 8),
                    5,
                    Direction::Down,
                ),
            ]
            .into_iter(),
        );
        assert_eq!(5031, get_password(&map));
    }
}
