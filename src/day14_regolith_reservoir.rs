use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::Duration;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input14.txt").unwrap();
    println!(
        "{} units of sand come to rest.",
        simulate_falling_sand(&input, false, false)
    );
    println!(
        "with floor: {} units of sand come to rest.",
        simulate_falling_sand(&input, false, true)
    );
}

fn simulate_falling_sand(input: &str, draw: bool, has_floor: bool) -> usize {
    let rock_path: Vec<Path> = input.lines().map(|l| l.parse().unwrap()).collect();
    let mut scan = Scan2D::new(Coord::new(500, 0), &rock_path, has_floor);
    if draw {
        let mut canvas = utils::canvas::Canvas::new()
            .unwrap()
            .with_delay(Duration::from_millis(200));
        while scan.add_sand_grain() {
            canvas.draw(&format!("{}", scan)).unwrap();
        }
    } else {
        while scan.add_sand_grain() {}
    }
    println!("{}", scan);
    scan.settled_sand.len()
}

struct Scan2D {
    rock: HashSet<Coord>,
    settled_sand: HashSet<Coord>,
    sand_entry: Coord,
    falling_sand: HashSet<Coord>,
    has_floor: bool,
    drop_off_zone_y: i64,
}

impl Scan2D {
    pub fn new(sand_entry: Coord, rock_path: &[Path], has_floor: bool) -> Self {
        let rock: HashSet<Coord> = rock_path
            .iter()
            .flat_map(|path| {
                path.0
                    .windows(2)
                    .flat_map(|range| range[0].path_to_inclusive(&range[1]))
            })
            .collect();
        let drop_off_zone_y = rock.iter().map(|r| r.y).max().unwrap() + 2;
        Self {
            rock,
            settled_sand: Default::default(),
            sand_entry,
            falling_sand: Default::default(),
            has_floor,
            drop_off_zone_y,
        }
    }
    pub fn add_sand_grain(&mut self) -> bool {
        let mut position = self.sand_entry.clone();
        let mut path = Vec::new();
        'outer: loop {
            if position.y > self.drop_off_zone_y {
                self.falling_sand = path.into_iter().collect();
                return false;
            }
            let mut next = NextPosition::new(&position);
            while let Some(next) = next.next() {
                if !self.is_blocked(&next) {
                    path.push(next.clone());
                    position = next;
                    continue 'outer;
                }
            }
            break;
        }
        self.settled_sand.insert(position.clone());
        position != self.sand_entry
    }
    fn is_blocked(&self, coord: &Coord) -> bool {
        self.rock.contains(coord)
            || self.settled_sand.contains(coord)
            || (self.has_floor && coord.y >= self.drop_off_zone_y)
    }
    fn range(&self) -> ((i64, i64), (i64, i64)) {
        let max_y = self.rock.iter().map(|c| c.y).max().unwrap();
        let max_x = self.rock.iter().map(|c| c.x).max().unwrap();
        let min_x = self.rock.iter().map(|c| c.x).min().unwrap();
        ((0, max_y), (min_x, max_x))
    }
}

struct NextPosition<'a> {
    coord: &'a Coord,
    iter: u16,
}

impl<'a> NextPosition<'a> {
    pub fn new(coord: &'a Coord) -> Self {
        Self { coord, iter: 0 }
    }
    pub fn next(&mut self) -> Option<Coord> {
        self.iter += 1;
        Some(match self.iter {
            1 => Coord::new(self.coord.x, self.coord.y + 1),
            2 => Coord::new(self.coord.x - 1, self.coord.y + 1),
            3 => Coord::new(self.coord.x + 1, self.coord.y + 1),
            _ => return None,
        })
    }
}

impl Display for Scan2D {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ((min_y, max_y), (min_x, max_x)) = self.range();
        for y in min_y..=(max_y + 1) {
            writeln!(
                f,
                "{}",
                (min_x..=max_x)
                    .map(|x| {
                        let c = &Coord::new(x, y);
                        if self.rock.contains(c) {
                            '#'
                        } else if self.settled_sand.contains(c) {
                            'o'
                        } else if &self.sand_entry == c {
                            '+'
                        } else if self.falling_sand.contains(c) {
                            '~'
                        } else {
                            '.'
                        }
                    })
                    .collect::<String>()
            )?;
        }
        Ok(())
    }
}

struct Path(Vec<Coord>);

impl FromStr for Path {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split(" -> ")
                .map(|s| {
                    let mut parts = s.split(',');
                    Coord::new(
                        parts.next().unwrap().parse().unwrap(),
                        parts.next().unwrap().parse().unwrap(),
                    )
                })
                .collect(),
        ))
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
struct Coord {
    x: i64,
    y: i64,
}

impl Coord {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
    pub fn path_to_inclusive(&self, other: &Self) -> impl Iterator<Item = Self> + '_ {
        let horizontal = self.y == other.y;
        assert_eq!(self.x == other.x, !horizontal);
        let range = if horizontal {
            if self.x < other.x {
                (self.x, other.x)
            } else {
                (other.x, self.x)
            }
        } else {
            if self.y < other.y {
                (self.y, other.y)
            } else {
                (other.y, self.y)
            }
        };
        (range.0..=range.1).map(move |i| {
            if horizontal {
                Self::new(i, self.y)
            } else {
                Self::new(self.x, i)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "498,4 -> 498,6 -> 496,6
503,4 -> 502,4 -> 502,9 -> 494,9";
        assert_eq!(24, simulate_falling_sand(input, true, false));
        assert_eq!(93, simulate_falling_sand(input, true, true));
    }
}
