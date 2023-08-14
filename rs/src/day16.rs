use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::iter::once;
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, line_ending, u32 as nom_u32};
use nom::combinator::{map, verify};
use nom::IResult;
use nom::multi::{separated_list0, separated_list1};
use nom::sequence::{preceded, tuple};
use petgraph::algo::dijkstra;
use petgraph::Outgoing;
use petgraph::graphmap::UnGraphMap;
use yaah::*;

#[aoc_generator(day16)]
fn read_valves(input: &'static str) -> Vec<Valve> {
    valves(input).unwrap().1
}

#[aoc(day16, part1)]
fn solve_part1(all_valves: &Vec<Valve>) -> u32 {
    let nodes:Vec<ValveNode> = all_valves.iter()
        .enumerate()
        .map(|(i, valve)| (1 << i, valve))
        .map(|(mask, valve)| ValveNode { name: valve.name.as_str(), flow_rate: valve.flow_rate, mask })
        .collect();

    let nodes_by_name: HashMap<String, ValveNode> = nodes.iter()
        .map(|node|(node.name.to_string(), node.clone()))
        .collect();

    let edges: Vec<(ValveNode, ValveNode, u32)> = all_valves.iter()
        .map(|valve| valve.tunnels.iter()
            .map(|name| nodes_by_name.get(name).unwrap())
            .map(|&right| (nodes_by_name.get(&valve.name).unwrap().clone(), right.clone(), 1))
            .collect::<Vec<(ValveNode, ValveNode, u32)>>()
        )
        .flatten()
        .collect();

    let graph: UnGraphMap<ValveNode, u32> = UnGraphMap::from_edges(edges);

    let condensed_graph = condensed_graph(&graph);

    let start = condensed_graph.nodes().find(|node| node.name == "AA").unwrap();
    let initial_state = PathState { remaining: 30, open: vec![&start], current: &start };

    most_pressure_release(initial_state, &condensed_graph)
}

fn condensed_graph<'a>(graph: &'a UnGraphMap<ValveNode<'a>, u32>) -> UnGraphMap<ValveNode<'a>, u32> {
    let edges:Vec<(ValveNode, ValveNode, u32)> = graph.nodes()
        .map(|node|dijkstra(&graph, node, None, |_| 1).iter()
            .filter(|(&other, _)| other != node)
            .filter(|(&other, _)| other.name == "AA" || other.flow_rate != 0)
            .map(|(&other, &cost)| (node, other, cost as u32))
            .collect::<Vec<(ValveNode, ValveNode, u32)>>()
        )
        .flatten()
        .collect();
    UnGraphMap::from_edges(edges)
}

#[aoc(day16, part2)]
fn solve_part2(all_valves: &Vec<Valve>) -> u32 {
    let nodes:Vec<ValveNode> = all_valves.iter()
        .enumerate()
        .map(|(i, valve)| (1 << i, valve))
        .map(|(mask, valve)| ValveNode { name: valve.name.as_str(), flow_rate: valve.flow_rate, mask })
        .collect();

    let nodes_by_name: HashMap<String, ValveNode> = nodes.iter()
        .map(|node|(node.name.to_string(), node.clone()))
        .collect();

    let edges: Vec<(ValveNode, ValveNode, u32)> = all_valves.iter()
        .map(|valve| valve.tunnels.iter()
            .map(|name| nodes_by_name.get(name).unwrap())
            .map(|&right| (nodes_by_name.get(&valve.name).unwrap().clone(), right.clone(), 1))
            .collect::<Vec<(ValveNode, ValveNode, u32)>>()
        )
        .flatten()
        .collect();

    let graph: UnGraphMap<ValveNode, u32> = UnGraphMap::from_edges(edges);
    let condensed_graph = condensed_graph(&graph);

    let start = condensed_graph.nodes().find(|node| node.name == "AA").unwrap();
    let initial_state = PathState { remaining: 26, open: vec![&start], current: &start };
    most_pressure_release(initial_state, &condensed_graph)
}

