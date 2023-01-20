use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input11.txt").unwrap();
    let mut monkeys: Vec<Monkey> = input
        .split("\n\n")
        .map(|line| line.parse().unwrap())
        .collect();
    println!(
        "level of monkey business: {}",
        get_monkey_business_level(&mut monkeys, 20)
    );

    let mut monkeys: Vec<Monkey> = input
        .split("\n\n")
        .map(|line| line.parse().unwrap())
        .collect();
    monkeys
        .iter_mut()
        .for_each(|m| m.reduce_worry_level = false);
    assign_lcd(&mut monkeys);
    println!(
        "without reducing worry level: {}",
        get_monkey_business_level(&mut monkeys, 10_000)
    );
}

fn assign_lcd(monkeys: &mut [Monkey]) {
    let lcd = monkeys.iter().map(|m| m.test.divisible_by).product::<u64>();
    monkeys
        .iter_mut()
        .for_each(|m| m.least_common_denominator = lcd);
}

fn get_monkey_business_level(monkeys: &mut [Monkey], rounds: u32) -> u64 {
    for _round in 0..rounds {
        for monkey_id in 0..monkeys.len() {
            let next = monkeys[monkey_id].round();
            for (next, mut items) in next {
                items.iter_mut().for_each(|i| i.thrown(next));
                monkeys[next].catch(items.into_iter());
            }
        }
        // if rounds > 20 {
        //     for item in monkeys.iter().flat_map(|m| m.items.iter()) {
        //         println!("item {}: {:?}", item.id, item.history);
        //     }
        // }
    }
    let mut times: Vec<_> = monkeys
        .iter()
        .map(|monkey| monkey.items_inspected)
        .collect();
    times.sort();
    let mut times = times.into_iter().rev();
    times.next().unwrap() * times.next().unwrap()
}

struct Monkey {
    items: Vec<Item>,
    operation: Operation,
    test: MonkeyTest,
    next: NextMonkey,
    items_inspected: u64,
    reduce_worry_level: bool,
    least_common_denominator: u64,
}

impl Monkey {
    pub fn round(&mut self) -> HashMap<usize, Vec<Item>> {
        let mut result: HashMap<usize, Vec<Item>> = HashMap::new();
        for mut item in std::mem::take(&mut self.items) {
            let next = self.inspect(&mut item);
            result.entry(next).or_default().push(item);
        }

        result
    }
    pub fn catch(&mut self, items: impl Iterator<Item = Item>) {
        self.items.extend(items)
    }
    fn inspect(&mut self, item: &mut Item) -> usize {
        self.items_inspected += 1;
        self.operation.operate(&mut item.value);
        if self.reduce_worry_level {
            item.value /= 3;
        }
        item.value = item.value % self.least_common_denominator;
        if self.test.test(item.value) {
            self.next.if_true
        } else {
            self.next.if_false
        }
    }
}

struct NextMonkey {
    if_true: usize,
    if_false: usize,
}

#[allow(unused)]
struct Item {
    value: u64,
    history: Vec<usize>,
    id: u64,
}

static ID: AtomicU64 = AtomicU64::new(0);

fn get_id() -> u64 {
    ID.fetch_add(1, Ordering::SeqCst)
}

impl Item {
    pub fn new(value: u64) -> Self {
        Self {
            value,
            history: Default::default(),
            id: get_id(),
        }
    }
    pub fn thrown(&mut self, to: usize) {
        self.history.push(to)
    }
}

enum Operation {
    Add(u64),
    Multiply(u64),
    Square,
}

impl Operation {
    pub fn operate(&self, value: &mut u64) {
        match self {
            Operation::Add(rhs) => *value += *rhs,
            Operation::Multiply(rhs) => *value *= *rhs,
            Operation::Square => *value *= *value,
        }
    }
}

struct MonkeyTest {
    divisible_by: u64,
}

impl MonkeyTest {
    pub fn test(&self, value: u64) -> bool {
        value % self.divisible_by == 0
    }
}

impl FromStr for Operation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s
            .split('=')
            .skip(1)
            .next()
            .unwrap()
            .trim()
            .split_whitespace();
        assert_eq!("old", parts.next().unwrap());
        let operation = parts.next().unwrap();
        let rhs = parts.next().unwrap();
        if rhs == "old" {
            assert_eq!("*", operation);
            Ok(Self::Square)
        } else {
            let number: u64 = rhs.parse().unwrap();
            Ok(match operation {
                "+" => Self::Add(number),
                "*" => Self::Multiply(number),
                other => panic!("invalid operation '{}'", other),
            })
        }
    }
}

impl FromStr for Monkey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        lines.next();
        let items: Vec<_> = lines
            .next()
            .unwrap()
            .split(':')
            .skip(1)
            .next()
            .unwrap()
            .split(',')
            .map(|n| Item::new(n.trim().parse().unwrap()))
            .collect();
        let operation: Operation = lines
            .next()
            .unwrap()
            .split(':')
            .skip(1)
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let test = lines
            .next()
            .unwrap()
            .split_whitespace()
            .last()
            .unwrap()
            .parse()
            .unwrap();
        let if_true = lines
            .next()
            .unwrap()
            .split_whitespace()
            .last()
            .unwrap()
            .parse()
            .unwrap();
        let if_false = lines
            .next()
            .unwrap()
            .split_whitespace()
            .last()
            .unwrap()
            .parse()
            .unwrap();
        Ok(Self {
            items,
            operation,
            test: MonkeyTest { divisible_by: test },
            next: NextMonkey { if_true, if_false },
            items_inspected: 0,
            reduce_worry_level: true,
            least_common_denominator: u64::MAX,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0

Monkey 2:
  Starting items: 79, 60, 97
  Operation: new = old * old
  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3

Monkey 3:
  Starting items: 74
  Operation: new = old + 3
  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1";
        let mut monkeys: Vec<Monkey> = input
            .split("\n\n")
            .map(|line| line.parse().unwrap())
            .collect();
        assert_eq!(10_605, get_monkey_business_level(&mut monkeys, 20));

        let mut monkeys: Vec<Monkey> = input
            .split("\n\n")
            .map(|line| line.parse().unwrap())
            .collect();
        monkeys
            .iter_mut()
            .for_each(|m| m.reduce_worry_level = false);
        assign_lcd(&mut monkeys);
        assert_eq!(
            2_713_310_158,
            get_monkey_business_level(&mut monkeys, 10_000)
        );
    }
}
