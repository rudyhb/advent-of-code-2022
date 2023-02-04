use std::fmt::{Display, Formatter};

use log::debug;

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input20.txt").unwrap();
    let mut file = File::new(input.lines().map(|l| l.parse().unwrap()).collect());
    file.mix(1);
    debug!("after mix: {}", file);
    println!("grove coordinate: {}", file.grove_coordinate());

    let mut file = File::new(input.lines().map(|l| l.parse().unwrap()).collect());
    file.use_decrypt_key();
    file.mix(10);
    debug!("after mix: {}", file);
    println!(
        "grove coordinate after decrypt: {}",
        file.grove_coordinate()
    );
}

struct File {
    current: Vec<usize>,
    values: Vec<i64>,
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.current)
    }
}

impl File {
    pub fn use_decrypt_key(&mut self) {
        const KEY: i64 = 811589153;
        self.values.iter_mut().for_each(|v| *v *= KEY);
    }
    pub fn grove_coordinate(&self) -> i64 {
        let zero = self.get_index(0);
        let n1 = self.nth(zero + 1000);
        let n2 = self.nth(zero + 2000);
        let n3 = self.nth(zero + 3000);
        println!("coordinates: {}, {}, {}", n1, n2, n3);
        n1 + n2 + n3
    }
    pub fn new(values: Vec<i64>) -> Self {
        Self {
            current: (0..values.len()).collect(),
            values,
        }
    }
    pub fn mix(&mut self, times: usize) {
        debug!("Initial arrangement:\n{}", self);
        for _ in 0..times {
            for i in 0..self.current.len() {
                let current = self.current.iter().position(|&p| p == i).unwrap();
                self.current.remove(current);
                self.current.insert(
                    (current
                        + ((self.values[i] % self.current.len() as i64) + self.current.len() as i64)
                            as usize)
                        % self.current.len(),
                    i,
                );
            }
        }
    }
    pub fn get_index(&self, value: i64) -> usize {
        let original_index = self.values.iter().position(|&p| p == value).unwrap();
        self.current
            .iter()
            .position(|&v| v == original_index)
            .unwrap()
    }
    pub fn nth(&self, index: usize) -> i64 {
        self.values
            [self.current[((index % self.current.len()) + self.current.len()) % self.current.len()]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "1\n2\n-3\n3\n-2\n0\n4";
        let mut file = File::new(input.lines().map(|l| l.parse().unwrap()).collect());
        file.mix(1);
        println!("after mix: {}", file);
        assert_eq!(3, file.grove_coordinate());

        let mut file = File::new(input.lines().map(|l| l.parse().unwrap()).collect());
        file.use_decrypt_key();
        file.mix(10);
        println!("after mix: {}", file);
        assert_eq!(1623178306, file.grove_coordinate());
    }
}
