use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use anyhow::{bail, Context};
use log::{debug, trace};
use utils::a_star::{a_star_search, AStarNode, AStarOptions, CurrentNodeDetails, Successor};
use utils::pretty_print::PrettyPrint;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input24.txt").unwrap();
    //     let input = "#.######
    // #>>.<^<#
    // #.<..<<#
    // #>v.><>#
    // #<^v^^>#
    // ######.#";
    let valley: Valley = input.parse().unwrap();
    let path = get_shortest_path(valley.clone());
    println!("fewest minutes: {}", path.len() - 1);
    let path = get_shortest_path_3x(valley);
    println!("fewest minutes v2: {}", path.len() - 1);
}

fn get_shortest_path_3x(valley: Valley) -> Vec<State> {
    let Valley {
        time: _,
        position,
        ground,
        blizzards_cache: mut blizzards,
    } = valley;
    let mut state = State { time: 0, position };
    fn distance(details: CurrentNodeDetails<State>) -> i32 {
        details
            .current_node
            .position
            .manhattan_distance(&details.target_node.position)
    }
    let mut path = vec![state.clone()];
    for i in 0..3 {
        let start = state;
        let end = State::new(
            0,
            if i % 2 == 0 {
                ground.get_end_position()
            } else {
                ground.get_initial_position()
            },
        );
        let options = AStarOptions::default()
            .with_no_logs()
            .with_ending_condition(Box::new(|current: &State, end: &State| {
                current.position == end.position
            }));
        path.extend(
            a_star_search(
                start,
                &end,
                |current| {
                    let blizzards = blizzards.get_blizzard_state(current.time + 1, &ground);
                    ground
                        .get_neighbors(&current.position)
                        .chain(vec![current.position.clone()])
                        .filter(|n| !blizzards.has_blizzard(n))
                        .map(|c| Successor::new(State::new(current.time + 1, c), 1))
                        .collect()
                },
                distance,
                Some(&options),
            )
            .map(|res| res.shortest_path)
            .unwrap()
            .into_iter()
            .skip(1),
        );
        state = path.last().unwrap().clone();
    }
    path
}

