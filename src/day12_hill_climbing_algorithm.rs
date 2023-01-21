use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

use derivative::Derivative;
use log::debug;
use utils::a_star::{a_star_search, AStarNode, AStarOptions, CurrentNodeDetails, Successor};

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input12.txt").unwrap();
    let hill: Hill = input.parse().unwrap();
    println!(
        "shortest path: {}",
        find_shortest_path(&hill).expect("no solution found")
    );
    println!(
        "shortest global path: {}",
        find_shortest_global_path_reverse(&hill)
    );
}

#[allow(unused)]
fn find_shortest_global_path(hill: Hill) -> i32 {
    let lowest_height = Square(0);
    let lowest_squares: Vec<_> = hill
        .map
        .iter()
        .filter(|(_, s)| **s == lowest_height)
        .map(|(c, _)| c.clone())
        .collect();
    lowest_squares
        .into_iter()
        .filter_map(|start| {
            let mut hill = hill.clone();
            hill.start = start;
            find_shortest_path(&hill)
        })
        .min()
        .unwrap()
}

fn find_shortest_global_path_reverse(hill: &Hill) -> i32 {
    let get_successors = |current: &State| -> Vec<Successor<State>> {
        let result: Vec<Coord> = current
            .position
            .get_neighbors(hill.max_x, hill.max_y)
            .filter(|c| {
                *c != current.last_position
                    && hill.get_height_difference(&current.position, c) >= -1
            })
            .collect();
        debug!(
            "\n{}\ncurrent: {:?}, next: {:?}",
            HillState {
                hill,
                current: &current.position,
            },
            current.position,
            result
        );
        result
            .into_iter()
            .map(|c| Successor::new(State::next(c, current.position.clone()), 1))
            .collect()
    };
    let distance_function = |details: CurrentNodeDetails<State>| -> i32 {
        hill.map.get(&details.current_node.position).unwrap().0 as i32
    };
    let map = hill.map.clone();
    let end_condition =
        move |current: &State, _: &State| -> bool { map.get(&current.position).unwrap().0 == 0 };
    a_star_search(
        State::new(hill.end.clone()),
        &State::new(hill.start.clone()),
        get_successors,
        distance_function,
        Some(
            &AStarOptions::default()
                .with_ending_condition(Box::new(end_condition))
                .with_no_logs(),
        ),
    )
    .map(|result| result.shortest_path_cost)
    .unwrap()
}

fn find_shortest_path(hill: &Hill) -> Option<i32> {
    let get_successors = |current: &State| -> Vec<Successor<State>> {
        let result: Vec<Coord> = current
            .position
            .get_neighbors(hill.max_x, hill.max_y)
            .filter(|c| {
                *c != current.last_position && hill.get_height_difference(&current.position, c) <= 1
            })
            .collect();
        debug!(
            "\n{}\ncurrent: {:?}, next: {:?}",
            HillState {
                hill,
                current: &current.position,
            },
            current.position,
            result
        );
        result
            .into_iter()
            .map(|c| Successor::new(State::next(c, current.position.clone()), 1))
            .collect()
    };
    let distance_function = |details: CurrentNodeDetails<State>| -> i32 {
        details.current_node.position.manhattan_distance(&hill.end) as i32
    };
    a_star_search(
        State::new(hill.start.clone()),
        &State::new(hill.end.clone()),
        get_successors,
        distance_function,
        Some(&AStarOptions::default().with_no_logs()),
    )
    .map(|result| result.shortest_path_cost)
    .ok()
}

struct HillState<'a> {
    hill: &'a Hill,
    current: &'a Coord,
}

impl<'a> Display for HillState<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            (0..=self.hill.max_y)
                .map(|y| (0..=self.hill.max_x)
                    .map(|x| {
                        let c = Coord::new(x, y);
                        if *self.current == c {
                            '#'
                        } else if c == self.hill.end {
                            'E'
                        } else {
                            char::from(*self.hill.map.get(&c).unwrap())
                        }
                    })
                    .collect::<String>())
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[derive(Debug, Clone)]
struct Hill {
    map: HashMap<Coord, Square>,
    max_x: i64,
    max_y: i64,
    start: Coord,
    end: Coord,
}

impl Hill {
    pub fn get_height_difference(&self, from: &Coord, to: &Coord) -> i32 {
        let h_from = self.map.get(from).unwrap().0 as i32;
        let h_to = if let Some(h) = self.map.get(to) {
            h.0 as i32
        } else {
            panic!("out of bounds - tried to access {:?}", to)
        };
        h_to - h_from
    }
}

#[derive(Derivative)]
#[derivative(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct State {
    position: Coord,
    #[derivative(PartialEq = "ignore")]
    #[derivative(PartialOrd = "ignore")]
    #[derivative(Hash = "ignore")]
    last_position: Coord,
}

impl State {
    pub fn new(position: Coord) -> Self {
        Self {
            last_position: position.clone(),
            position,
        }
    }
    pub fn next(position: Coord, last_position: Coord) -> Self {
        Self {
            position,
            last_position,
        }
    }
}

impl AStarNode for State {}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
struct Coord {
    x: i64,
    y: i64,
}

impl Debug for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl Coord {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
    pub fn get_neighbors(&self, max_x: i64, max_y: i64) -> impl Iterator<Item = Self> + '_ {
        let mut neighbors = Vec::with_capacity(3);
        if self.y > 0 {
            neighbors.push(Coord::new(self.x, self.y - 1));
        }
        if self.y < max_y {
            neighbors.push(Coord::new(self.x, self.y + 1));
        }
        if self.x > 0 {
            neighbors.push(Coord::new(self.x - 1, self.y));
        }
        if self.x < max_x {
            neighbors.push(Coord::new(self.x + 1, self.y));
        }
        neighbors.into_iter()
    }
    pub fn manhattan_distance(&self, other: &Self) -> u64 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

#[derive(Copy, Clone, PartialEq)]
struct Square(u8);

impl Debug for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

impl From<char> for Square {
    fn from(value: char) -> Self {
        Self(value as u8 - 'a' as u8)
    }
}

impl From<Square> for char {
    fn from(value: Square) -> Self {
        (value.0 + 'a' as u8) as char
    }
}

impl FromStr for Hill {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map: HashMap<Coord, Square> = HashMap::new();
        let mut start = None;
        let mut end = None;
        for (coord, c) in s.lines().enumerate().flat_map(|(y, line)| {
            line.chars().enumerate().map(move |(x, c)| {
                let coord = Coord::new(x as i64, y as i64);
                (coord, c)
            })
        }) {
            let height = if c == 'S' {
                start = Some(coord.clone());
                'a'
            } else if c == 'E' {
                end = Some(coord.clone());
                'z'
            } else {
                c
            };
            map.insert(coord, Square::from(height));
        }
        Ok(Self {
            max_x: map.keys().map(|k| k.x).max().unwrap(),
            max_y: map.keys().map(|k| k.y).max().unwrap(),
            map,
            start: start.unwrap(),
            end: end.unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi";
        let hill: Hill = input.parse().unwrap();
        assert_eq!(31, find_shortest_path(&hill).unwrap());
        assert_eq!(29, find_shortest_global_path_reverse(&hill));
    }
}
