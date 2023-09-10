use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, line_ending, u32 as nom_u32};
use nom::combinator::{map, verify};
use nom::multi::{separated_list0, separated_list1};
use nom::sequence::{preceded, tuple};
use nom::IResult;
use petgraph::algo::floyd_warshall;
use petgraph::graphmap::UnGraphMap;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::ops::{Add, Div, Mul, Sub};
use yaah::*;

#[aoc_generator(day16)]
fn read_valves(input: &'static str) -> Vec<Valve> {
	valves(input).unwrap().1
}

#[aoc(day16, part1)]
fn solve_part1(valves: &Vec<Valve>) -> u32 {
	let volcano = build_volcano(valves.clone());

	volcano.max_pressure(volcano.start_state(30))
}

#[aoc(day16, part2)]
fn solve_part2(valves: &Vec<Valve>) -> u32 {
	build_volcano(valves.clone()).max_combo()
}

/// all valve indexes =>
fn distance_map(
	distances: HashMap<(ValveLabel, ValveLabel), u32>,
	valves: &Vec<Valve>,
) -> DistanceMap {
	HashMap::from_iter(
		distances
			.into_iter()
			.map(|((from, to), d)| (valves.iter().position(|valve| from == valve.name), to, d))
			.map(|(from, to, d)| (from, valves.iter().position(|valve| to == valve.name), d))
			.filter(|(from, to, _)| from.is_some() && to.is_some())
			.map(|(from, to, d)| ((from.unwrap(), to.unwrap()), d)),
	)
}

fn build_full_distance_map(valves: Vec<Valve>) -> HashMap<(ValveLabel, ValveLabel), u32> {
	let valve_map: HashMap<ValveLabel, Valve> =
		HashMap::from_iter(valves.into_iter().map(|valve| (valve.name, valve.clone())));
	let edges = valve_map
		.iter()
		.map(|(name, valve)| {
			valve
				.tunnels
				.iter()
				.filter_map(|tunnel| valve_map.get(tunnel))
				.map(|other| (name.clone(), other.name.clone()))
		})
		.flatten();
	let graph: UnGraphMap<ValveLabel, u32> = UnGraphMap::from_edges(edges);

	floyd_warshall(&graph, |_| 1u32).unwrap()
}

fn is_key_valve(valve: &Valve) -> bool {
	valve.flow > 0 || valve.name == ['A', 'A']
}