fn most_pressure_release(state: PathState, graph: &UnGraphMap<ValveNode, u32>) -> u32 {
    state.current.flow_rate * state.remaining + max_child_pressure(graph, &state)
}

fn max_child_pressure(graph: &UnGraphMap<ValveNode, u32>, state: &PathState) -> u32 {
    graph.edges_directed(*state.current, Outgoing)
        .map(|(_, child, &distance)| (child, distance))
        .filter(|(child, _)| state.closed(child))
        .filter(|(_, distance)| state.remaining > *distance)
        .map(|(child, distance)| most_pressure_release(move_to(state, &child, distance), graph))
        .max()
        .unwrap_or(0)
}

#[derive(Debug, Eq, PartialEq, Clone, Ord, PartialOrd, Hash)]
struct PathState<'a> {
    remaining: u32,
    open: Vec<&'a ValveNode<'a>>,
    current: &'a ValveNode<'a>,
}

impl PathState<'_> {
    fn closed(&self, node: &ValveNode) -> bool {
        !self.open.contains(&node)
    }
}

impl fmt::Display for PathState<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let open = self.open.iter().map(|node| node.name.clone()).join(",");
        write!(f, "{} minutes left, at {} already open=[{open}]", self.remaining, self.current)
    }
}

fn move_to<'a>(state: &'a PathState<'a>, current: &'a ValveNode<'a>, distance: u32) -> PathState<'a> {
    let open = state.open.clone().into_iter().chain(once(current)).collect();
    // let open = [state.open.clone(), vec![current]].concat();
    PathState {
        remaining: state.remaining - distance - 1,
        open,
        current,
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct ValveNode<'a> {
    name: &'a str,
    flow_rate: u32,
    mask: u64,
}

impl fmt::Display for ValveNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.name, self.flow_rate)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Ord, PartialOrd, Hash)]
pub struct Valve {
    name: String,
    flow_rate: u32,
    tunnels: Vec<String>,
}

fn valves(input: &str) -> IResult<&str, Vec<Valve>> {
    separated_list1(line_ending, valve)(input)
}

fn valve(input: &str) -> IResult<&str, Valve> {
    map(tuple((
        preceded(tag("Valve "), valve_label),
        preceded(tag(" has "), valve_flow_rate),
        preceded(tag("; "), tunnels)
    )), |(name, flow_rate, tunnels)| Valve { name, flow_rate, tunnels })(input)
}


fn valve_flow_rate(input: &str) -> IResult<&str, u32> {
    preceded(tag("flow rate="), nom_u32)(input)
}

fn tunnels(input: &str) -> IResult<&str, Vec<String>> {
    alt((valve_tunnels, valve_tunnel))(input)
}

fn valve_tunnels(input: &str) -> IResult<&str, Vec<String>> {
    preceded(tag("tunnels lead to valves "),
             separated_list0(tag(", "), valve_label))(input)
}

fn valve_tunnel(input: &str) -> IResult<&str, Vec<String>> {
    preceded(tag("tunnel leads to valve "),
             map(valve_label, |label: String| vec![label]))(input)
}

fn valve_label(input: &str) -> IResult<&str, String> {
    map(verify(alpha1, |s: &str| s.len() == 2), |s: &str| s.to_string())(input)
}

#[cfg(test)]
mod test {
    use crate::day16::{read_valves, solve_part1, solve_part2, tunnels, valve_flow_rate, valve_label, valve_tunnels, valves};

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
        assert_eq!(Ok(("", "AA".to_string())), valve_label("AA"));
        assert_eq!(Ok(("", "Az".to_string())), valve_label("Az"));

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
        assert_eq!(Ok(("", vec!["DD".to_string(), "II".to_string(), "BB".to_string()])), valve_tunnels("tunnels lead to valves DD, II, BB"));
        assert_eq!(Ok(("", vec!["GG".to_string()])), tunnels("tunnel leads to valve GG"));
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

    #[ignore]
    #[test]
    fn part2() {
        assert_eq!(1707, solve_part2(&read_valves(EXAMPLE)))
    }
}
