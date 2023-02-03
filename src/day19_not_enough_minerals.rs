use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use anyhow::Context;
use log::{debug, info};

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input19.txt").unwrap();
    // let input = "Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.
    // Blueprint 2: Each ore robot costs 2 ore. Each clay robot costs 3 ore. Each obsidian robot costs 3 ore and 8 clay. Each geode robot costs 3 ore and 12 obsidian.";
    let blueprints: Vec<Blueprint> = input.lines().map(|l| l.parse().unwrap()).collect();
    println!(
        "sum quality levels: {}",
        sum_quality_levels(24, &blueprints)
    );

    println!(
        "product after 32 minutes: {}",
        product_first_three(32, &blueprints)
    );
}

fn sum_quality_levels(minutes: u32, blueprints: &[Blueprint]) -> u32 {
    blueprints
        .iter()
        .map(|blueprint| {
            let geodes = State::new(minutes).max_geodes(blueprint);
            println!("max for blueprint {}: {}", blueprint.id, geodes);
            blueprint.id * geodes
        })
        .sum()
}

fn product_first_three(minutes: u32, blueprints: &[Blueprint]) -> u32 {
    blueprints
        .iter()
        .take(3)
        .map(|blueprint| {
            let geodes = State::new(minutes).max_geodes(blueprint);
            println!("max for blueprint {}: {}", blueprint.id, geodes);
            geodes
        })
        .product()
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct State {
    resources: [u32; 4],
    bots: [u32; 4],
    pub minutes_left: u32,
}

impl State {
    pub fn get_resource(&self, mineral: Mineral) -> u32 {
        self.resources[mineral as u8 as usize - 1]
    }
    pub fn get_resource_mut(&mut self, mineral: Mineral) -> &mut u32 {
        &mut self.resources[mineral as u8 as usize - 1]
    }
    pub fn get_bot_mut(&mut self, mineral: Mineral) -> &mut u32 {
        &mut self.bots[mineral as u8 as usize - 1]
    }
    pub fn get_bot(&self, mineral: Mineral) -> u32 {
        self.bots[mineral as u8 as usize - 1]
    }
    pub fn collect_resources(&mut self) {
        for i in 0..self.resources.len() {
            self.resources[i] += self.bots[i];
        }
    }
}

// #[derive(Clone)]
// struct MaxResult {
//     pub geodes_count: u32,
//     pub history: Vec<State>,
// }
type MaxResult = u32;

struct Cache {
    cache: HashMap<State, MaxResult>,
    minutes_left: u32,
    initial_minutes: u32,
}

impl Cache {
    pub fn new(minutes: u32) -> Self {
        Self {
            cache: Default::default(),
            minutes_left: 0,
            initial_minutes: minutes,
        }
    }
    pub fn get(&self, state: &State) -> Option<MaxResult> {
        self.cache.get(state).cloned()
    }
    pub fn set(&mut self, state: State, result: MaxResult) {
        if state.minutes_left > self.minutes_left {
            info!(
                "{}% done ({}/{})",
                100 * (state.minutes_left) / self.initial_minutes,
                state.minutes_left,
                self.initial_minutes
            );
            self.minutes_left = state.minutes_left;
        }
        self.cache.insert(state, result);
    }
}

impl State {
    const MAX_BOTS: u32 = 15;
    pub fn new(minutes_left: u32) -> Self {
        let mut s: Self = Self {
            resources: Default::default(),
            bots: Default::default(),
            minutes_left,
        };
        *s.get_bot_mut(Mineral::Ore) += 1;
        s
    }
    fn max_geodes_work(mut self, blueprint: &Blueprint, cache: &mut Cache) -> MaxResult {
        if self.minutes_left == 0 {
            return
                // MaxResult {
                // geodes_count:
                self
                    .get_resource(Mineral::Geode)
            //         ,
            //     history: vec![],
            // }
            ;
        }
        if let Some(result) = cache.get(&self) {
            return result;
        }
        let initial_state = self.clone();
        self.minutes_left -= 1;
        // let current_level = self.bots.keys().map(|k| *k as u8).max().unwrap();

        let bots_available = blueprint.available_bots(&self);
        let max = bots_available
            .into_iter()
            .filter(|&mineral| self.get_bot(mineral) <= Self::MAX_BOTS)
            // .filter(|mineral| *mineral as u8 >= current_level)
            .map(|mineral| {
                let mut s = self.clone();
                blueprint.consume_resources_for_bot(&mineral, &mut s);
                s.collect_resources();
                *s.get_bot_mut(mineral) += 1;
                let
                    // mut
                    max = s.
                    // clone().
                    max_geodes_work(blueprint, cache);
                // max.history.push(s);
                max
            })
            // .max_by_key(|s| s.geodes_count);
            .max();
        let result = if max.is_none() || self.get_resource(Mineral::Ore) < blueprint.max_ore_cost {
            self.collect_resources();
            let
                // mut
                res = self.clone().max_geodes_work(blueprint, cache);
            // res.history.push(self);
            if let Some(max) = max {
                // if res.geodes_count > max.geodes_count {
                //     res
                // } else {
                //     max
                // }
                max.max(res)
            } else {
                res
            }
        } else {
            max.unwrap()
        };
        cache.set(initial_state, result.clone());
        result
    }
    pub fn max_geodes(self, blueprint: &Blueprint) -> u32 {
        // let minutes = self.minutes_left;
        let result = {
            let mut cache = Cache::new(self.minutes_left);
            self.max_geodes_work(blueprint, &mut cache)
        };
        debug!("\nresults for {:?}:", blueprint);
        // for state in result.history.iter().rev() {
        //     info!("-- Minute {} --\n{:?}", minutes - state.minutes_left, state);
        // }
        // result.geodes_count
        result
    }
}

#[derive(Debug)]
struct Blueprint {
    id: u32,
    bot_costs: BTreeMap<Mineral, Cost>,
    max_ore_cost: u32,
}

impl Blueprint {
    pub fn consume_resources_for_bot(&self, bot: &Mineral, resources: &mut State) {
        for (&mineral, &cost) in self.bot_costs.get(bot).expect("bot not found").0.iter() {
            *resources.get_resource_mut(mineral) -= cost;
        }
    }
    pub fn available_bots(&self, resources: &State) -> Vec<Mineral> {
        self.bot_costs
            .iter()
            .filter(|(_, costs)| {
                costs
                    .0
                    .iter()
                    .all(|(&mineral, &cost)| resources.get_resource(mineral) >= cost)
            })
            .map(|(bot, _)| *bot)
            .collect()
    }
    pub fn build_from(id: u32, bot_costs: impl Iterator<Item = (Mineral, Cost)>) -> Self {
        let bot_costs: BTreeMap<_, _> = bot_costs.collect();

        Self {
            id,
            max_ore_cost: bot_costs
                .iter()
                .map(|(_, costs)| costs.0.get(&Mineral::Ore).copied().unwrap_or_default())
                .max()
                .unwrap(),
            bot_costs,
        }
    }
}

#[derive(Debug)]
struct Cost(BTreeMap<Mineral, u32>);

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Ord, PartialOrd)]
enum Mineral {
    Ore = 1,
    Clay,
    Obsidian,
    Geode,
}

