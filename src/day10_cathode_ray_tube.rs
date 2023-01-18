use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input10.txt").unwrap();
    let mut cpu = Cpu::new(InstructionSequence::new(
        input.lines().map(|l| l.parse().unwrap()).collect(),
    ));
    println!(
        "sum signal strengths: {}",
        cpu.clone()
            .sum_signal_strengths(vec![20, 60, 100, 140, 180, 220])
    );

    let mut crt = Crt::new(&mut cpu);
    crt.run();
    println!("result:\n{}", crt);
}

struct Crt<'a> {
    cpu: &'a mut Cpu,
    screen: [[bool; 40]; 6],
}

impl<'a> Crt<'a> {
    pub fn new(cpu: &'a mut Cpu) -> Self {
        Self {
            cpu,
            screen: [[false; 40]; 6],
        }
    }
    pub fn run(&mut self) {
        for (i, pixel) in self
            .screen
            .iter_mut()
            .flat_map(|pixel_row| pixel_row.iter_mut().enumerate())
        {
            *pixel = self
                .cpu
                .cycles()
                .next()
                .expect("ran out of instructions")
                .sprite_seen_at_pixel(i);
        }
    }
}

impl<'a> Display for Crt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.screen
                .iter()
                .map(|line| line
                    .iter()
                    .map(|pixel| if *pixel { '#' } else { '.' })
                    .collect::<String>())
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[derive(Clone)]
struct Cpu {
    state: State,
    instructions: InstructionSequence,
}

#[derive(Clone, Debug)]
struct State {
    pub register: i64,
    pub cycle: u32,
}

impl State {
    pub fn signal_strength(&self) -> i64 {
        (self.cycle as i64) * self.register
    }
    pub fn sprite_seen_at_pixel(&self, pixel_number: usize) -> bool {
        let n = pixel_number as i64;
        n >= self.register - 1 && n <= self.register + 1
    }
}

impl Cpu {
    pub fn new(instructions: InstructionSequence) -> Self {
        Self {
            state: State {
                cycle: 0,
                register: 1,
            },
            instructions,
        }
    }
    pub fn sum_signal_strengths(&mut self, cycles: Vec<usize>) -> i64 {
        self.cycles()
            .skip(cycles[0] - 1)
            .next()
            .unwrap()
            .signal_strength()
            + cycles
                .windows(2)
                .map(|w| {
                    self.cycles()
                        .skip(w[1] - w[0] - 1)
                        .next()
                        .unwrap()
                        .signal_strength()
                })
                .sum::<i64>()
    }
    pub fn cycles(&mut self) -> CpuCycles {
        CpuCycles { cpu: self }
    }
}

struct CpuCycles<'a> {
    cpu: &'a mut Cpu,
}

impl<'a> Iterator for CpuCycles<'a> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        self.cpu.state.cycle += 1;
        let state = self.cpu.state.clone();
        if let Some(instruction) = self.cpu.instructions.next() {
            match instruction {
                None | Some(Instruction::NoOperation) => {}
                Some(Instruction::AddX(value)) => {
                    self.cpu.state.register += value;
                }
            }
        } else {
            return None;
        }
        Some(state)
    }
}

#[derive(Clone)]
struct InstructionSequence {
    instructions: Vec<Instruction>,
    current: usize,
    in_progress: usize,
}

impl InstructionSequence {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self {
            instructions,
            current: 0,
            in_progress: 0,
        }
    }
}

impl Iterator for InstructionSequence {
    type Item = Option<Instruction>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.instructions.len() {
            return None;
        }
        if let Instruction::AddX(_) = &self.instructions[self.current] {
            if self.in_progress < 1 {
                self.in_progress += 1;
                return Some(None);
            }
        }
        let result = Some(self.instructions[self.current].clone());
        self.in_progress = 0;
        self.current += 1;
        Some(result)
    }
}

#[derive(Clone)]
enum Instruction {
    NoOperation,
    AddX(i64),
}

impl FromStr for Instruction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = s.split_whitespace();
        Ok(match words.next().unwrap() {
            "noop" => Self::NoOperation,
            "addx" => Self::AddX(words.next().unwrap().parse().unwrap()),
            other => panic!("invalid instruction '{}'", other),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "addx 15\naddx -11\naddx 6\naddx -3\naddx 5\naddx -1\naddx -8\naddx 13\naddx 4\nnoop\naddx -1\naddx 5\naddx -1\naddx 5\naddx -1\naddx 5\naddx -1\naddx 5\naddx -1\naddx -35\naddx 1\naddx 24\naddx -19\naddx 1\naddx 16\naddx -11\nnoop\nnoop\naddx 21\naddx -15\nnoop\nnoop\naddx -3\naddx 9\naddx 1\naddx -3\naddx 8\naddx 1\naddx 5\nnoop\nnoop\nnoop\nnoop\nnoop\naddx -36\nnoop\naddx 1\naddx 7\nnoop\nnoop\nnoop\naddx 2\naddx 6\nnoop\nnoop\nnoop\nnoop\nnoop\naddx 1\nnoop\nnoop\naddx 7\naddx 1\nnoop\naddx -13\naddx 13\naddx 7\nnoop\naddx 1\naddx -33\nnoop\nnoop\nnoop\naddx 2\nnoop\nnoop\nnoop\naddx 8\nnoop\naddx -1\naddx 2\naddx 1\nnoop\naddx 17\naddx -9\naddx 1\naddx 1\naddx -3\naddx 11\nnoop\nnoop\naddx 1\nnoop\naddx 1\nnoop\nnoop\naddx -13\naddx -19\naddx 1\naddx 3\naddx 26\naddx -30\naddx 12\naddx -1\naddx 3\naddx 1\nnoop\nnoop\nnoop\naddx -9\naddx 18\naddx 1\naddx 2\nnoop\nnoop\naddx 9\nnoop\nnoop\nnoop\naddx -1\naddx 2\naddx -37\naddx 1\naddx 3\nnoop\naddx 15\naddx -21\naddx 22\naddx -6\naddx 1\nnoop\naddx 2\naddx 1\nnoop\naddx -10\nnoop\nnoop\naddx 20\naddx 1\naddx 2\naddx 2\naddx -6\naddx -11\nnoop\nnoop\nnoop";
        let mut cpu = Cpu::new(InstructionSequence::new(
            input.lines().map(|l| l.parse().unwrap()).collect(),
        ));
        assert_eq!(
            13140,
            cpu.clone()
                .sum_signal_strengths(vec![20, 60, 100, 140, 180, 220])
        );
        let mut crt = Crt::new(&mut cpu);
        crt.run();
        assert_eq!(
            "\
##..##..##..##..##..##..##..##..##..##..
###...###...###...###...###...###...###.
####....####....####....####....####....
#####.....#####.....#####.....#####.....
######......######......######......####
#######.......#######.......#######.....",
            format!("{}", crt)
        );
    }
}
