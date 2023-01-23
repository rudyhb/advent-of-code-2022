use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use log::{debug, info};

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input13.txt").unwrap();
    let pairs = parse(&input);
    info!("which pairs go first:\n{}", PairVisualiser(&pairs));
    println!(
        "sum of pair indices in order: {}",
        sum_pairs_indices_in_order(&pairs)
    );
    println!(
        "decoder key: {}",
        generate_decoder_key(
            &pairs
                .into_iter()
                .flat_map(|p| vec![p.0, p.1])
                .collect::<Vec<_>>()
        )
    );
}

fn generate_decoder_key(packets: &[Packet]) -> usize {
    let decoder1: Packet = "[[2]]".parse().unwrap();
    let decoder2: Packet = "[[6]]".parse().unwrap();

    let mut ordered_packets = Vec::with_capacity(packets.len() + 2);
    ordered_packets.extend(packets);
    ordered_packets.push(&decoder1);
    ordered_packets.push(&decoder2);
    ordered_packets.sort();

    debug!(
        "sorted:\n{}",
        ordered_packets
            .iter()
            .map(|p| format!("{}", p))
            .collect::<Vec<String>>()
            .join("\n")
    );

    let get_key =
        |decoder: &Packet| ordered_packets.iter().position(|&p| p == decoder).unwrap() + 1;

    get_key(&decoder1) * get_key(&decoder2)
}

fn sum_pairs_indices_in_order(pairs: &[(Packet, Packet)]) -> usize {
    pairs
        .iter()
        .enumerate()
        .filter(|(_, (left, right))| left <= right)
        .map(|(i, _)| i + 1)
        .sum()
}

struct PairVisualiser<'a>(&'a [(Packet, Packet)]);

impl<'a> Display for PairVisualiser<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|pair| {
                    let less = pair.0 <= pair.1;
                    format!(
                        "{}{}\n{}{}",
                        if less { "-->" } else { "   " },
                        pair.0,
                        if !less { "-->" } else { "   " },
                        pair.1
                    )
                })
                .collect::<Vec<String>>()
                .join("\n\n")
        )
    }
}

fn parse(s: &str) -> Vec<(Packet, Packet)> {
    s.split("\n\n")
        .map(|pairs| {
            let mut p = pairs.lines();
            (
                p.next().unwrap().parse().unwrap(),
                p.next().unwrap().parse().unwrap(),
            )
        })
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
struct Packet(Vec<Element>);

impl Packet {
    pub fn new(elements: Vec<Element>) -> Self {
        Self(elements)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Element {
    Packet(Packet),
    Number(u64),
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        for i in 0..self.0.len() {
            if other.0.len() <= i {
                debug!(
                    "{} > {} -- more elements ({} <= {})",
                    self,
                    other,
                    other.0.len(),
                    i
                );
                return Some(Ordering::Greater);
            }
            let cmp = self.0[i].partial_cmp(&other.0[i]).unwrap();
            if cmp != Ordering::Equal {
                debug!(
                    "{} {:?} {} because {} {:?} {}",
                    self, cmp, other, self.0[i], cmp, other.0[i]
                );
                return Some(cmp);
            }
        }
        if other.0.len() > self.0.len() {
            debug!("{} < {} -- less elements", self, other);
            return Some(Ordering::Less);
        }
        Some(Ordering::Equal)
    }
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for Element {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Element::Packet(p_self) => match other {
                Element::Packet(p_other) => p_self.partial_cmp(&p_other),
                Element::Number(n_other) => {
                    let p_other = Packet::new(vec![Element::Number(*n_other)]);
                    p_self.partial_cmp(&p_other)
                }
            },
            Element::Number(n_self) => match other {
                Element::Packet(p_other) => {
                    let p_self = Packet::new(vec![Element::Number(*n_self)]);
                    p_self.partial_cmp(&p_other)
                }
                Element::Number(n_other) => n_self.partial_cmp(n_other),
            },
        }
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.0
                .iter()
                .map(|element| {
                    match element {
                        Element::Packet(p) => format!("{}", p),
                        Element::Number(n) => format!("{}", n),
                    }
                })
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Element::Packet(p) => write!(f, "{}", p),
            Element::Number(n) => write!(f, "{}", n),
        }
    }
}

impl FromStr for Packet {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        assert_eq!(s.bytes().next().unwrap(), '[' as u8, "missing [");
        assert_eq!(s.bytes().last().unwrap(), ']' as u8, "missing ]");
        let input = &s[1..s.len() - 1];

        if input.len() == 0 {
            return Ok(Self(vec![]));
        }

        let mut commas = vec![0];
        let mut pos = 0;
        let mut open_brackets = 0;
        let mut last = 0;
        for c in input.chars() {
            last = pos;
            match c {
                ',' => {
                    if open_brackets == 0 {
                        commas.push(pos);
                    }
                }
                '[' => {
                    open_brackets += 1;
                }
                ']' => {
                    open_brackets -= 1;
                }
                _ => {}
            }
            pos += c.len_utf8();
        }
        commas.push(last + ','.len_utf8());

        Ok(Self(
            commas
                .windows(2)
                .map(|range| input[range[0]..range[1]].trim_matches(','))
                .map(|element| {
                    if element.chars().next().unwrap() == '[' {
                        debug!("parsing list: {}", element);
                        Element::Packet(element.parse().unwrap())
                    } else {
                        debug!("parsing number: {}", element);
                        Element::Number(element.parse().unwrap())
                    }
                })
                .collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]";

        let pairs = parse(input);
        assert_eq!(
            input,
            pairs
                .iter()
                .map(|pair| { format!("{}\n{}", pair.0, pair.1) })
                .collect::<Vec<String>>()
                .join("\n\n")
        );
        assert_eq!(13, sum_pairs_indices_in_order(&pairs));
        assert_eq!(
            140,
            generate_decoder_key(
                &pairs
                    .into_iter()
                    .flat_map(|p| vec![p.0, p.1])
                    .collect::<Vec<_>>()
            )
        );
    }
}
