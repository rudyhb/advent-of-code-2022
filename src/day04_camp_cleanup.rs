use std::str::FromStr;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input4.txt").unwrap();
    let pairs = parse_assignment_pairs(&input);
    println!(
        "pairs where range fully contains the other: {}",
        pairs
            .iter()
            .filter(|pair| pair[0].fully_contains(&pair[1]) || pair[1].fully_contains(&pair[0]))
            .count()
    );
    println!(
        "pairs that overlap: {}",
        pairs
            .iter()
            .filter(|pair| pair[0].overlaps(&pair[1]))
            .count()
    );
}

fn parse_assignment_pairs(s: &str) -> Vec<[Range; 2]> {
    s.lines()
        .map(|line| {
            let mut parts = line.split(',');
            [
                parts.next().unwrap().parse().unwrap(),
                parts.next().unwrap().parse().unwrap(),
            ]
        })
        .collect()
}

struct Range {
    from: u64,
    to: u64,
}

impl Range {
    pub fn fully_contains(&self, other: &Self) -> bool {
        self.from <= other.from && self.to >= other.to
    }
    pub fn overlaps(&self, other: &Self) -> bool {
        self.from <= other.to && self.to >= other.from
    }
}

impl FromStr for Range {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.trim().split('-');
        let result = Self {
            from: parts.next().unwrap().parse().unwrap(),
            to: parts.next().unwrap().parse().unwrap(),
        };
        if parts.next().is_some() {
            panic!("cannot parse range '{}'", s)
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "2-4,6-8
2-3,4-5
5-7,7-9
2-8,3-7
6-6,4-6
2-6,4-8";
        let pairs = parse_assignment_pairs(input);
        assert_eq!(
            2,
            pairs
                .iter()
                .filter(|pair| pair[0].fully_contains(&pair[1]) || pair[1].fully_contains(&pair[0]))
                .count()
        );
        assert_eq!(
            4,
            pairs
                .iter()
                .filter(|pair| pair[0].overlaps(&pair[1]))
                .count()
        );
    }
}
