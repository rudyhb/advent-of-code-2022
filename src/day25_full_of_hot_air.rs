use std::fmt::{Display, Formatter};
use std::iter::Sum;
use std::ops::Add;
use std::str::FromStr;

use anyhow::{bail, Context};
use itertools::{EitherOrBoth, Itertools};

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input25.txt").unwrap();
    let result = input
        .lines()
        .map(|v| v.parse::<Snafu>().unwrap())
        .sum::<Snafu>();
    println!("sum: {}", result);
}

#[derive(Clone, Debug, PartialEq)]
struct Snafu(Vec<SnafuDigit>);

#[derive(Copy, Clone, Debug, PartialEq)]
enum SnafuDigit {
    NonNegative(u64),
    Negative(u64),
}

impl TryFrom<char> for SnafuDigit {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if value.is_numeric() {
            let val = value.to_digit(10).context("parse int error")?;
            if val > 2 {
                bail!("out of bounds SnafuDigit '{}'", val);
            }
        }
        Ok(match value {
            '0' => SnafuDigit::NonNegative(0),
            '1' => SnafuDigit::NonNegative(1),
            '2' => SnafuDigit::NonNegative(2),
            '-' => SnafuDigit::Negative(1),
            '=' => SnafuDigit::Negative(2),
            other => bail!("invalid SnafuDigit '{}'", other),
        })
    }
}

impl From<SnafuDigit> for char {
    fn from(value: SnafuDigit) -> Self {
        match value {
            SnafuDigit::NonNegative(val) => {
                if val > 2 {
                    panic!("out of bounds SnafuDigit: {}", val);
                }
                char::from_digit(val as u32, 10).unwrap()
            }
            SnafuDigit::Negative(val) => {
                if val == 0 || val > 2 {
                    panic!("out of bounds SnafuDigit: {}", val);
                }
                if val == 1 {
                    '-'
                } else {
                    '='
                }
            }
        }
    }
}

impl Display for Snafu {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.iter().map(|&v| char::from(v)).collect::<String>()
        )
    }
}

impl FromStr for Snafu {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.chars()
                .map(|c| SnafuDigit::try_from(c))
                .collect::<anyhow::Result<_>>()?,
        ))
    }
}

impl From<&Snafu> for u64 {
    fn from(value: &Snafu) -> Self {
        u64::from(&Base5::from(value))
    }
}

impl From<u64> for Snafu {
    fn from(value: u64) -> Self {
        Self::from(&Base5::from(value))
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Base5(Vec<u64>);

impl From<u64> for Base5 {
    fn from(mut value: u64) -> Self {
        let mut result = vec![];
        while value > 0 {
            let r = value % 5;
            result.push(r);
            value /= 5;
        }
        result.reverse();
        Self(result)
    }
}

impl From<&Base5> for u64 {
    fn from(value: &Base5) -> Self {
        let n = value.0.len();
        value
            .0
            .iter()
            .enumerate()
            .map(|(i, &val)| 5u64.pow((n - i - 1) as u32) * val)
            .sum()
    }
}

impl From<&Snafu> for Base5 {
    fn from(value: &Snafu) -> Self {
        let mut carry = false;
        let mut result: Vec<u64> = value
            .0
            .iter()
            .rev()
            .map(|val| match val {
                SnafuDigit::NonNegative(val) => {
                    if *val == 0 && carry {
                        4
                    } else if carry {
                        carry = false;
                        *val - 1
                    } else {
                        *val
                    }
                }
                SnafuDigit::Negative(val) => {
                    let c = if carry { 1 } else { 0 };
                    carry = true;
                    5 - *val - c
                }
            })
            .collect();
        while let Some(0) = result.iter().last() {
            result.pop();
        }
        result.reverse();
        Self(result)
    }
}

impl From<&Base5> for Snafu {
    fn from(value: &Base5) -> Self {
        let mut result = vec![];
        let mut carry = false;
        for &value in value.0.iter().rev() {
            let value = if carry { value + 1 } else { value };
            if value <= 2 {
                result.push(SnafuDigit::NonNegative(value));
                carry = false;
            } else {
                if value == 5 {
                    result.push(SnafuDigit::NonNegative(0));
                } else {
                    result.push(SnafuDigit::Negative(5 - value));
                }
                carry = true;
            }
        }
        if carry {
            result.push(SnafuDigit::NonNegative(1));
        }
        result.reverse();
        Self(result)
    }
}

impl Add for &Base5 {
    type Output = Base5;

    fn add(self, rhs: Self) -> Self::Output {
        let mut carry = false;
        let mut result: Vec<_> = self
            .0
            .iter()
            .rev()
            .zip_longest(rhs.0.iter().rev())
            .map(|values| {
                let next = if carry { 1 } else { 0 }
                    + match values {
                        EitherOrBoth::Both(l, r) => *l + *r,
                        EitherOrBoth::Left(l) => *l,
                        EitherOrBoth::Right(r) => *r,
                    };
                if next >= 5 {
                    carry = true;
                    next - 5
                } else {
                    carry = false;
                    next
                }
            })
            .collect();
        if carry {
            result.push(1);
        }
        result.reverse();
        Base5(result)
    }
}

impl Add for &Snafu {
    type Output = Snafu;

    fn add(self, rhs: Self) -> Self::Output {
        Snafu::from(&(&Base5::from(self) + &Base5::from(rhs)))
    }
}

impl Sum for Snafu {
    fn sum<I: Iterator<Item = Self>>(mut iter: I) -> Self {
        let mut result = if let Some(next) = iter.next() {
            next
        } else {
            return Snafu::from(0u64);
        };
        for next in iter {
            result = &result + &next;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test0() {
        fn assert_eq(decimal: &str, snafu: &str) {
            println!("testing {} vs {}", decimal, snafu);
            let decimal = decimal.parse::<u64>().unwrap();
            let snafu = snafu.parse::<Snafu>().unwrap();
            assert_eq!(decimal, u64::from(&snafu));
            let base5 = Base5::from(decimal);
            println!("testing {:?} vs {}", base5.0, snafu);
            assert_eq!(base5, Base5::from(&snafu));
            assert_eq!(snafu, Snafu::from(&base5));
        }
        let decimal_snafu = "        1              1
        2              2
        3             1=
        4             1-
        5             10
        6             11
        7             12
        8             2=
        9             2-
       10             20
       15            1=0
       20            1-0
     2022         1=11-2
    12345        1-0---0
314159265  1121-1110-1=0";
        decimal_snafu.lines().for_each(|s| {
            let mut parts = s.trim().split_whitespace();
            assert_eq(parts.next().unwrap(), parts.next().unwrap())
        });

        let snafu_decimal = "1=-0-2     1747
 12111      906
  2=0=      198
    21       11
  2=01      201
   111       31
 20012     1257
   112       32
 1=-1=      353
  1-12      107
    12        7
    1=        3
   122       37";
        snafu_decimal.lines().for_each(|s| {
            let mut parts = s.trim().split_whitespace().rev();
            assert_eq(parts.next().unwrap(), parts.next().unwrap())
        });
    }

    #[test]
    fn test1() {
        let input = "1=-0-2
12111
2=0=
21
2=01
111
20012
112
1=-1=
1-12
12
1=
122";
        let result = input
            .lines()
            .map(|v| v.parse::<Snafu>().unwrap())
            .sum::<Snafu>();
        assert_eq!("2=-1=0", format!("{}", result));
    }
}
