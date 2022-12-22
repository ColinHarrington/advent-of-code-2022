use std::cmp::Ordering;
use std::fmt;
use std::fmt::Pointer;
use std::iter::zip;
use std::ops::ControlFlow;
use std::ptr::write;
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use nom::branch::alt;
use nom::character::complete::{char as nom_char, line_ending, not_line_ending, u32 as nom_u32};
use nom::IResult;
use nom::multi::{many0, many1, separated_list0, separated_list1};
use nom::sequence::{preceded, separated_pair, terminated, tuple};
use yaah::*;

type PacketPair = (Packet, Packet);

#[aoc_generator(day13)]
fn gen(input: &'static str) -> Vec<PacketPair> {
    packet_pairs(input).unwrap().1
}

//697 too low
//5394 too low
//4208 too low
#[aoc(day13, part1)]
fn solve_part1(pairs: &Vec<PacketPair>) -> u32 {
    pairs.iter().enumerate()
        .map(|(idx, pair)| (1 + idx, pair))
        .inspect(|(i, pair)| println!("== Pair {} ==", i))
        .filter(|(idx, (packet1, packet2))| packet1.compare(packet2) == PacketOrder::Correct)
        .map(|(i, _)| i as u32)
        .inspect(|i| println!("Index = {i}"))
        .sum()
}

fn packet_pairs(input: &str) -> IResult<&str, Vec<(Packet, Packet)>> {
    let (input, packets) = separated_list1(tuple((line_ending, line_ending)),
                                           separated_pair(packet, line_ending, packet))(input)?;
    Ok((input, packets))
}

fn packet(input: &str) -> IResult<&str, Packet> {
    let (input, data) = preceded(nom_char('['), terminated(
        separated_list0(nom_char(','), alt((packet_data_value, packet_data_list))),
        nom_char(']')))(input)?;
    Ok((input, Packet { data }))
}

fn packet_data_value(input: &str) -> IResult<&str, PacketData> {
    let (input, value) = nom_u32(input)?;
    Ok((input, PacketData::Value(PacketDataValue { value })))
}

fn packet_data_list(input: &str) -> IResult<&str, PacketData> {
    let (input, Packet { data }) = packet(input)?;
    Ok((input, PacketData::List(PacketDataList { list: data })))
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Packet {
    data: Vec<PacketData>,
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.data.iter().map(|d| format!("{}", d)).join(","))
    }
}

impl Packet {
    fn compare(&self, other: &Packet) -> PacketOrder {
        println!("  - Compare {self} vs {other}");
        self.data.iter().zip_longest(other.data.iter())
            .map(|pair| match pair {
                Both(l, r) => l.compare(r),
                Left(l) => PacketOrder::Incorrect,
                Right(r) => PacketOrder::Continue,
            })
            // .inspect(|o|println!("PacketOrder: {:?}", o))
            .find(|order| match order {
                PacketOrder::Incorrect => true,
                PacketOrder::Correct => true,
                _ => false
            }).unwrap_or(PacketOrder::Correct)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum PacketOrder {
    Correct,
    Incorrect,
    Continue,
}

trait PacketCompare {
    fn compare(&self, other: &PacketData) -> PacketOrder;
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct PacketDataValue {
    value: u32,
}

impl fmt::Display for PacketDataValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PacketDataValue {
    fn to_list(&self) -> PacketDataList {
        PacketDataList { list: vec![PacketData::Value(self.clone())] }
    }
}

impl PacketCompare for PacketDataValue {
    fn compare(&self, other: &PacketData) -> PacketOrder {
        println!("  - Compare {self} vs {other}");
        match other {
            PacketData::Value(data) => match self.value.cmp(&data.value) {
                Ordering::Greater => PacketOrder::Incorrect,
                Ordering::Less => PacketOrder::Correct,
                Ordering::Equal => PacketOrder::Continue
            },
            PacketData::List(list) => self.to_list().compare_lists(list)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct PacketDataList {
    list: Vec<PacketData>,
}

impl fmt::Display for PacketDataList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.list.iter().map(|d| format!("{}", d)).join(","))
    }
}

impl PacketCompare for PacketDataList {
    fn compare(&self, other: &PacketData) -> PacketOrder {
        match other {
            PacketData::Value(data) => self.compare_lists(&data.to_list()),
            PacketData::List(list) => self.compare_lists(list)
        }
    }
}

impl PacketDataList {
    /// If both values are lists, compare the first value of each list, then the second value, and so on.
    /// If the left list runs out of items first, the inputs are in the right order.
    /// If the right list runs out of items first, the inputs are not in the right order.
    /// If the lists are the same length and no comparison makes a decision about the order, continue checking the next part of the input.
    fn compare_lists(&self, other: &PacketDataList) -> PacketOrder {
        println!("  - Compare {self} vs {other}");
        self.list.iter().zip_longest(other.list.iter())
            .map(|pair| match pair {
                Both(l, r) => l.compare(r),
                Left(_) => PacketOrder::Incorrect,
                Right(_) => PacketOrder::Continue,
            }).find(|order| match order {
            PacketOrder::Incorrect => true,
            PacketOrder::Correct => true,
            _ => false
        }).unwrap_or(PacketOrder::Continue)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PacketData {
    Value(PacketDataValue),
    List(PacketDataList),
}

impl fmt::Display for PacketData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PacketData::Value(value) => value.fmt(f),
            PacketData::List(data) => data.fmt(f)
        }
    }
}

impl PacketCompare for PacketData {
    fn compare(&self, other: &PacketData) -> PacketOrder {
        match self {
            PacketData::Value(data) => data.compare(other),
            PacketData::List(list) => list.compare(other)
        }
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;
    use itertools::{enumerate, Itertools};
    use crate::day13::{gen, packet, Packet, packet_data_value, packet_pairs, PacketCompare, PacketData, PacketDataValue, PacketOrder, solve_part1};

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
        assert_eq!(Ok(("", PacketData::Value(PacketDataValue { value: 1 }))), packet_data_value("1"));
    }

    #[test]
    fn packet_display() {
        let (tail, pairs) = packet_pairs(EXAMPLE).unwrap();

        let constructed = pairs.iter().map(|(a, b)| format!("{a}\n{b}")).join("\n\n");
        assert_eq!(EXAMPLE, constructed);
    }

    #[test]
    fn packet_comparison() {
        let (tail, pairs) = packet_pairs(EXAMPLE).unwrap();

        let expected_results = vec![true, true, false, true, false, true, false, false];

        for ((i, (p1, p2)), &expected) in pairs.iter().enumerate().zip(expected_results.iter()) {
            println!("== Pair {} ==", i + 1);
            assert_eq!(expected, p1.compare(p2) == PacketOrder::Correct);
        }
    }

    /// If both values are integers, the lower integer should come first.
    /// If the left integer is lower than the right integer, the inputs are in the right order.
    /// If the left integer is higher than the right integer, the inputs are not in the right order.
    /// Otherwise, the inputs are the same integer; continue checking the next part of the input.
    #[test]
    fn compare_value_packets() {
        let test_data = vec![
            (3, 5, PacketOrder::Correct),
            (3, 1, PacketOrder::Incorrect),
            (1, 1, PacketOrder::Continue),
        ];

        for (left, right, expected) in test_data {
            let value1 = PacketDataValue { value: left };
            let value2 = PacketData::Value(PacketDataValue { value: right });
            assert_eq!(expected, value1.compare(&value2));
        }
    }

    #[test]
    fn part1() {
        assert_eq!(13, solve_part1(&gen(EXAMPLE)));
    }
}