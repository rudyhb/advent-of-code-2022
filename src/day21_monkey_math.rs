use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::sync::Mutex;

use anyhow::{bail, ensure, Context};
use lazy_static::lazy_static;
use string_interner::{DefaultSymbol, StringInterner};

lazy_static! {
    static ref INTERNER: Mutex<StringInterner> = Mutex::new(StringInterner::new());
}

fn get_name(s: DefaultSymbol) -> String {
    INTERNER.lock().unwrap().resolve(s).unwrap().to_string()
}

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input21.txt").unwrap();
    let mut monkeys = MonkeyGroup::new(input.lines().map(|l| l.parse().unwrap()));
    let root = monkeys.run_until_goal();
    println!("root yells: {}", root.value);
    let path = monkeys.get_path("humn", "root");
    println!("path from humn to root: {:?}", path);
    println!("need to yell: {}", monkeys.solve());
}

#[derive(Clone)]
struct MonkeyGroup {
    monkeys: HashMap<DefaultSymbol, Monkey>,
    dependents: HashMap<DefaultSymbol, Vec<DefaultSymbol>>,
}

impl MonkeyGroup {
    pub fn solve(&self) -> i64 {
        let root = INTERNER.lock().unwrap().get("root").unwrap();
        let path = self.get_path("humn", "root");
        let penultimate = path.iter().rev().nth(1).unwrap().name;
        let other = self
            .monkeys
            .get(&root)
            .unwrap()
            .dependencies()
            .unwrap()
            .into_iter()
            .filter(|&s| s != penultimate)
            .next()
            .unwrap();
        let other_value = self.monkeys.get(&other).unwrap().value.unwrap();
        path.windows(2)
            .rev()
            .skip(1)
            .fold(other_value, |target, next| {
                next[1].make_result_by_changing(next[0].name, target, &self.monkeys)
            })
    }
    pub fn get_path(&self, from: &str, to: &str) -> Vec<Monkey> {
        let mut result = vec![];
        let mut current = self.monkeys.get(&Self::get_monkey(from)).unwrap();
        let end = self.monkeys.get(&Self::get_monkey(to)).unwrap();
        result.push(current.clone());
        while current as *const _ != end as *const _ {
            let next = self.dependents.get(&current.name).unwrap();
            if next.len() != 1 {
                panic!(
                    "there are {} options after {}",
                    next.len(),
                    get_name(current.name)
                );
            }
            current = self.monkeys.get(&next[0]).unwrap();
            result.push(current.clone());
        }
        result
    }
    pub fn new(monkeys: impl Iterator<Item = Monkey>) -> Self {
        let monkeys: HashMap<DefaultSymbol, Monkey> = monkeys.map(|m| (m.name, m)).collect();
        let mut dependents: HashMap<DefaultSymbol, Vec<DefaultSymbol>> = HashMap::new();
        for (parent, dependent) in monkeys
            .iter()
            .filter_map(|(&dependent, m)| {
                m.dependencies()
                    .map(|d| [(d[0], dependent), (d[1], dependent)])
            })
            .flatten()
        {
            dependents.entry(parent).or_default().push(dependent);
        }
        Self {
            monkeys,
            dependents,
        }
    }
    #[allow(unused)]
    pub fn solve_brute_force(&self) -> i64 {
        let goal = Self::get_monkey("root");
        let start = Self::get_monkey("humn");
        for i in 0.. {
            let mut monkeys = self.clone();
            monkeys.monkeys.get_mut(&start).unwrap().set_value(i);
            let result = monkeys.run_until(goal);
            if result.a == result.b {
                return i;
            }
        }
        panic!("no solution");
    }
    fn get_monkey(name: &str) -> DefaultSymbol {
        INTERNER
            .lock()
            .unwrap()
            .get(name)
            .expect("cannot find root monkey name")
    }
    pub fn run_until_goal(&mut self) -> MonkeyResult {
        self.run_until(Self::get_monkey("root"))
    }
    fn run_until(&mut self, goal_monkey: DefaultSymbol) -> MonkeyResult {
        let mut done: Vec<_> = self
            .monkeys
            .iter()
            .filter_map(|(key, m)| m.value.map(|_| key))
            .copied()
            .collect();
        while !done.is_empty() {
            let mut next = vec![];
            for parent in done.into_iter() {
                if let Some(dependents) = self.dependents.get(&parent) {
                    for dependent in dependents {
                        if let Some(result) = self
                            .monkeys
                            .get(dependent)
                            .unwrap()
                            .try_get_result(&self.monkeys)
                        {
                            self.monkeys
                                .get_mut(dependent)
                                .unwrap()
                                .set_value(result.value);
                            if goal_monkey == *dependent {
                                return result;
                            }
                            next.push(*dependent);
                        }
                    }
                }
            }
            done = next;
        }
        panic!("did not reach monkey ")
    }
}

#[derive(Clone)]
struct Monkey {
    name: DefaultSymbol,
    operation: MonkeyOperation,
    value: Option<i64>,
}

