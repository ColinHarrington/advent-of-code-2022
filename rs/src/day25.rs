use itertools::Itertools;
use radix_fmt::radix_5;
use std::fmt::{Display, Formatter};
use yaah::*;

pub struct Snafu {
	value: Vec<char>,
}

impl Snafu {
	fn to_i64(&self) -> i64 {
		self.base5() - self.offset()
	}
	fn offset(&self) -> i64 {
		i64::from_str_radix("2".repeat(self.value.len()).as_str(), 5).unwrap()
	}
	fn base5(&self) -> i64 {
		i64::from_str_radix(
			self.value
				.clone()
				.into_iter()
				.map(decode_snafu_char)
				.join("")
				.as_str(),
			5,
		)
		.unwrap()
	}
}

fn decode_snafu_char(c: char) -> char {
	match c {
		'=' => '0',
		'-' => '1',
		'0' => '2',
		'1' => '3',
		'2' => '4',
		_ => panic!(),
	}
}

fn encode_snafu_char(c: char) -> char {
	match c {
		'0' => '=',
		'1' => '-',
		'2' => '0',
		'3' => '1',
		'4' => '2',
		_ => panic!(),
	}
}

impl From<i64> for Snafu {
	fn from(value: i64) -> Self {
		let base5 = format!("{}", radix_5(value));
		let offset = i64::from_str_radix("2".repeat(base5.len()).as_str(), 5).unwrap();
		format!("{}", radix_5(value + offset))
			.chars()
			.map(encode_snafu_char)
			.join("")
			.as_str()
			.into()
	}
}

impl From<String> for Snafu {
	fn from(value: String) -> Self {
		Snafu {
			value: value.chars().collect(),
		}
	}
}

impl From<&str> for Snafu {
	fn from(value: &str) -> Self {
		Snafu {
			value: value.chars().collect(),
		}
	}
}

impl Display for Snafu {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.value.iter().join(""))
	}
}

#[aoc_generator(day25)]
fn read_fuel_requirements(input: &'static str) -> Vec<Snafu> {
	input.lines().map(|line| line.into()).collect()
}

#[aoc(day25, part1)]
fn solve_part1(fuel_requirements: &[Snafu]) -> String {
	Snafu::from(
		fuel_requirements
			.iter()
			.map(|snafu| snafu.to_i64())
			.sum::<i64>(),
	)
	.to_string()
}

#[cfg(test)]
mod test {
	use crate::day25::{read_fuel_requirements, solve_part1, Snafu};

	const EXAMPLE: &str = r"1=-0-2
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

	#[test]
	fn part1() {
		let fuel_req = read_fuel_requirements(EXAMPLE);
		assert_eq!("2=-1=0", solve_part1(&fuel_req));
	}

	#[test]
	fn to_snafu() {
		assert_eq!(Snafu::from(4890).to_string(), "2=-1=0".to_string());
	}
}
