use itertools::Itertools;
use nom::branch::alt;
use nom::character::complete::{char as nom_char, line_ending, u32 as nom_u32};
use nom::multi::{many1, separated_list0, separated_list1};
use nom::sequence::{preceded, separated_pair, terminated, tuple};
use nom::IResult;
use std::cmp::Ordering;
use std::fmt;
use yaah::*;

type PacketPair = (PacketData, PacketData);

#[aoc_generator(day13, part1)]
fn generate_packet_pairs(input: &'static str) -> Vec<PacketPair> {
	packet_pairs(input).unwrap().1
}

#[aoc_generator(day13, part2)]
fn generate_packet_list(input: &'static str) -> Vec<PacketData> {
	separated_list1(many1(line_ending), packet)(input)
		.unwrap()
		.1
}

#[aoc(day13, part1)]
fn solve_part1(pairs: &[PacketPair]) -> u32 {
	pairs
		.iter()
		.enumerate()
		.map(|(idx, pair)| (1 + idx, pair))
		.filter(|(_, (packet1, packet2))| packet1.cmp(packet2) == Ordering::Less)
		.map(|(i, _)| i as u32)
		.sum()
}

#[aoc(day13, part2)]
fn solve_part2(input: &[PacketData]) -> u32 {
	let dividers = vec![packet("[[2]]").unwrap().1, packet("[[6]]").unwrap().1];

	let mut packets: Vec<PacketData> = input
		.iter()
		.cloned()
		.chain(dividers.iter().cloned())
		.collect();

	packets.sort();

	packets
		.iter()
		.enumerate()
		.map(|(idx, packet)| (1 + idx, packet))
		.filter(|(_, packet)| dividers.contains(packet))
		.map(|(n, _)| n as u32)
		.product()
}

fn packet_pairs(input: &str) -> IResult<&str, Vec<(PacketData, PacketData)>> {
	let (input, packets) = separated_list1(
		tuple((line_ending, line_ending)),
		separated_pair(packet, line_ending, packet),
	)(input)?;
	Ok((input, packets))
}

fn packet(input: &str) -> IResult<&str, PacketData> {
	let (input, data) = preceded(
		nom_char('['),
		terminated(
			separated_list0(nom_char(','), alt((packet_data_value, packet))),
			nom_char(']'),
		),
	)(input)?;
	Ok((input, PacketData::List(data)))
}

fn packet_data_value(input: &str) -> IResult<&str, PacketData> {
	let (input, value) = nom_u32(input)?;
	Ok((input, PacketData::Value(value)))
}

#[derive(Debug, Eq, Clone)]
pub enum PacketData {
	Value(u32),
	List(Vec<PacketData>),
}

impl fmt::Display for PacketData {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			PacketData::Value(value) => write!(f, "{}", value),
			PacketData::List(list) => {
				write!(f, "[{}]", list.iter().map(|d| format!("{}", d)).join(","))
			}
		}
	}
}

impl PartialEq<Self> for PacketData {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Value(left), Self::Value(right)) => left == right,
			(Self::Value(left), Self::List(right)) => vec![PacketData::Value(*left)] == *right,
			(Self::List(left), Self::Value(right)) => *left == vec![PacketData::Value(*right)],
			(Self::List(left), Self::List(right)) => left == right,
		}
	}
}

impl PartialOrd<Self> for PacketData {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for PacketData {
	fn cmp(&self, other: &Self) -> Ordering {
		match (self, other) {
			(Self::Value(left), Self::Value(right)) => left.cmp(right),
			(Self::Value(left), Self::List(right)) => vec![PacketData::Value(*left)].cmp(right),
			(Self::List(left), Self::Value(right)) => left.cmp(&vec![PacketData::Value(*right)]),
			(Self::List(left), Self::List(right)) => left.cmp(right),
		}
	}
}

#[cfg(test)]
mod test {
	use crate::day13::{
		generate_packet_list, generate_packet_pairs, packet_data_value, packet_pairs, solve_part1,
		solve_part2, PacketData,
	};
	use itertools::Itertools;
	use std::cmp::Ordering;

	const EXAMPLE: &str = r"[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]";

	#[test]
	fn parsing_packet_pairs() {
		let (tail, pairs) = packet_pairs(EXAMPLE).unwrap();
		assert_eq!("", tail);
		assert_eq!(8, pairs.len());
	}

	#[test]
	fn parsing_packet_data_value() {
		assert_eq!(Ok(("", PacketData::Value(1))), packet_data_value("1"));
	}

	#[test]
	fn packet_display() {
		let (_, pairs) = packet_pairs(EXAMPLE).unwrap();

		let constructed = pairs.iter().map(|(a, b)| format!("{a}\n{b}")).join("\n\n");
		assert_eq!(EXAMPLE, constructed);
	}

	#[test]
	fn packet_comparison() {
		let (_, pairs) = packet_pairs(EXAMPLE).unwrap();

		let expected_results = vec![true, true, false, true, false, true, false, false];

		for ((i, (p1, p2)), &expected) in pairs.iter().enumerate().zip(expected_results.iter()) {
			println!("== Pair {} ==", i + 1);
			assert_eq!(expected, p1.cmp(p2) == Ordering::Less);
		}
	}

	#[test]
	fn part1() {
		assert_eq!(13, solve_part1(&generate_packet_pairs(EXAMPLE)));
	}

	#[test]
	fn part2() {
		assert_eq!(140, solve_part2(&generate_packet_list(EXAMPLE)));
	}
}