impl FromStr for Blueprint {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');
        let id = parts
            .next()
            .context("no first line")?
            .split_whitespace()
            .nth(1)
            .context("no id")?
            .parse()?;
        let mut parts = parts.next().context("no second line")?.split('.');
        let mut words = parts.next().context("no ore")?.split_whitespace().rev();
        let ore = Cost(
            vec![(Mineral::Ore, words.nth(1).context("ore")?.parse()?)]
                .into_iter()
                .collect(),
        );
        let mut words = parts.next().context("no clay")?.split_whitespace().rev();
        let clay = Cost(
            vec![(Mineral::Ore, words.nth(1).context("ore 2")?.parse()?)]
                .into_iter()
                .collect(),
        );
        let mut words = parts
            .next()
            .context("no obsidian")?
            .split_whitespace()
            .rev();
        let obsidian = Cost(
            vec![
                (Mineral::Clay, words.nth(1).context("clay")?.parse()?),
                (Mineral::Ore, words.nth(2).context("ore 3")?.parse()?),
            ]
            .into_iter()
            .collect(),
        );
        let mut words = parts.next().context("no geode")?.split_whitespace().rev();
        let geode = Cost(
            vec![
                (
                    Mineral::Obsidian,
                    words.nth(1).context("obsidian")?.parse()?,
                ),
                (Mineral::Ore, words.nth(2).context("ore 4")?.parse()?),
            ]
            .into_iter()
            .collect(),
        );
        Ok(Self::build_from(
            id,
            vec![
                (Mineral::Ore, ore),
                (Mineral::Clay, clay),
                (Mineral::Obsidian, obsidian),
                (Mineral::Geode, geode),
            ]
            .into_iter(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.
Blueprint 2: Each ore robot costs 2 ore. Each clay robot costs 3 ore. Each obsidian robot costs 3 ore and 8 clay. Each geode robot costs 3 ore and 12 obsidian.";

        let blueprints: Vec<Blueprint> = input.lines().map(|l| l.parse().unwrap()).collect();
        assert_eq!(33, sum_quality_levels(24, &blueprints));
    }
}
