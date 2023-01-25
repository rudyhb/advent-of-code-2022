use std::fmt::{Display, Formatter};
use std::str::FromStr;

use log::debug;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input15.txt").unwrap();
    let y = 2_000_000;
    println!(
        "positions with no beacons at y= {}: {}",
        y,
        count_positions_without_beacon(&input, y)
    );
    println!(
        "frequency of the only empty cell: {}",
        find_frequency_only_empty_cell(&input, 4_000_000)
    );
}

fn count_positions_without_beacon(input: &str, row: i64) -> usize {
    let grid: Grid = input.parse().unwrap();
    let mut ranges: RangesInclusive = grid
        .sensors
        .iter()
        .filter_map(|sensor| sensor.intersection(row))
        .collect::<Vec<_>>()
        .into();
    for beacon in grid
        .sensors
        .iter()
        .map(|s| s.closest_beacon.clone())
        .filter(|b| b.y == row)
        .map(|b| b.x)
    {
        ranges.add_exception(beacon);
    }
    debug!("y={}: {}", row, ranges);
    debug!("{:?}", ranges);
    ranges.count_ints()
}

fn find_frequency_only_empty_cell(input: &str, max: i64) -> i64 {
    let full_row = RangeInclusive::new(0, max);
    let grid: Grid = input.parse().unwrap();
    let rows_with_empty_cells = (0..max)
        .map(|y| {
            let mut ranges: RangesInclusive = grid
                .sensors
                .iter()
                .filter_map(|s| s.intersection(y))
                .collect::<Vec<_>>()
                .into();
            ranges.remove_outliers(0, max);
            (y, ranges)
        })
        .filter(|(_, ranges)| !(ranges.0.len() == 1 && ranges.0[0] == full_row))
        .collect::<Vec<_>>();
    assert_eq!(1, rows_with_empty_cells.len());
    assert_eq!(2, rows_with_empty_cells[0].1 .0.len());
    let empty_cell = Coord::new(
        rows_with_empty_cells[0].1 .0[0].to + 1,
        rows_with_empty_cells[0].0,
    );
    println!("empty cell: {:?}", empty_cell);
    empty_cell.tuning_frequency()
}

struct Grid {
    sensors: Vec<Sensor>,
}

struct Sensor {
    position: Coord,
    closest_beacon: Coord,
}