fn build_volcano(all_valves: Vec<Valve>) -> Volcano {
	let distances = build_full_distance_map(all_valves.clone());

	let valves: Vec<Valve> = all_valves
		.into_iter()
		.filter(is_key_valve)
		.sorted_by_key(|valve| valve.flow)
		.rev()
		.collect();
	let flows = valves.iter().map(|valve| valve.flow).collect();
	let map = distance_map(distances, &valves);
	Volcano { valves, flows, map }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub struct State {
	minutes: u32,
	current: usize,
	remaining: u16,
	pressure: u32,
}

impl Ord for State {
	fn cmp(&self, other: &Self) -> Ordering {
		self.pressure.cmp(&other.pressure)
	}
}

impl State {
	fn remains(&self, valve: &usize) -> bool {
		self.remaining & (1u16 << valve) != 0
	}

	fn remaining(&self, valve_count: usize) -> Vec<usize> {
		(0..valve_count)
			.into_iter()
			.filter(|valve| self.remains(valve))
			.collect()
	}

	/// Best possible case
	///     Every remaining valve is reachable with a distance of 1
	///     They are processed in greatest to least
	fn upper_bound(&self, volcano: &Volcano) -> u32 {
		volcano
			.flows
			.iter()
			.enumerate()
			.filter(|(valve, _)| self.remains(valve))
			.map(|(_, flow)| flow)
			.take(self.minutes.div(2) as usize)
			.fold((self.minutes, self.pressure), |(min, total), flow| {
				(min.sub(2), total.add(flow.mul(min - 2)))
			})
			.1
	}

	fn travel(&self, valve: usize, volcano: &Volcano) -> Option<State> {
		let distance = volcano.distance(self.current, valve);
		if let Some(minutes) = self.minutes.checked_sub(distance + 1) {
			if minutes > 0 {
				let flow = volcano.flows.get(valve).unwrap();
				Some(State {
					remaining: self.remaining & !(1u16 << valve),
					pressure: self.pressure.add(flow.mul(minutes)),
					minutes,
					current: valve,
				})
			} else {
				None
			}
		} else {
			None
		}
	}
}

type DistanceMap = HashMap<(usize, usize), u32>;

#[derive(Debug, Clone, PartialEq)]
pub struct Volcano {
	valves: Vec<Valve>,
	flows: Vec<u32>,
	map: DistanceMap,
}

impl Volcano {
	fn distance(&self, from: usize, to: usize) -> u32 {
		*self.map.get(&(from, to)).unwrap()
	}

	fn position(&self, name: ValveLabel) -> usize {
		self.valves
			.iter()
			.position(|valve| valve.name == name)
			.unwrap()
	}

	fn start_state(&self, minutes: u32) -> State {
		let current = self.position(['A', 'A']);
		let remaining = (0..self.valves.len())
			.filter(|v| current.ne(v))
			.map(|valve| 1 << valve)
			.fold(0, |acc, valve| acc | valve);
		State {
			minutes,
			current,
			remaining,
			pressure: 0,
		}
	}

	fn max_pressure(&self, start: State) -> u32 {
		let mut best = 0;

		let mut queue: VecDeque<State> = VecDeque::from([start]);

		while let Some(state) = queue.pop_front() {
			if state.pressure > best {
				best = state.pressure
			} else if best > state.upper_bound(self) {
				continue;
			}
			state
				.remaining(self.flows.len())
				.into_iter()
				.filter_map(|valve| state.travel(valve, self))
				.for_each(|branch| queue.push_back(branch));
		}
		best
	}

	fn max_combo(&self) -> u32 {
		let start = self.start_state(26);
		self.generate_remaining_combinations()
			.into_par_iter()
			.map(|(remaining, b)| (self.max_pressure(State { remaining, ..start }), b))
			.map(|(a, remaining)| (a, self.max_pressure(State { remaining, ..start })))
			.map(|(a, b)| a + b)
			.max()
			.unwrap_or(0)
	}

	/// Options for splitting the workload into two parts
	fn generate_remaining_combinations(&self) -> Vec<(u16, u16)> {
		let max = 2usize.pow(self.valves.len().sub(1) as u32).sub(1) as u16;
		let half = max >> 1;
		(1..=half).into_iter().map(|a| (a, !a & max)).collect_vec()
	}
}

pub type ValveLabel = [char; 2];

pub struct ValveName([char; 2]);

impl From<&str> for ValveName {
	fn from(value: &str) -> Self {
		Self(value.chars().collect::<Vec<char>>().try_into().unwrap())
	}
}

#[derive(Debug, Eq, PartialEq, Clone, Ord, PartialOrd, Hash)]
pub struct Valve {
	name: ValveLabel,
	flow: u32,
	tunnels: Vec<ValveLabel>,
}

fn valves(input: &str) -> IResult<&str, Vec<Valve>> {
	separated_list1(line_ending, valve)(input)
}

fn valve(input: &str) -> IResult<&str, Valve> {
	map(
		tuple((
			preceded(tag("Valve "), valve_label),
			preceded(tag(" has "), valve_flow_rate),
			preceded(tag("; "), tunnels),
		)),
		|(name, flow, tunnels)| Valve {
			name,
			flow,
			tunnels,
		},
	)(input)
}

fn valve_flow_rate(input: &str) -> IResult<&str, u32> {
	preceded(tag("flow rate="), nom_u32)(input)
}

fn tunnels(input: &str) -> IResult<&str, Vec<ValveLabel>> {
	alt((valve_tunnels, valve_tunnel))(input)
}

fn valve_tunnels(input: &str) -> IResult<&str, Vec<ValveLabel>> {
	preceded(
		tag("tunnels lead to valves "),
		separated_list0(tag(", "), valve_label),
	)(input)
}

fn valve_tunnel(input: &str) -> IResult<&str, Vec<ValveLabel>> {
	preceded(
		tag("tunnel leads to valve "),
		map(valve_label, |label: ValveLabel| vec![label]),
	)(input)
}

fn valve_label(input: &str) -> IResult<&str, ValveLabel> {
	map(verify(alpha1, |s: &str| s.len() == 2), |s: &str| {
		s.chars().collect::<Vec<char>>().try_into().unwrap()
	})(input)
}

#[cfg(test)]
mod test {
	use crate::day16::{
		read_valves, solve_part1, solve_part2, tunnels, valve_flow_rate, valve_label,
		valve_tunnels, valves,
	};

	const EXAMPLE: &str = r"Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
Valve BB has flow rate=13; tunnels lead to valves CC, AA
Valve CC has flow rate=2; tunnels lead to valves DD, BB
Valve DD has flow rate=20; tunnels lead to valves CC, AA, EE
Valve EE has flow rate=3; tunnels lead to valves FF, DD
Valve FF has flow rate=0; tunnels lead to valves EE, GG
Valve GG has flow rate=0; tunnels lead to valves FF, HH
Valve HH has flow rate=22; tunnel leads to valve GG
Valve II has flow rate=0; tunnels lead to valves AA, JJ
Valve JJ has flow rate=21; tunnel leads to valve II";

	#[test]
	fn parse_valve_label() {
		assert_eq!(Ok(("", ['A', 'A'])), valve_label("AA"));
		assert_eq!(Ok(("", ['A', 'z'])), valve_label("Az"));

		assert!(valve_label("ABC").is_err());
		assert!(valve_label("A0").is_err());
		assert!(valve_label("  ").is_err());
	}

	#[test]
	fn parse_valve_flow_rate() {
		assert_eq!(Ok(("", 0)), valve_flow_rate("flow rate=0"));
		assert_eq!(Ok(("", 20)), valve_flow_rate("flow rate=20"));
	}

	#[test]
	fn parse_valve_tunnels() {
		assert_eq!(
			Ok(("", vec![['D', 'D'], ['I', 'I'], ['B', 'B']])),
			valve_tunnels("tunnels lead to valves DD, II, BB")
		);
		assert_eq!(
			Ok(("", vec![['G', 'G']])),
			tunnels("tunnel leads to valve GG")
		);
	}

	#[test]
	fn test_input_parsing() {
		let (tail, valves) = valves(EXAMPLE).unwrap();

		assert_eq!("", tail);

		assert_eq!(10, valves.len())
	}

	#[test]
	fn part1() {
		assert_eq!(1651, solve_part1(&read_valves(EXAMPLE)))
	}

	#[test]
	fn part2() {
		assert_eq!(1707, solve_part2(&read_valves(EXAMPLE)))
	}
}