fn get_shortest_path(valley: Valley) -> Vec<State> {
    debug!("initial position:\n{}", valley);
    let Valley {
        time: _,
        position: start,
        ground,
        blizzards_cache: mut blizzards,
    } = valley;
    debug!(
        "initial blizzards:\n{}",
        blizzards.initial_state.0.pretty_print()
    );
    let start = State::new(0, start);
    let end = State::new(0, ground.get_end_position());
    fn distance(details: CurrentNodeDetails<State>) -> i32 {
        details
            .current_node
            .position
            .manhattan_distance(&details.target_node.position)
    }
    let options = AStarOptions::default()
        // .with_no_logs()
        .with_ending_condition(Box::new(|current: &State, end: &State| {
            current.position == end.position
        }));
    a_star_search(
        start,
        &end,
        |current| {
            let blizzards = blizzards.get_blizzard_state(current.time + 1, &ground);
            trace!(
                "blizzards at t={}:\n{}",
                current.time + 1,
                blizzards.0.pretty_print()
            );
            trace!(
                "possible: {}",
                ground
                    .get_neighbors(&current.position)
                    .collect::<Vec<_>>()
                    .pretty_print()
            );
            trace!(
                "next: {}",
                ground
                    .get_neighbors(&current.position)
                    .chain(vec![current.position.clone()])
                    .filter(|n| !blizzards.has_blizzard(n))
                    .collect::<Vec<_>>()
                    .pretty_print()
            );
            ground
                .get_neighbors(&current.position)
                .chain(vec![current.position.clone()])
                .filter(|n| !blizzards.has_blizzard(n))
                .map(|c| Successor::new(State::new(current.time + 1, c), 1))
                .collect()
        },
        distance,
        Some(&options),
    )
    .map(|res| {
        debug!(
            "shortest path:\nInitial state:\n{}\n{}",
            Valley {
                time: 0,
                position: ground.get_initial_position(),
                ground: ground.clone(),
                blizzards_cache: blizzards.clone(),
            },
            res.shortest_path
                .windows(2)
                .map(|s| {
                    let last = &s[0];
                    let current = &s[1];
                    format!(
                        "Minute {}, {}:\n{}",
                        current.time,
                        current.position.formatted_direction(&last.position),
                        Valley {
                            time: current.time,
                            position: current.position.clone(),
                            ground: ground.clone(),
                            blizzards_cache: blizzards.clone(),
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        );
        res.shortest_path
    })
    .unwrap()
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct State {
    time: u32,
    position: Coord,
}

impl State {
    pub fn new(time: u32, position: Coord) -> Self {
        Self { time, position }
    }
}

impl AStarNode for State {}

#[derive(Clone)]
struct Valley {
    time: u32,
    position: Coord,
    ground: Ground,
    blizzards_cache: BlizzardsCache,
}

impl Valley {
    pub fn new(start: i32, end: i32, height: i32, width: i32, blizzards: BlizzardsState) -> Self {
        let ground = Ground {
            start,
            end,
            height,
            width,
        };
        Self {
            time: 0,
            position: ground.get_initial_position(),
            ground,
            blizzards_cache: BlizzardsCache::new(blizzards),
        }
    }
}

#[derive(Clone)]
struct BlizzardsCache {
    initial_state: BlizzardsState,
    cache: HashMap<u32, BlizzardsState>,
}

impl BlizzardsCache {
    pub fn new(initial_state: BlizzardsState) -> Self {
        Self {
            initial_state,
            cache: Default::default(),
        }
    }
    pub fn try_get_blizzard_state(&self, time: u32) -> Option<&BlizzardsState> {
        if time == 0 {
            return Some(&self.initial_state);
        }
        self.cache.get(&time)
    }
    pub fn get_blizzard_state(&mut self, time: u32, ground: &Ground) -> &BlizzardsState {
        if time == 0 {
            return &self.initial_state;
        }
        if !self.cache.contains_key(&time) {
            let previous = self.get_blizzard_state(time - 1, ground);
            let current = previous.next(ground);
            self.cache.insert(time, current);
        }
        self.cache.get(&time).unwrap()
    }
}

#[derive(Clone)]
struct BlizzardsState(HashMap<Coord, Vec<Direction>>);

impl BlizzardsState {
    pub fn has_blizzard(&self, coord: &Coord) -> bool {
        self.0.contains_key(coord)
    }
    pub fn next(&self, ground: &Ground) -> Self {
        fn add_wrap(val: &mut i32, min: i32, max: i32) {
            if *val == max {
                *val = min;
            } else {
                *val += 1;
            }
        }

        fn sub_wrap(val: &mut i32, min: i32, max: i32) {
            if *val == min {
                *val = max;
            } else {
                *val -= 1;
            }
        }
        Self(
            self.0
                .iter()
                .flat_map(|(c, blizzards)| {
                    blizzards.iter().map(|&d| {
                        let mut c = c.clone();
                        match d {
                            Direction::Up => sub_wrap(&mut c.y, 1, ground.height - 2),
                            Direction::Down => add_wrap(&mut c.y, 1, ground.height - 2),
                            Direction::Left => sub_wrap(&mut c.x, 0, ground.width - 1),
                            Direction::Right => add_wrap(&mut c.x, 0, ground.width - 1),
                        }
                        (c, d)
                    })
                })
                .fold(Default::default(), |mut result, (coord, direction)| {
                    result.entry(coord).or_default().push(direction);
                    result
                }),
        )
    }
}

#[derive(Clone)]
struct Ground {
    start: i32,
    end: i32,
    height: i32,
    width: i32,
}

impl Ground {
    pub fn get_initial_position(&self) -> Coord {
        Coord::new(self.start, 0)
    }
    pub fn get_end_position(&self) -> Coord {
        Coord::new(self.end, self.height - 1)
    }
    pub fn get_neighbors(&self, position: &Coord) -> Neighbors {
        let mut neighbors = Neighbors {
            i: 0,
            position: position.clone(),
            can_move_up: true,
            can_move_down: true,
            can_move_sides: true,
            width: self.width,
        };
        if position.y == 0 {
            neighbors.can_move_sides = false;
            neighbors.can_move_up = false;
        } else if position.y == 1 {
            if position.x != self.start {
                neighbors.can_move_up = false;
            }
        } else if position.y == self.height - 2 {
            if position.x != self.end {
                neighbors.can_move_down = false;
            }
        } else if position.y == self.height - 1 {
            neighbors.can_move_down = false;
            neighbors.can_move_sides = false;
        }
        neighbors
    }
}

struct Neighbors {
    i: u32,
    position: Coord,
    can_move_up: bool,
    can_move_down: bool,
    can_move_sides: bool,
    width: i32,
}

impl Iterator for Neighbors {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i > 3 {
            return None;
        }
        if self.i == 0 {
            self.i += 1;
            if self.can_move_up {
                return Some(Coord {
                    x: self.position.x,
                    y: self.position.y - 1,
                });
            }
        }
        if self.i == 1 {
            self.i += 1;
            if self.can_move_down {
                return Some(Coord {
                    x: self.position.x,
                    y: self.position.y + 1,
                });
            }
        }
        if !self.can_move_sides {
            self.i += 2;
            return None;
        }
        if self.i == 2 {
            self.i += 1;
            if self.position.x > 0 {
                return Some(Coord {
                    x: self.position.x - 1,
                    y: self.position.y,
                });
            }
        }
        self.i += 1;
        if self.position.x < self.width - 1 {
            Some(Coord {
                x: self.position.x + 1,
                y: self.position.y,
            })
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl TryFrom<char> for Direction {
    type Error = ();

    fn try_from(value: char) -> Result<Direction, ()> {
        Ok(match value {
            '^' => Direction::Up,
            'v' => Direction::Down,
            '<' => Direction::Left,
            '>' => Direction::Right,
            _other => return Err(()),
        })
    }
}

impl From<Direction> for char {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Up => '^',
            Direction::Down => 'v',
            Direction::Left => '<',
            Direction::Right => '>',
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct Coord {
    y: i32,
    x: i32,
}

impl Coord {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub fn manhattan_distance(&self, other: &Self) -> i32 {
        self.x.abs_diff(other.x) as i32 + self.y.abs_diff(other.y) as i32
    }
    pub fn formatted_direction(&self, from: &Self) -> &'static str {
        assert!(self.x.abs_diff(from.x) + self.y.abs_diff(from.y) < 2);
        if self == from {
            "wait"
        } else if self.x < from.x {
            "move left"
        } else if self.x > from.x {
            "move right"
        } else if self.y < from.y {
            "move up"
        } else {
            "move down"
        }
    }
}

impl FromStr for Valley {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines().collect::<Vec<_>>();
        let height = lines.len();
        let width = match lines.iter().next().context("empty")?.chars().count() {
            0..=2 => bail!("valley too narrow"),
            l => l - 2,
        };
        let start = lines
            .iter()
            .next()
            .context("empty")?
            .chars()
            .position(|c| c == '.')
            .context("start not found")? as u32
            - 1;
        let end = lines
            .iter()
            .last()
            .context("empty")?
            .chars()
            .position(|c| c == '.')
            .context("end not found")? as u32
            - 1;
        let blizzards: HashMap<Coord, Vec<Direction>> = lines
            .into_iter()
            .enumerate()
            .skip(1)
            .take(height - 2)
            .flat_map(move |(y, line)| {
                line.chars().skip(1).enumerate().filter_map(move |(x, c)| {
                    Direction::try_from(c)
                        .map(|d| {
                            trace!(
                                "blizzard {} at {:?}",
                                char::from(d),
                                Coord::new(x as i32, y as i32)
                            );
                            (Coord::new(x as i32, y as i32), d)
                        })
                        .ok()
                })
            })
            .fold(Default::default(), |mut result, (coord, direction)| {
                result.entry(coord).or_default().push(direction);
                result
            });
        Ok(Self::new(
            start as i32,
            end as i32,
            height as i32,
            width as i32,
            BlizzardsState(blizzards),
        ))
    }
}

impl Display for Valley {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let blizzards = if let Some(b) = self.blizzards_cache.try_get_blizzard_state(self.time) {
            b
        } else {
            return write!(f, "[no blizzard data for t={}]", self.time);
        };
        for y in 0..self.ground.height {
            writeln!(
                f,
                "#{}#",
                (0..self.ground.width)
                    .map(|x| {
                        let c = Coord::new(x, y);
                        if self.position == c {
                            return 'E';
                        }
                        if y == 0 {
                            if x == self.ground.start {
                                '.'
                            } else {
                                '#'
                            }
                        } else if y == self.ground.height - 1 {
                            if x == self.ground.end {
                                '.'
                            } else {
                                '#'
                            }
                        } else {
                            if let Some(b) = blizzards.0.get(&c) {
                                if b.len() == 1 {
                                    char::from(b[0])
                                } else {
                                    b.len().to_string().chars().next().unwrap()
                                }
                            } else {
                                '.'
                            }
                        }
                    })
                    .collect::<String>()
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "#.######
#>>.<^<#
#.<..<<#
#>v.><>#
#<^v^^>#
######.#";
        let valley: Valley = input.parse().unwrap();
        assert_eq!(
            "#E######
#>>.<^<#
#.<..<<#
#>v.><>#
#<^v^^>#
######.#
",
            format!("{}", valley)
        );
        let path = get_shortest_path(valley.clone());
        println!("shortest path: {:?}", path);
        assert_eq!(18, path.len() - 1);
        let path = get_shortest_path_3x(valley);
        println!("shortest path v2: {:?}", path);
        assert_eq!(54, path.len() - 1);
    }
}
