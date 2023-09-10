use itertools::Itertools;
use nom::bytes::complete::tag;
use nom::character::complete::{i32 as nom_i32, line_ending};
use nom::multi::separated_list1;
use nom::sequence::{preceded, separated_pair};
use nom::IResult;
use nom::Parser;
use std::ops::RangeInclusive;
use yaah::*;

#[aoc_generator(day15)]
fn gen_sensors(input: &'static str) -> Vec<Sensor> {
	sensors(input).unwrap().1
}

#[aoc(day15, part1)]
fn solve_part1(sensors: &Vec<Sensor>) -> u32 {
	row_coverage(2000000, sensors)
}

#[aoc(day15, part2)]
fn solve_part2(sensors: &Vec<Sensor>) -> i64 {
	find_distress_signal(0..=4000000, sensors)
}

fn tuning_frequency(location: Location) -> i64 {
	(location.0 as i64) * 4000000 + (location.1 as i64)
}

fn find_distress_signal(rows: RangeInclusive<i32>, sensors: &Vec<Sensor>) -> i64 {
	let beacon = rows
		.clone()
		.find_map(|row| {
			sensors
				.iter()
				.filter_map(|sensor| sensor.range_at(row)) //Sensors in range of row
				.sorted_by(|left, right| left.start().cmp(right.start()))
				.fold(vec![], fold_ranges)
				.into_iter()
				.find_map(|range| {
					match (range.contains(rows.start()), range.contains(rows.end())) {
						(true, false) => Some(Location(range.end() + 1, row)),
						_ => None,
					}
				})
		})
		.unwrap();
	tuning_frequency(beacon)
}

fn row_coverage(row: i32, sensors: &Vec<Sensor>) -> u32 {
	sensors
		.iter()
		.filter_map(|sensor| sensor.range_at(row))
		.sorted_by(|left, right| left.start().cmp(right.start()))
		.fold(vec![], fold_ranges)
		.iter()
		.map(|range| (range.end() - range.start()) as u32)
		.sum::<u32>()
}

fn fold_ranges(
	mut ranges: Vec<RangeInclusive<i32>>,
	range: RangeInclusive<i32>,
) -> Vec<RangeInclusive<i32>> {
	match ranges.pop() {
		None => ranges.push(range),
		Some(prev) => ranges.extend(merge_range(prev, range)),
	}
	ranges
}

fn merge_range(left: RangeInclusive<i32>, right: RangeInclusive<i32>) -> Vec<RangeInclusive<i32>> {
	match (left.contains(right.start()), left.contains(right.end())) {
		(true, true) => vec![left.clone()], //Complete overlap
		(true, false) => vec![RangeInclusive::new(*left.start(), *right.end())], //Extended right
		(false, true) => vec![RangeInclusive::new(*right.start(), *left.end())], //Extend left
		(false, false) => match *right.start() == left.end() + 1 {
			true => vec![RangeInclusive::new(*left.start(), *right.end())],
			false => vec![left.clone(), right.clone()],
		},
	}
}

#[derive(Debug, Eq, PartialEq)]
pub struct Location(i32, i32);

#[derive(Debug, Eq, PartialEq)]
pub struct Sensor {
	at: Location,
	beacon: Location,
	distance: u32,
}

impl Sensor {
	pub fn new(at: Location, beacon: Location) -> Self {
		let distance = manhattan_distance(&at, &beacon);
		Self {
			at,
			beacon,
			distance,
		}
	}

	fn range_at(&self, row: i32) -> Option<RangeInclusive<i32>> {
		let ydiff = self.at.1.abs_diff(row);

		match ydiff {
			diff if diff > self.distance => None,
			diff => {
				let remaining = (self.distance - diff) as i32;
				Some((self.at.0 - remaining)..=(self.at.0 + remaining))
			}
		}
	}
}

fn manhattan_distance(from: &Location, to: &Location) -> u32 {
	from.0.abs_diff(to.0) + from.1.abs_diff(to.1)
}

fn sensors(input: &str) -> IResult<&str, Vec<Sensor>> {
	separated_list1(line_ending, sensor)(input)
}

fn sensor(input: &str) -> IResult<&str, Sensor> {
	separated_pair(
		preceded(tag("Sensor at "), location),
		tag(": closest beacon is at "),
		location,
	)
	.map(|(at, beacon)| Sensor::new(at, beacon))
	.parse(input)
}

