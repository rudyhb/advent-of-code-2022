use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use log::debug;
use string_interner::{DefaultSymbol, StringInterner, Symbol};

pub(crate) fn run() {
    let input = std::fs::read_to_string("input/input16.txt").unwrap();
    let network = Network::parse_from(&input).unwrap();
    // let best = network.run::<State>(30);
    // println!("open history: {}", best.formatted_history());
    println!(
        "max pressure released after 30 min: {}",
        network.find_max_pressure_faster(30, 1) // best.total_pressure
    );
    // let best = network.run::<CoopState>(26);
    println!("with coop:");
    // println!("open history: {}", best.formatted_history());
    println!(
        "coop max pressure released after 26 min: {}",
        network.find_max_pressure_faster(26, 2) // best.total_pressure
    );
}

struct Valve {
    name: DefaultSymbol,
    flow_rate: u64,
    tunnels: Vec<DefaultSymbol>,
}

struct Network {
    interner: StringInterner,
    valves: HashMap<DefaultSymbol, Valve>,
    shortest_paths: HashMap<(DefaultSymbol, DefaultSymbol), Vec<DefaultSymbol>>,
    start: DefaultSymbol,
}

#[derive(Clone)]
struct Actor {
    minutes_left: usize,
    pressure_opened: u64,
    position: DefaultSymbol,
}

