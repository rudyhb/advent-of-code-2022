use std::str::FromStr;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input2.txt").unwrap();

    println!("my total score: {}", play(&input));
    println!("my total score v2: {}", play_v2(&input));
}

fn play(input: &str) -> u64 {
    input
        .lines()
        .map(|line| line.parse::<Round>().unwrap().get_my_score())
        .sum()
}

fn play_v2(input: &str) -> u64 {
    input
        .lines()
        .map(|line| {
            let mut round: Round = line.parse().unwrap();
            round.fix_encryption();
            round.get_my_score()
        })
        .sum()
}

#[derive(Copy, Clone)]
enum Shape {
    Rock,
    Paper,
    Scissors,
}

struct Round {
    player: Shape,
    opponent: Shape,
}

enum GameResult {
    Win,
    Lose,
    Tie,
}

impl Shape {
    pub fn result_against(&self, other: &Self) -> GameResult {
        match self {
            Shape::Rock => match other {
                Shape::Rock => GameResult::Tie,
                Shape::Paper => GameResult::Lose,
                Shape::Scissors => GameResult::Win,
            },
            Shape::Paper => match other {
                Shape::Rock => GameResult::Win,
                Shape::Paper => GameResult::Tie,
                Shape::Scissors => GameResult::Lose,
            },
            Shape::Scissors => match other {
                Shape::Rock => GameResult::Lose,
                Shape::Paper => GameResult::Win,
                Shape::Scissors => GameResult::Tie,
            },
        }
    }
}

impl Round {
    pub fn get_my_score(&self) -> u64 {
        return match self.player {
            Shape::Rock => 1,
            Shape::Paper => 2,
            Shape::Scissors => 3,
        } + match self.player.result_against(&self.opponent) {
            GameResult::Win => 6,
            GameResult::Lose => 0,
            GameResult::Tie => 3,
        };
    }
    pub fn fix_encryption(&mut self) {
        self.player = match self.player {
            Shape::Rock => {
                // X: should lose
                match self.opponent {
                    Shape::Rock => Shape::Scissors,
                    Shape::Paper => Shape::Rock,
                    Shape::Scissors => Shape::Paper,
                }
            }
            Shape::Paper => {
                // Y: should tie
                self.opponent
            }
            Shape::Scissors => {
                // Z: should win
                match self.opponent {
                    Shape::Rock => Shape::Paper,
                    Shape::Paper => Shape::Scissors,
                    Shape::Scissors => Shape::Rock,
                }
            }
        };
    }
}

impl FromStr for Round {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let opponent: Shape = match parts.next().expect("line too short") {
            "A" => Shape::Rock,
            "B" => Shape::Paper,
            "C" => Shape::Scissors,
            other => panic!("invalid opponent option '{}'", other),
        };
        let player: Shape = match parts.next().expect("line too short") {
            "X" => Shape::Rock,
            "Y" => Shape::Paper,
            "Z" => Shape::Scissors,
            other => panic!("invalid player option '{}'", other),
        };
        Ok(Self { player, opponent })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        assert_eq!(
            play(
                "A Y
B X
C Z"
            ),
            15
        )
    }
    #[test]
    fn test2() {
        assert_eq!(
            play_v2(
                "A Y
B X
C Z"
            ),
            12
        )
    }
}
