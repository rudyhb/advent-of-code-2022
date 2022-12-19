use std::str::FromStr;

use log::info;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input1.txt").unwrap();

    println!("most calories: {}", solve(&input));
    println!("sum top 3 calories: {}", solve_2(&input));
}

fn get_elves(input: &str) -> Vec<Elf> {
    input
        .split("\n\n")
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect()
}

fn solve(input: &str) -> u64 {
    let elves = get_elves(input);
    info!("elves: {:?}", elves);
    elves
        .iter()
        .map(|elf| elf.food.iter().map(|f| f.0).sum())
        .max()
        .unwrap()
}

fn solve_2(input: &str) -> u64 {
    let elves = get_elves(input);
    let mut sorted: Vec<_> = elves
        .iter()
        .map(|elf| elf.food.iter().map(|f| f.0).sum())
        .collect();
    sorted.sort();
    sorted.iter().rev().take(3).sum()
}

#[derive(Debug)]
struct Elf {
    food: Vec<Food>,
}

impl FromStr for Elf {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            food: s
                .split_whitespace()
                .filter(|s| !s.is_empty())
                .map(|val| Food(val.parse().expect("parse error")))
                .collect(),
        })
    }
}

#[derive(Debug)]
struct Food(u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        assert_eq!(
            solve(
                "1000
2000
3000

4000

5000
6000

7000
8000
9000

10000"
            ),
            24_000
        );
        assert_eq!(
            solve_2(
                "1000
2000
3000

4000

5000
6000

7000
8000
9000

10000"
            ),
            45_000
        );
    }
}