impl Network {
    fn find_max_pressure_step(
        &self,
        actors: Vec<Actor>,
        closed_valves: HashSet<DefaultSymbol>,
    ) -> u64 {
        if closed_valves.is_empty() {
            return actors.iter().map(|a| a.pressure_opened).sum();
        }
        let next: Vec<Vec<(Actor, DefaultSymbol)>> = actors
            .iter()
            .map(|actor| {
                let from = actor.position;
                closed_valves
                    .iter()
                    .copied()
                    .filter_map(|to| {
                        let cost = self.shortest_paths.get(&(from, to)).unwrap().len() + 2;
                        if cost < actor.minutes_left {
                            let mut actor = actor.clone();
                            actor.minutes_left -= cost;
                            actor.pressure_opened += self.valves.get(&to).unwrap().flow_rate
                                * (actor.minutes_left as u64);
                            actor.position = to;
                            Some((actor, to))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .collect();
        match next.len() {
            1 => next
                .into_iter()
                .next()
                .unwrap()
                .into_iter()
                .map(|(actor, to)| {
                    let mut closed_valves = closed_valves.clone();
                    closed_valves.remove(&to);
                    self.find_max_pressure_step(vec![actor], closed_valves)
                })
                .max()
                .unwrap_or(actors[0].pressure_opened),
            2 => {
                let mut results: Vec<_> = {
                    let next = &next;
                    let closed_valves = &closed_valves;
                    (0..next[0].len())
                        .flat_map(|i| {
                            (0..next[1].len()).filter(move |&j| j != i).map(move |j| {
                                let first = &next[0][i];
                                let second = &next[1][j];
                                let actors = vec![first.0.clone(), second.0.clone()];
                                let mut closed_valves = closed_valves.clone();
                                closed_valves.remove(&first.1);
                                closed_valves.remove(&second.1);
                                self.find_max_pressure_step(actors, closed_valves)
                            })
                        })
                        .collect()
                };
                if next[0].len() <= 1 {
                    results.extend(next.into_iter().nth(1).unwrap().into_iter().map(
                        |(actor, to)| {
                            let mut closed_valves = closed_valves.clone();
                            closed_valves.remove(&to);
                            self.find_max_pressure_step(vec![actor], closed_valves)
                        },
                    ));
                }
                results
                    .into_iter()
                    .max()
                    .unwrap_or(actors[0].pressure_opened)
            }
            other => panic!("invalid number of actors: {}", other),
        }
    }

    pub fn find_max_pressure_faster(&self, minutes: usize, num_actors: usize) -> u64 {
        let closed_valves: HashSet<DefaultSymbol> = self
            .valves
            .iter()
            .filter(|(_, v)| v.flow_rate > 0)
            .map(|(i, _)| *i)
            .collect();
        let actors = vec![
            Actor {
                minutes_left: minutes,
                pressure_opened: 0,
                position: self.start,
            };
            num_actors
        ];
        self.find_max_pressure_step(actors, closed_valves)
    }
    pub fn parse_from(s: &str) -> Result<Self> {
        let mut interner = StringInterner::default();
        let valves: HashMap<DefaultSymbol, Valve> = s
            .lines()
            .map(|line| {
                let valve = Self::parse_valve(line, &mut interner);
                valve.map(|res| (res.name, res))
            })
            .collect::<Result<_>>()?;

        Ok(Self {
            start: interner.get("AA").expect("AA not found in network"),
            interner,
            shortest_paths: Self::create_shortest_paths(&valves),
            valves,
        })
    }
    fn create_shortest_paths(
        valves: &HashMap<DefaultSymbol, Valve>,
    ) -> HashMap<(DefaultSymbol, DefaultSymbol), Vec<DefaultSymbol>> {
        debug!("starting create shortest paths");
        let min = valves.keys().min().unwrap().to_usize();
        let max = valves.keys().max().unwrap().to_usize();
        let result = (min..=max)
            .flat_map(|i| {
                (min..=max).filter(move |j| &i != j).map(move |j| {
                    let i = DefaultSymbol::try_from_usize(i).unwrap();
                    let j = DefaultSymbol::try_from_usize(j).unwrap();
                    ((i, j), Self::get_shortest_path(valves, i, j))
                })
            })
            .collect();
        debug!("done create shortest paths");
        result
    }
    fn get_shortest_path(
        valves: &HashMap<DefaultSymbol, Valve>,
        from: DefaultSymbol,
        to: DefaultSymbol,
    ) -> Vec<DefaultSymbol> {
        let mut paths: Vec<_> = valves
            .get(&from)
            .unwrap()
            .tunnels
            .iter()
            .map(|&i| vec![i])
            .collect();
        for _ in 0..50 {
            let mut next = vec![];
            for mut path in paths.into_iter() {
                let last = path.iter().last().copied().unwrap();
                if last == to {
                    path.pop();
                    return path;
                }
                next.extend(valves.get(&last).unwrap().tunnels.iter().map(|&last| {
                    let mut copy = path.clone();
                    copy.push(last);
                    copy
                }));
            }
            paths = next;
        }
        panic!("no path between {:?} and {:?}", from, to);
    }
    fn parse_valve(s: &str, interner: &mut StringInterner) -> Result<Valve> {
        let mut parts = s.split(';');
        let mut words = parts.next().context("empty")?.split_whitespace();
        let name = interner.get_or_intern(words.nth(1).context("no words")?);
        let flow_rate: u64 = words
            .last()
            .context("no flow rate")?
            .split('=')
            .nth(1)
            .context("invalid flow rate")?
            .parse()?;
        let mut words = parts.next().context("no ;")?.split_whitespace().rev();
        let mut tunnels = Vec::new();
        loop {
            let word = words
                .next()
                .context("cannot find 'valves'")?
                .trim_matches(',');
            if word.trim_end_matches('s') == "valve" {
                break;
            }
            tunnels.push(interner.get_or_intern(word));
        }
        Ok(Valve {
            name,
            flow_rate,
            tunnels,
        })
    }
    #[allow(unused)]
    pub fn run<'a, S: Stateful<'a>>(&'a self, minutes: usize) -> S {
        let mut pending = vec![S::new(self, minutes, self.start)];
        let mut finished: Vec<S> = vec![];

        let mut i = 0;
        while !pending.is_empty() {
            i += 1;
            debug!("round {}: {} left", i, pending.len());
            pending = pending
                .into_iter()
                .flat_map(|state| match state.next() {
                    NextState::Done(s) => {
                        finished.push(s);
                        vec![]
                    }
                    NextState::Children(next) => next,
                })
                .collect();
        }

        finished
            .into_iter()
            .max_by_key(|s| s.get_total_pressure())
            .unwrap()
    }
}

trait Stateful<'a>
where
    Self: Sized,
{
    fn new(network: &'a Network, minutes: usize, start: DefaultSymbol) -> Self;
    fn next(self) -> NextState<Self>;
    fn get_total_pressure(&self) -> u64;
    fn get_open_history(&self) -> &[DefaultSymbol];
    fn get_network(&self) -> &'a Network;

    fn formatted_history(&self) -> String {
        format!(
            "{:?}",
            self.get_open_history()
                .iter()
                .map(|i| self.get_network().interner.resolve(*i).unwrap())
                .collect::<Vec<_>>()
        )
    }
}

#[derive(Clone)]
struct State<'a> {
    minutes_left: usize,
    network: &'a Network,
    closed_valves: HashSet<DefaultSymbol>,
    total_pressure: u64,
    position: DefaultSymbol,
    open_history: Vec<DefaultSymbol>,
}

impl<'a> State<'a> {
    pub fn new(network: &'a Network, minutes: usize, position: DefaultSymbol) -> Self {
        Self {
            minutes_left: minutes,
            closed_valves: network
                .valves
                .iter()
                .filter(|(_, v)| v.flow_rate > 0)
                .map(|(i, _)| *i)
                .collect(),
            network,
            total_pressure: 0,
            position,
            open_history: vec![],
        }
    }
    fn get_move_cost(&self, from: DefaultSymbol, to: DefaultSymbol) -> usize {
        self.network.shortest_paths.get(&(from, to)).unwrap().len() + 1
    }
    fn next_candidates(&self) -> Vec<DefaultSymbol> {
        self.closed_valves
            .iter()
            .filter(|&&next| {
                if self.position == next {
                    return false;
                }
                let cost = self.get_move_cost(self.position, next) + 1;
                cost < self.minutes_left
            })
            .copied()
            .collect()
    }
    fn move_and_open(&mut self, destination: DefaultSymbol) {
        if !self.closed_valves.remove(&destination) {
            panic!(
                "valve {} already removed",
                self.network.interner.resolve(destination).unwrap()
            )
        }
        let cost = self.get_move_cost(self.position, destination) + 1;
        self.minutes_left -= cost;
        self.position = destination;
        let valve = self.network.valves.get(&self.position).unwrap();
        self.total_pressure += (self.minutes_left as u64) * valve.flow_rate;
        self.open_history.push(self.position);
    }
    pub fn next(self) -> NextState<Self> {
        let next = self.next_candidates();
        if next.is_empty() {
            NextState::Done(self)
        } else {
            NextState::Children(
                next.into_iter()
                    .map(|next| {
                        let mut s = self.clone();
                        s.move_and_open(next);
                        s
                    })
                    .collect(),
            )
        }
    }
}

impl<'a> Stateful<'a> for State<'a> {
    fn new(network: &'a Network, minutes: usize, start: DefaultSymbol) -> Self {
        Self::new(network, minutes, start)
    }

    fn next(self) -> NextState<Self> {
        Self::next(self)
    }

    fn get_total_pressure(&self) -> u64 {
        self.total_pressure
    }

    fn get_open_history(&self) -> &[DefaultSymbol] {
        &self.open_history
    }

    fn get_network(&self) -> &'a Network {
        self.network
    }
}

#[derive(Clone)]
struct CoopState<'a> {
    network: &'a Network,
    moving: [MovingCharacter; 2],
    closed_valves: HashSet<DefaultSymbol>,
    total_pressure: u64,
    open_history: Vec<DefaultSymbol>,
}

#[derive(Clone)]
struct MovingCharacter {
    pub start_minute: usize,
    pub end_minute: usize,
    pub destination: DefaultSymbol,
    pub done: bool,
}

impl<'a> CoopState<'a> {
    pub fn new(network: &'a Network, minutes: usize, position: DefaultSymbol) -> Self {
        Self {
            closed_valves: network
                .valves
                .iter()
                .filter(|(_, v)| v.flow_rate > 0)
                .map(|(i, _)| *i)
                .collect(),
            network,
            total_pressure: 0,
            open_history: vec![],
            moving: [
                MovingCharacter {
                    start_minute: minutes,
                    end_minute: minutes,
                    destination: position,
                    done: false,
                },
                MovingCharacter {
                    start_minute: minutes,
                    end_minute: minutes,
                    destination: position,
                    done: false,
                },
            ],
        }
    }
    fn get_move_cost(&self, from: DefaultSymbol, to: DefaultSymbol) -> usize {
        self.network.shortest_paths.get(&(from, to)).unwrap().len() + 1
    }
    fn next_candidates(&self, index: usize) -> Vec<DefaultSymbol> {
        self.closed_valves
            .iter()
            .filter(|&&next| {
                if self.moving.iter().any(|m| m.destination == next) {
                    return false;
                }
                let cost = self.get_move_cost(self.moving[index].destination, next) + 1;
                cost < self.moving[index].end_minute
            })
            .copied()
            .collect()
    }
    fn move_and_open(&mut self, destination: DefaultSymbol, index: usize) {
        if destination != self.network.start && !self.closed_valves.remove(&destination) {
            panic!(
                "valve {} already removed",
                self.network.interner.resolve(destination).unwrap()
            )
        }
        let cost = self.get_move_cost(self.moving[index].destination, destination) + 1;
        let mut moving = &mut self.moving[index];
        moving.destination = destination;
        let flow_rate = self.network.valves.get(&destination).unwrap().flow_rate;
        moving.start_minute = moving.end_minute;
        moving.end_minute = moving.start_minute - cost;
        self.total_pressure += (moving.end_minute as u64) * flow_rate;
        self.open_history.push(moving.destination);
    }
    pub fn next(mut self) -> NextState<Self> {
        if self.moving.iter().all(|m| m.done) {
            NextState::Done(self)
        } else {
            let (index, _) = self
                .moving
                .iter()
                .enumerate()
                .max_by_key(|m| m.1.end_minute)
                .unwrap();
            let next = self.next_candidates(index);
            if next.is_empty() {
                self.moving[index] = MovingCharacter {
                    start_minute: 0,
                    end_minute: 0,
                    destination: self.network.start,
                    done: true,
                };
                NextState::Children(vec![self])
            } else {
                NextState::Children(
                    next.into_iter()
                        .map(|next| {
                            let mut s = self.clone();
                            s.move_and_open(next, index);
                            s
                        })
                        .collect(),
                )
            }
        }
    }
}

impl<'a> Stateful<'a> for CoopState<'a> {
    fn new(network: &'a Network, minutes: usize, start: DefaultSymbol) -> Self {
        Self::new(network, minutes, start)
    }

    fn next(self) -> NextState<Self> {
        Self::next(self)
    }

    fn get_total_pressure(&self) -> u64 {
        self.total_pressure
    }

    fn get_open_history(&self) -> &[DefaultSymbol] {
        &self.open_history
    }

    fn get_network(&self) -> &'a Network {
        self.network
    }
}

enum NextState<T> {
    Done(T),
    Children(Vec<T>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let input = "Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
Valve BB has flow rate=13; tunnels lead to valves CC, AA
Valve CC has flow rate=2; tunnels lead to valves DD, BB
Valve DD has flow rate=20; tunnels lead to valves CC, AA, EE
Valve EE has flow rate=3; tunnels lead to valves FF, DD
Valve FF has flow rate=0; tunnels lead to valves EE, GG
Valve GG has flow rate=0; tunnels lead to valves FF, HH
Valve HH has flow rate=22; tunnel leads to valve GG
Valve II has flow rate=0; tunnels lead to valves AA, JJ
Valve JJ has flow rate=21; tunnel leads to valve II";
        let network = Network::parse_from(input).unwrap();
        assert_eq!(1651, network.run::<State>(30).total_pressure);
        assert_eq!(1707, network.run::<CoopState>(26).total_pressure);

        assert_eq!(1651, network.find_max_pressure_faster(30, 1));
        assert_eq!(1707, network.find_max_pressure_faster(26, 2));
    }
}