impl Sensor {
    pub fn intersection(&self, y: i64) -> Option<RangeInclusive> {
        let distance = self.position.manhattan_distance(&self.closest_beacon);
        let dy = (self.position.y - y).abs();
        if dy <= distance {
            let dx = distance - dy;
            Some(RangeInclusive::new(
                self.position.x - dx,
                self.position.x + dx,
            ))
        } else {
            None
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
struct Coord {
    x: i64,
    y: i64,
}

impl Coord {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
    pub fn manhattan_distance(&self, other: &Self) -> i64 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
    pub fn tuning_frequency(&self) -> i64 {
        4_000_000 * self.x + self.y
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
struct RangeInclusive {
    from: i64,
    to: i64,
}

impl RangeInclusive {
    pub fn new(from: i64, to: i64) -> Self {
        Self { from, to }
    }
    pub fn includes(&self, value: i64) -> bool {
        self.from <= value && self.to >= value
    }
    pub fn len(&self) -> usize {
        (self.to - self.from + 1) as usize
    }
    pub fn try_split_at(&self, value: i64) -> Option<Vec<Self>> {
        assert!(self.includes(value));
        if self.from == self.to {
            return None;
        }
        Some(if self.from == value {
            vec![Self {
                from: self.from + 1,
                to: self.to,
            }]
        } else if self.to == value {
            vec![Self {
                from: self.from,
                to: self.to - 1,
            }]
        } else {
            vec![
                Self {
                    from: self.from,
                    to: value - 1,
                },
                Self {
                    from: value + 1,
                    to: self.to,
                },
            ]
        })
    }
    pub fn try_combine(&self, other: &Self) -> Option<Self> {
        let (left, right) = if self <= other {
            (self, other)
        } else {
            (other, self)
        };
        if (left.to + 1) >= right.from {
            Some(Self {
                from: left.from,
                to: left.to.max(right.to),
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct RangesInclusive(Vec<RangeInclusive>);

impl RangesInclusive {
    pub fn count_ints(&self) -> usize {
        self.0.iter().map(|range| range.len()).sum()
    }
    pub fn add_exception(&mut self, exception: i64) {
        if let Some(i) = self.0.iter().position(|r| r.includes(exception)) {
            if let Some(new_ranges) = self.0[i].try_split_at(exception) {
                let mut new_ranges = new_ranges.into_iter();
                self.0[i] = new_ranges.next().unwrap();
                for (j, range) in new_ranges.enumerate() {
                    self.0.insert(i + j + 1, range);
                }
            } else {
                self.0.remove(i);
            }
        }
    }
    pub fn remove_outliers(&mut self, min: i64, max: i64) {
        let mut result = Vec::new();
        for range in std::mem::take(&mut self.0)
            .into_iter()
            .filter(|range| range.from <= max && range.to >= min)
        {
            result.push(RangeInclusive {
                from: range.from.max(min),
                to: range.to.min(max),
            })
        }
        self.0 = result;
    }
}

impl Display for RangesInclusive {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let from = self.0[0].from;
        let to = self.0[self.0.len() - 1].to;
        writeln!(f, "range {} -> {}:", from, to)?;
        let mut values = self.0.iter().flat_map(|range| range.from..=range.to);
        let mut value = values.next();
        write!(
            f,
            "{}",
            (from..=to)
                .map(|i| {
                    if Some(i) == value {
                        value = values.next();
                        '#'
                    } else {
                        '.'
                    }
                })
                .collect::<String>()
        )
    }
}

impl From<Vec<RangeInclusive>> for RangesInclusive {
    fn from(mut values: Vec<RangeInclusive>) -> Self {
        if values.len() < 2 {
            return Self(values);
        }
        values.sort_by(|a, b| b.cmp(a));
        let mut result = Vec::new();
        let mut left = values.pop().unwrap();
        let mut right_opt = values.pop();
        while let Some(right) = right_opt {
            if let Some(combined) = left.try_combine(&right) {
                left = combined;
            } else {
                result.push(left);
                left = right;
            }
            right_opt = values.pop();
        }
        result.push(left);

        Self(result)
    }
}

impl FromStr for Grid {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_coord(s: &str) -> Coord {
            let mut parts = s.split_whitespace().rev();
            fn parse(s: &str) -> i64 {
                s.split('=')
                    .skip(1)
                    .next()
                    .unwrap()
                    .trim_matches(',')
                    .parse()
                    .unwrap()
            }
            Coord {
                y: parse(parts.next().unwrap()),
                x: parse(parts.next().unwrap()),
            }
        }
        Ok(Self {
            sensors: s
                .lines()
                .map(|line| {
                    let mut parts = line.split(':');
                    Sensor {
                        position: parse_coord(parts.next().unwrap()),
                        closest_beacon: parse_coord(parts.next().unwrap()),
                    }
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "Sensor at x=2, y=18: closest beacon is at x=-2, y=15
Sensor at x=9, y=16: closest beacon is at x=10, y=16
Sensor at x=13, y=2: closest beacon is at x=15, y=3
Sensor at x=12, y=14: closest beacon is at x=10, y=16
Sensor at x=10, y=20: closest beacon is at x=10, y=16
Sensor at x=14, y=17: closest beacon is at x=10, y=16
Sensor at x=8, y=7: closest beacon is at x=2, y=10
Sensor at x=2, y=0: closest beacon is at x=2, y=10
Sensor at x=0, y=11: closest beacon is at x=2, y=10
Sensor at x=20, y=14: closest beacon is at x=25, y=17
Sensor at x=17, y=20: closest beacon is at x=21, y=22
Sensor at x=16, y=7: closest beacon is at x=15, y=3
Sensor at x=14, y=3: closest beacon is at x=15, y=3
Sensor at x=20, y=1: closest beacon is at x=15, y=3";

        assert_eq!(26, count_positions_without_beacon(input, 10));
        assert_eq!(56000011, find_frequency_only_empty_cell(&input, 20));
    }
}
