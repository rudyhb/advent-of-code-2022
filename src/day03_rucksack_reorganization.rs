use std::collections::HashSet;
use std::fmt::Debug;
use std::iter::Chain;
use std::slice::Iter;
use std::str::FromStr;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input3.txt").unwrap();
    let rucksacks: Vec<Rucksack> = input.lines().map(|l| l.parse().unwrap()).collect();
    println!(
        "sum of priorities in rucksacks: {}",
        rucksacks
            .iter()
            .map(|r| r.get_letter_in_both().0 as u64)
            .sum::<u64>()
    );

    let groups = rucksacks_to_groups(&rucksacks);
    println!(
        "sum of priorities in group: {}",
        groups
            .iter()
            .map(|g| g.get_letter_in_all().0 as u64)
            .sum::<u64>()
    );
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct Letter(u8);

impl From<char> for Letter {
    fn from(value: char) -> Self {
        let value = value as u8;
        Self(if value <= 90 {
            27 + value - 65
        } else {
            1 + value - 97
        })
    }
}

impl From<Letter> for char {
    fn from(value: Letter) -> Self {
        if value.0 >= 27 {
            (value.0 + 65 - 27) as char
        } else {
            (value.0 + 97 - 1) as char
        }
    }
}

#[derive(Clone, Debug)]
struct Rucksack {
    compartments: [Vec<Letter>; 2],
}

impl Rucksack {
    fn get_letter_in_both(&self) -> Letter {
        let set: HashSet<&Letter> = self.compartments[0].iter().collect();
        self.compartments[1]
            .iter()
            .filter(|l| set.contains(l))
            .copied()
            .next()
            .expect("no letter found in both compartments")
    }
    fn get_letters(&self) -> Chain<Iter<'_, Letter>, Iter<'_, Letter>> {
        self.compartments[0]
            .iter()
            .chain(self.compartments[1].iter())
    }
}

struct Group([Rucksack; 3]);

impl Group {
    fn get_letter_in_all(&self) -> Letter {
        let set1: HashSet<&Letter> = self.0[0].get_letters().collect();
        let set2: HashSet<&Letter> = self.0[1]
            .get_letters()
            .filter(|l| set1.contains(l))
            .collect();
        self.0[2]
            .get_letters()
            .filter(|l| set2.contains(l))
            .copied()
            .next()
            .expect("no letter found in all 3 rucksacks")
    }
}

fn rucksacks_to_groups(sacks: &Vec<Rucksack>) -> Vec<Group> {
    sacks
        .chunks(3)
        .map(|sacks| {
            Group(
                sacks
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            )
        })
        .collect()
}

impl FromStr for Rucksack {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let letters: Vec<Letter> = s.chars().map(|c| Letter::from(c)).collect();
        assert_eq!(letters.len() % 2, 0, "invalid rucksack");
        let n = letters.len() / 2;
        Ok(Self {
            compartments: [
                letters.iter().copied().take(n).collect(),
                letters.iter().copied().skip(n).collect(),
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test0() {
        fn test(c: char) {
            assert_eq!(c, char::from(Letter::from(c)));
        }
        test('a');
        test('w');
        test('A');
        test('W');
    }

    #[test]
    fn test1() {
        let input = "vJrwpWtwJgWrhcsFMMfFFhFp
jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
PmmdzqPrVvPwwTWBwg
wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
ttgJtRGJQctTZtZT
CrZsJsPPZsGzwwsLwLmpwMDw";
        let rucksacks: Vec<Rucksack> = input.lines().map(|l| l.parse().unwrap()).collect();
        assert_eq!(
            157,
            rucksacks
                .iter()
                .map(|r| r.get_letter_in_both().0 as u64)
                .sum::<u64>()
        );
        let groups = rucksacks_to_groups(&rucksacks);
        assert_eq!(
            70,
            groups
                .iter()
                .map(|g| g.get_letter_in_all().0 as u64)
                .sum::<u64>()
        );
    }
}
