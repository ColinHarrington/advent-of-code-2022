use std::ops::RangeInclusive;
use yaah::*;

#[aoc_generator(day4)]
fn gen(input: &'static str) -> Vec<String> {
	input.lines().map(|s| s.to_string()).collect()
}

#[aoc(day4, part1)]
fn solve_part1(stringy_pairs: &[String]) -> usize {
	stringy_pairs
		.iter()
		.map(|s| assignment_pair(s.as_str()))
		.filter(|(l, r)| fully_contains(l.clone(), r.clone()))
		.count()
}

#[aoc(day4, part2)]
fn solve_part2(stringy_pairs: &[String]) -> usize {
	stringy_pairs
		.iter()
		.map(|s| assignment_pair(s.as_str()))
		.filter(|(l, r)| overlap_at_all(l.clone(), r.clone()))
		.count()
}

fn assignment_pair(s: &str) -> (RangeInclusive<u32>, RangeInclusive<u32>) {
	let pair: Vec<RangeInclusive<u32>> = s
		.split(',')
		.map(|s| s.to_string())
		.map(range_from_string)
		.collect();
	(pair[0].clone(), pair[1].clone())
}

pub fn fully_contains(r1: RangeInclusive<u32>, r2: RangeInclusive<u32>) -> bool {
	(r1.contains(r2.start()) && r1.contains(r2.end()))
		|| (r2.contains(r1.start()) && r2.contains(r1.end()))
}

pub fn overlap_at_all(r1: RangeInclusive<u32>, r2: RangeInclusive<u32>) -> bool {
	r1.contains(r2.start())
		|| r1.contains(r2.end())
		|| r2.contains(r1.start())
		|| r2.contains(r1.end())
}

fn range_from_string(s: String) -> RangeInclusive<u32> {
	let r: Vec<u32> = s.split('-').filter_map(|s| s.parse::<u32>().ok()).collect();
	RangeInclusive::new(r[0], r[1])
}

#[cfg(test)]
mod test {
	use crate::day4::{fully_contains, gen, range_from_string, solve_part1, solve_part2};

	const EXAMPLE: &str = "2-4,6-8
2-3,4-5
5-7,7-9
2-8,3-7
6-6,4-6
2-6,4-8";

	#[test]
	fn test_range_pair() {
		let r1 = range_from_string("2-8".to_string());
		let r2 = range_from_string("3-7".to_string());

		assert!(fully_contains(r1, r2));
	}

	/// Some of the pairs have noticed that one of their assignments fully contains the other.
	/// For example, `2-8` fully contains `3-7`, and `6-6` is fully contained by `4-6`.
	/// In pairs where one assignment fully contains the other,
	/// one Elf in the pair would be exclusively cleaning sections their partner will already be cleaning,
	/// so these seem like the most in need of reconsideration. In this example, there are `2` such pairs.
	#[test]
	fn example_part1() {
		assert_eq!(2, solve_part1(&gen(EXAMPLE)));
	}

	/// So, in this example, the number of overlapping assignment pairs is `4`.
	#[test]
	fn example_part2() {
		assert_eq!(4, solve_part2(&gen(EXAMPLE)));
	}
}