struct MonkeyResult {
    value: i64,
    a: i64,
    b: i64,
}

impl Monkey {
    pub fn make_result_by_changing(
        &self,
        variable: DefaultSymbol,
        result: i64,
        monkeys: &HashMap<DefaultSymbol, Monkey>,
    ) -> i64 {
        let dependencies = self.dependencies().unwrap();
        let left = dependencies[0] == variable;
        let other = monkeys
            .get(&if left {
                dependencies[1]
            } else {
                dependencies[0]
            })
            .unwrap()
            .value
            .unwrap();
        match &self.operation {
            MonkeyOperation::Value(_) => panic!("cannot change value"),
            MonkeyOperation::Sum(_, _) => result - other,
            MonkeyOperation::Difference(_, _) => {
                if left {
                    result + other
                } else {
                    other - result
                }
            }
            MonkeyOperation::Product(_, _) => result / other,
            MonkeyOperation::Division(_, _) => {
                if left {
                    result * other
                } else {
                    other / result
                }
            }
        }
    }
    pub fn try_get_result(&self, monkeys: &HashMap<DefaultSymbol, Monkey>) -> Option<MonkeyResult> {
        if let Some([a, b]) = self.dependencies() {
            if let Some(a) = monkeys.get(&a).unwrap().value {
                if let Some(b) = monkeys.get(&b).unwrap().value {
                    let value = match &self.operation {
                        MonkeyOperation::Value(v) => *v,
                        MonkeyOperation::Sum(_, _) => a + b,
                        MonkeyOperation::Difference(_, _) => a - b,
                        MonkeyOperation::Product(_, _) => a * b,
                        MonkeyOperation::Division(_, _) => a / b,
                    };
                    return Some(MonkeyResult { value, a, b });
                }
            }
        }
        None
    }
    pub fn set_value(&mut self, value: i64) {
        self.value = Some(value);
    }
    pub fn new(name: DefaultSymbol, operation: MonkeyOperation) -> Self {
        Self {
            value: if let MonkeyOperation::Value(v) = &operation {
                Some(*v)
            } else {
                None
            },
            name,
            operation,
        }
    }
    pub fn dependencies(&self) -> Option<[DefaultSymbol; 2]> {
        match &self.operation {
            MonkeyOperation::Value(_) => None,
            MonkeyOperation::Sum(a, b)
            | MonkeyOperation::Difference(a, b)
            | MonkeyOperation::Product(a, b)
            | MonkeyOperation::Division(a, b) => Some([*a, *b]),
        }
    }
}

impl Debug for Monkey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            get_name(self.name),
            match &self.operation {
                MonkeyOperation::Value(v) => v.to_string(),
                MonkeyOperation::Sum(a, b) => format!("{} + {}", get_name(*a), get_name(*b)),
                MonkeyOperation::Difference(a, b) => format!("{} - {}", get_name(*a), get_name(*b)),
                MonkeyOperation::Product(a, b) => format!("{} * {}", get_name(*a), get_name(*b)),
                MonkeyOperation::Division(a, b) => format!("{} / {}", get_name(*a), get_name(*b)),
            }
        )
    }
}

#[derive(Clone)]
enum MonkeyOperation {
    Value(i64),
    Sum(DefaultSymbol, DefaultSymbol),
    Difference(DefaultSymbol, DefaultSymbol),
    Product(DefaultSymbol, DefaultSymbol),
    Division(DefaultSymbol, DefaultSymbol),
}

impl FromStr for Monkey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut interner = INTERNER.lock().unwrap();
        let mut parts = s.split(": ");
        let name = interner.get_or_intern(parts.next().context("no name")?);
        let parts: Vec<_> = parts
            .next()
            .context("no operation")?
            .trim()
            .split_whitespace()
            .collect();
        let operation = if parts.len() == 1 {
            MonkeyOperation::Value(parts[0].parse()?)
        } else {
            ensure!(parts.len() == 3);
            let values = (
                interner.get_or_intern(parts[0]),
                interner.get_or_intern(parts[2]),
            );
            match parts[1] {
                "+" => MonkeyOperation::Sum(values.0, values.1),
                "-" => MonkeyOperation::Difference(values.0, values.1),
                "*" => MonkeyOperation::Product(values.0, values.1),
                "/" => MonkeyOperation::Division(values.0, values.1),
                other => bail!("invalid operation '{}'", other),
            }
        };
        Ok(Self::new(name, operation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "root: pppw + sjmn
dbpl: 5
cczh: sllz + lgvd
zczc: 2
ptdq: humn - dvpt
dvpt: 3
lfqf: 4
humn: 5
ljgn: 2
sjmn: drzm * dbpl
sllz: 4
pppw: cczh / lfqf
lgvd: ljgn * ptdq
drzm: hmdt - zczc
hmdt: 32";
        let mut monkeys = MonkeyGroup::new(input.lines().map(|l| l.parse().unwrap()));
        let root = monkeys.run_until_goal();
        assert_eq!(152, root.value);
        assert_eq!(301, monkeys.solve());
    }
}