fn location(input: &str) -> IResult<&str, Location> {
	separated_pair(
		preceded(tag("x="), nom_i32),
		tag(", "),
		preceded(tag("y="), nom_i32),
	)
	.map(|(x, y)| Location(x, y))
	.parse(input)
}

#[cfg(test)]
mod test {
	use crate::day15::{
		find_distress_signal, fold_ranges, gen_sensors, location, merge_range, row_coverage,
		sensor, Location, Sensor,
	};

	const EXAMPLE: &str = r"Sensor at x=2, y=18: closest beacon is at x=-2, y=15
Sensor at x=9, y=16: closest beacon is at x=10, y=16
Sensor at x=13, y=2: closest beacon is at x=15, y=3
Sensor at x=12, y=14: closest beacon is at x=10, y=16
Sensor at x=10, y=20: closest beacon is at x=10, y=16
Sensor at x=14, y=17: closest beacon is at x=10, y=16
Sensor at x=8, y=7: closest beacon is at x=2, y=10
Sensor at x=2, y=0: closest beacon is at x=2, y=10
Sensor at x=0, y=11: closest beacon is at x=2, y=10
Sensor at x=20, y=14: closest beacon is at x=25, y=17
Sensor at x=17, y=20: closest beacon is at x=21, y=22
Sensor at x=16, y=7: closest beacon is at x=15, y=3
Sensor at x=14, y=3: closest beacon is at x=15, y=3
Sensor at x=20, y=1: closest beacon is at x=15, y=3";

	#[test]
	fn parse_location() {
		assert_eq!(Ok(("", Location(2, 18))), location("x=2, y=18"));
		assert_eq!(Ok(("", Location(-2, 15))), location("x=-2, y=15"));
		assert_eq!(Ok(("", Location(20, 14))), location("x=20, y=14"));
		assert_eq!(Ok(("", Location(25, 17))), location("x=25, y=17"));
	}

	#[test]
	fn parse_sensor() {
		let line = "Sensor at x=2, y=18: closest beacon is at x=-2, y=15";
		let expected = Sensor {
			at: Location(2, 18),
			beacon: Location(-2, 15),
			distance: 7,
		};
		assert_eq!(Ok(("", expected)), sensor(line))
	}

	#[test]
	fn sensor_distance() {
		let (_, sensor) = sensor("Sensor at x=8, y=7: closest beacon is at x=2, y=10").unwrap();
		assert_eq!(9, sensor.distance);
	}

	#[test]
	fn parse_sensors() {
		let sensors = gen_sensors(EXAMPLE);

		assert_eq!(14, sensors.len());

		assert_eq!(
			*sensors.first().unwrap(),
			Sensor::new(Location(2, 18), Location(-2, 15))
		);

		assert_eq!(
			*sensors.last().unwrap(),
			Sensor::new(Location(20, 1), Location(15, 3))
		);
	}

	#[test]
	fn sensor_range_at() {
		let sensor = Sensor::new(Location(8, 7), Location(2, 10));

		assert_eq!(None, sensor.range_at(-3));
		assert_eq!(8..=8, sensor.range_at(-2).unwrap());
		assert_eq!(-1..=17, sensor.range_at(7).unwrap());
		assert_eq!(2..=14, sensor.range_at(10).unwrap());
		assert_eq!(8..=8, sensor.range_at(16).unwrap());
		assert_eq!(None, sensor.range_at(17));
	}

	#[test]
	fn part1_example() {
		assert_eq!(26, row_coverage(10, &gen_sensors(EXAMPLE)));
	}

	#[test]
	fn test_merge_range() {
		assert_eq!(vec![-2..=14], merge_range(-2..=2, 2..=14));
		assert_eq!(vec![2..=14], merge_range(2..=14, 2..=2));

		assert_eq!(vec![2..=8, 14..=18], merge_range(2..=8, 14..=18));
	}

	#[test]
	fn test_merge_ranges() {
		let example1 = vec![-2..=2, 2..=14, 2..=2, 12..=12, 14..=18, 16..=24];

		assert_eq!(
			vec![-2..=24],
			example1.into_iter().fold(vec![], fold_ranges)
		);

		let example2 = vec![-2..=2, 2..=2, 12..=12, 14..=18, 16..=24];

		assert_eq!(
			vec![-2..=2, 12..=12, 14..=24],
			example2.into_iter().fold(vec![], fold_ranges)
		);
	}

	#[test]
	fn part2_example() {
		assert_eq!(
			56000011,
			find_distress_signal(0..=20, &gen_sensors(EXAMPLE))
		);
	}
}
