use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Add;
use std::str::FromStr;

use lazy_static::lazy_static;
use log::debug;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input18.txt").unwrap();
    // let input = "2,2,2\n1,2,2\n3,2,2\n2,1,2\n2,3,2\n2,2,1\n2,2,3\n2,2,4\n2,2,6\n1,2,5\n3,2,5\n2,1,5\n2,3,5\n";

    let mut grid = Grid3D::new();
    for shape in input.lines().map(|l| Shape::new(l.parse().unwrap())) {
        grid.add_shape(shape);
    }
    println!("{}", grid);
    println!("surface area: {}", grid.total_surface_area());
    println!("external surface area: {}", grid.external_surface_area());
}

struct Grid3D {
    shapes: Vec<Shape>,
}

impl Grid3D {
    pub fn new() -> Self {
        Self {
            shapes: Default::default(),
        }
    }
    pub fn total_surface_area(&self) -> usize {
        self.shapes.iter().map(|s| s.surface_area()).sum()
    }
    pub fn external_surface_area(&self) -> usize {
        let (min_x, max_x, min_y, max_y, min_z, max_z) = self.range();
        let mut wall: HashSet<Side> = HashSet::new();
        let mut empty: HashSet<Coord> = HashSet::new();
        // let mut empty: HashSet<Coord> = (min_x - 1..=max_x + 1)
        //     .flat_map(move |x| (min_y - 1..=max_y + 1).map(move |y| Coord::new(x, y, min_z - 1)))
        //     .chain((min_x - 1..=max_x + 1).flat_map(move |x| {
        //         (min_y - 1..=max_y + 1).map(move |y| Coord::new(x, y, max_z + 2))
        //     }))
        //     .chain((min_x - 1..=max_x + 1).flat_map(move |x| {
        //         (min_z - 1..=max_z + 1).map(move |z| Coord::new(x, min_y - 2, z))
        //     }))
        //     .chain((min_x - 1..=max_x + 1).flat_map(move |x| {
        //         (min_z - 1..=max_z + 1).map(move |z| Coord::new(x, max_y + 2, z))
        //     }))
        //     .chain((min_z - 1..=max_z + 1).flat_map(move |z| {
        //         (min_y - 1..=max_y + 1).map(move |y| Coord::new(min_x - 2, y, z))
        //     }))
        //     .chain((min_z - 1..=max_z + 1).flat_map(move |z| {
        //         (min_y - 1..=max_y + 1).map(move |y| Coord::new(max_x + 2, y, z))
        //     }))
        //     .collect();
        let mut pending: HashSet<Side> = (min_x..=max_x)
            .flat_map(move |x| {
                (min_y..=max_y)
                    .map(move |y| Side::new(Coord::new(x, y, min_z - 1), Direction::PlusZ))
            })
            .chain((min_x..=max_x).flat_map(move |x| {
                (min_y..=max_y)
                    .map(move |y| Side::new(Coord::new(x, y, max_z + 1), Direction::MinusZ))
            }))
            .chain((min_x..=max_x).flat_map(move |x| {
                (min_z..=max_z)
                    .map(move |z| Side::new(Coord::new(x, min_y - 1, z), Direction::PlusY))
            }))
            .chain((min_x..=max_x).flat_map(move |x| {
                (min_z..=max_z)
                    .map(move |z| Side::new(Coord::new(x, max_y + 1, z), Direction::MinusY))
            }))
            .chain((min_z..=max_z).flat_map(move |z| {
                (min_y..=max_y)
                    .map(move |y| Side::new(Coord::new(min_x - 1, y, z), Direction::PlusX))
            }))
            .chain((min_z..=max_z).flat_map(move |z| {
                (min_y..=max_y)
                    .map(move |y| Side::new(Coord::new(max_x + 1, y, z), Direction::MinusX))
            }))
            .collect();
        empty.extend(pending.iter().map(|p| p.coord.clone()));
        debug!(
            "x=[{}, {}], y=[{}, {}], z=[{}, {}]",
            min_x, max_x, min_y, max_y, min_z, max_z
        );
        while !pending.is_empty() {
            debug!(
                "pending={}, wall={}, empty={}",
                pending.len(),
                wall.len(),
                empty.len()
            );
            let next = pending.iter().cloned().next().unwrap();
            pending.remove(&next);
            let opposite = next.opposite();
            debug!("curr={:?}, opp={:?}", next, opposite);
            if wall.contains(&opposite) || empty.contains(&opposite.coord) {
                continue;
            }
            if self.contains_side(&opposite) {
                wall.insert(opposite);
                continue;
            }
            empty.insert(opposite.coord.clone());
            pending.extend(
                Direction::all_directions()
                    .into_iter()
                    .filter(|&d| d != &opposite.direction)
                    .map(|&d| Side::new(opposite.coord.clone(), d)),
            );
        }
        debug!("{:?}", wall);
        wall.len()
    }
    fn range(&self) -> (i32, i32, i32, i32, i32, i32) {
        let iter = self
            .shapes
            .iter()
            .flat_map(|s| s.outer_sides.iter().map(|s| &s.coord));
        let first = iter.clone().next().unwrap();
        iter.fold(
            (first.x, first.x, first.y, first.y, first.z, first.z),
            |mut result, next| {
                result.0 = result.0.min(next.x);
                result.1 = result.1.max(next.x);
                result.2 = result.2.min(next.y);
                result.3 = result.3.max(next.y);
                result.4 = result.4.min(next.z);
                result.5 = result.5.max(next.z);
                result
            },
        )
    }
    fn contains_side(&self, side: &Side) -> bool {
        self.shapes.iter().any(|s| s.outer_sides.contains(side))
    }
    fn get_filled_directions(&self, coord: &Coord) -> Vec<Direction> {
        self.shapes
            .iter()
            .flat_map(|s| s.outer_sides.iter())
            .filter(|s| &s.coord == coord)
            .map(|s| s.direction)
            .collect()
    }
    pub fn add_shape(&mut self, shape: Shape) {
        let sides: Vec<_> = shape.outer_sides.iter().map(|s| s.opposite()).collect();
        let mut matching_sides = HashSet::new();
        let matching: Vec<_> = self
            .shapes
            .iter_mut()
            .enumerate()
            .filter_map(|(i, self_shape)| {
                let removed = self_shape.try_intersect(sides.iter());
                if !removed.is_empty() {
                    matching_sides.extend(removed);
                    Some(i)
                } else {
                    None
                }
            })
            .collect();
        if matching.is_empty() {
            self.shapes.push(shape);
        } else {
            let to_merge: Vec<_> = matching
                .into_iter()
                .rev()
                .map(|i| self.shapes.remove(i))
                .collect();
            let extra_sides: Vec<_> = sides
                .iter()
                .filter(|s| !matching_sides.contains(s))
                .map(|s| s.opposite())
                .collect();
            self.shapes.push(Shape::build_from(
                to_merge
                    .into_iter()
                    .flat_map(|s| s.outer_sides)
                    .chain(extra_sides),
            ));
        }
    }
}

