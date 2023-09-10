use std::collections::HashSet;
use yaah::*;

#[aoc(day6, part1)]
fn solve_part1(input: &'static str) -> u32 {
	let range = 0..(input.len() - 4);
	range
		.into_iter()
		.find_map(
			|i| match &input[i..(i + 4)].chars().collect::<HashSet<char>>().len() {
				4 => Some(i as u32 + 4),
				_ => None,
			},
		)
		.unwrap()
}

#[aoc(day6, part2)]
fn solve_part2(input: &'static str) -> u32 {
	let range = 0..(input.len() - 14);
	range
		.into_iter()
		.find_map(
			|i| match &input[i..(i + 14)].chars().collect::<HashSet<char>>().len() {
				14 => Some(i as u32 + 14),
				_ => None,
			},
		)
		.unwrap()
}

#[cfg(test)]
mod test {
	use crate::day6::{solve_part1, solve_part2};

	#[test]
	fn examples() {
		assert_eq!(7, solve_part1("mjqjpqmgbljsphdztnvjfqwrcgsmlb"));
		assert_eq!(5, solve_part1("bvwbjplbgvbhsrlpgdmjqwftvncz"));
		assert_eq!(6, solve_part1("nppdvjthqldpwncqszvftbrmjlhg"));
		assert_eq!(10, solve_part1("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"));
		assert_eq!(11, solve_part1("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"));
	}

	#[test]
	fn examples_part2() {
		assert_eq!(19, solve_part2("mjqjpqmgbljsphdztnvjfqwrcgsmlb"));
		assert_eq!(23, solve_part2("bvwbjplbgvbhsrlpgdmjqwftvncz"));
		assert_eq!(23, solve_part2("nppdvjthqldpwncqszvftbrmjlhg"));
		assert_eq!(29, solve_part2("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"));
		assert_eq!(26, solve_part2("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"));
	}
}
// mjqjpqmgbljsphdztnvjfqwrcgsmlb
// hdztnvjfqwrcgs
