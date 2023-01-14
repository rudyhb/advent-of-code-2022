use std::str::FromStr;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input5.txt").unwrap();
    let (mut stacks, instructions) = parse_input(&input);
    for instruction in instructions.iter() {
        stacks.move_crate(instruction);
    }

    println!("top of stacks: {}", stacks.top_of_stacks());

    let (mut stacks, instructions) = parse_input(&input);
    stacks.move_at_once = true;
    for instruction in instructions.iter() {
        stacks.move_crate(instruction);
    }

    println!("top of stacks v2: {}", stacks.top_of_stacks());
}

fn parse_input(input: &str) -> (Stacks, Vec<Instruction>) {
    let mut parts = input.split("\n\n");
    (
        parts.next().unwrap().parse().unwrap(),
        parts
            .next()
            .unwrap()
            .lines()
            .map(|line| line.parse().unwrap())
            .collect(),
    )
}

#[derive(Debug, Clone)]
struct Stack(Vec<char>);

impl Stack {
    pub fn new() -> Self {
        Self(Default::default())
    }
    fn push(&mut self, values: &[char]) {
        self.0.extend(values)
    }
    fn pop(&mut self, amount: usize) -> Vec<char> {
        let mut result = Vec::with_capacity(amount);
        for _ in 0..amount {
            result.push(self.0.pop().unwrap());
        }
        result
    }
    pub fn top(&self) -> char {
        *self.0.last().unwrap()
    }
}

#[derive(Debug)]
struct Stacks {
    stacks: Vec<Stack>,
    move_at_once: bool,
}

impl Stacks {
    pub fn top_of_stacks(&self) -> String {
        self.stacks.iter().map(|s| s.top()).collect()
    }
    pub fn move_crate(&mut self, instruction: &Instruction) {
        let mut crates = self.stacks[instruction.from].pop(instruction.amount);
        if self.move_at_once {
            crates.reverse();
        }
        self.stacks[instruction.to].push(&crates);
    }
}

#[derive(Debug)]
struct Instruction {
    amount: usize,
    from: usize,
    to: usize,
}

impl FromStr for Instruction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        parts.next();
        let amount = parts.next().unwrap().parse::<usize>().unwrap();
        parts.next();
        let from = parts.next().unwrap().parse::<usize>().unwrap() - 1;
        parts.next();
        let to = parts.next().unwrap().parse::<usize>().unwrap() - 1;
        Ok(Self { amount, from, to })
    }
}

impl FromStr for Stacks {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let mut line = lines.next().unwrap();
        let n = line.len() + 1;
        if n % 4 != 0 {
            panic!("invalid length of Stacks")
        }
        let mut stacks: Vec<Stack> = vec![Stack::new(); n / 4];

        loop {
            let parts: Vec<char> = line.chars().collect();
            for i in 0..stacks.len() {
                let j = 4 * i + 1;
                if !parts[j].is_whitespace() {
                    stacks[i].0.push(parts[j]);
                }
            }

            line = if let Some(line) = lines.next() {
                if line.chars().filter(|c| !c.is_whitespace()).next() == Some('[') {
                    line
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        for stack in stacks.iter_mut() {
            stack.0.reverse();
        }

        Ok(Self {
            stacks: stacks,
            move_at_once: false,
        })
    }
}

mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = format!(
            "    [D]{}
[N] [C]{}
[Z] [M] [P]
 1   2   3

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2",
            " ".repeat(4),
            " ".repeat(4)
        );

        let (mut stacks, instructions) = parse_input(&input);
        for instruction in instructions.iter() {
            stacks.move_crate(instruction);
        }

        println!("stacks: {:?}", stacks);
        println!("instructions: {:?}", instructions);
        assert_eq!("CMZ", stacks.top_of_stacks());

        let (mut stacks, instructions) = parse_input(&input);
        stacks.move_at_once = true;
        for instruction in instructions.iter() {
            stacks.move_crate(instruction);
        }

        println!("stacks: {:?}", stacks);
        println!("instructions: {:?}", instructions);
        assert_eq!("MCD", stacks.top_of_stacks());
    }
}