#[derive(Eq, PartialEq)]
struct Shape {
    outer_sides: HashSet<Side>,
}

impl Shape {
    pub fn new(coord: Coord) -> Self {
        Self {
            outer_sides: Direction::all_directions()
                .iter()
                .map(|&d| Side::new(coord.clone(), d))
                .collect(),
        }
    }
    pub fn build_from(sides: impl Iterator<Item = Side>) -> Self {
        Self {
            outer_sides: sides.collect(),
        }
    }
    pub fn try_intersect<'a>(&mut self, sides: impl Iterator<Item = &'a Side>) -> Vec<Side> {
        sides
            .filter_map(|side| {
                if self.outer_sides.remove(side) {
                    Some(side.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn surface_area(&self) -> usize {
        self.outer_sides.len()
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
struct Side {
    coord: Coord,
    direction: Direction,
}

impl Debug for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}{}",
            self.coord,
            match self.direction {
                Direction::PlusX => "+x",
                Direction::MinusX => "-x",
                Direction::PlusY => "+y",
                Direction::MinusY => "-y",
                Direction::PlusZ => "+z",
                Direction::MinusZ => "-z",
            }
        )
    }
}

impl PartialOrd for Side {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.direction == other.direction {
            Some(
                self.coord
                    .projected_in(self.direction)
                    .cmp(&other.coord.projected_in(self.direction)),
            )
        } else {
            None
        }
    }
}

impl Side {
    pub fn new(coord: Coord, direction: Direction) -> Self {
        Self { coord, direction }
    }
    pub fn opposite(&self) -> Self {
        let coord = &self.coord + &Coord::from(self.direction);
        Self {
            coord,
            direction: self.direction.opposite(),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
enum Direction {
    PlusX,
    MinusX,
    PlusY,
    MinusY,
    PlusZ,
    MinusZ,
}

impl Direction {
    pub fn all_directions() -> &'static [Direction] {
        lazy_static! {
            static ref ALL: [Direction; 6] = [
                Direction::PlusX,
                Direction::MinusX,
                Direction::PlusY,
                Direction::MinusY,
                Direction::PlusZ,
                Direction::MinusZ
            ];
        }

        ALL.as_slice()
    }
    pub fn opposite(&self) -> Self {
        match self {
            Direction::PlusX => Direction::MinusX,
            Direction::MinusX => Direction::PlusX,
            Direction::PlusY => Direction::MinusY,
            Direction::MinusY => Direction::PlusY,
            Direction::PlusZ => Direction::MinusZ,
            Direction::MinusZ => Direction::PlusZ,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
struct Coord {
    x: i32,
    y: i32,
    z: i32,
}

impl Coord {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
    pub fn projected_in(&self, direction: Direction) -> i32 {
        match direction {
            Direction::PlusX => self.x,
            Direction::MinusX => -self.x,
            Direction::PlusY => self.y,
            Direction::MinusY => -self.y,
            Direction::PlusZ => self.z,
            Direction::MinusZ => -self.z,
        }
    }
}

impl Debug for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.x, self.y, self.z,)
    }
}

impl From<Direction> for Coord {
    fn from(value: Direction) -> Self {
        match value {
            Direction::PlusX => Coord::new(1, 0, 0),
            Direction::MinusX => Coord::new(-1, 0, 0),
            Direction::PlusY => Coord::new(0, 1, 0),
            Direction::MinusY => Coord::new(0, -1, 0),
            Direction::PlusZ => Coord::new(0, 0, 1),
            Direction::MinusZ => Coord::new(0, 0, -1),
        }
    }
}

impl Add for &Coord {
    type Output = Coord;

    fn add(self, rhs: Self) -> Self::Output {
        Coord {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl FromStr for Coord {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(',');
        Ok(Self {
            x: parts.next().unwrap().parse().unwrap(),
            y: parts.next().unwrap().parse().unwrap(),
            z: parts.next().unwrap().parse().unwrap(),
        })
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Direction::PlusX => ">",
                Direction::MinusX => "<",
                Direction::PlusY => "^",
                Direction::MinusY => "v",
                Direction::PlusZ => "+",
                Direction::MinusZ => "-",
            }
        )
    }
}

impl Display for Grid3D {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (min_x, max_x, min_y, max_y, min_z, max_z) = self.range();
        for z in min_z..=max_z {
            writeln!(
                f,
                "z={}:\n{}",
                z,
                (min_y..=max_y)
                    .map(|y| {
                        format!(
                            "{:02}|{}",
                            y,
                            (min_x..=max_x)
                                .map(|x| {
                                    let coord = Coord::new(x, y, z);
                                    let direction = self.get_filled_directions(&coord);
                                    match direction.len() {
                                        0 => ".".to_string(),
                                        1 => direction[0].to_string(),
                                        more => more.to_string(),
                                    }
                                })
                                .collect::<String>()
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            )?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "2,2,2
1,2,2
3,2,2
2,1,2
2,3,2
2,2,1
2,2,3
2,2,4
2,2,6
1,2,5
3,2,5
2,1,5
2,3,5";

        let mut grid = Grid3D::new();
        for shape in input.lines().map(|l| Shape::new(l.parse().unwrap())) {
            grid.add_shape(shape);
        }
        println!("{}", grid);
        assert_eq!(64, grid.total_surface_area());
        assert_eq!(58, grid.external_surface_area());
    }
}
