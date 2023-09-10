use nom::character::complete::{i64 as nom_i64, line_ending};
use nom::multi::separated_list1;
use nom::IResult;
use yaah::*;

type Element = (usize, i64);

#[aoc_generator(day20)]
fn read_input(input: &'static str) -> Vec<Element> {
	encryption_file(input)
		.unwrap()
		.1
		.iter()
		.enumerate()
		.map(|(i, &e)| (i, e))
		.collect()
}

#[aoc(day20, part1)]
fn solve_part1(file: &Vec<Element>) -> i64 {
	let mixed = file
		.iter()
		.fold(file.clone(), make_move)
		.iter()
		.map(|e| e.1)
		.collect();
	grove_coordinates(mixed)
}

#[aoc(day20, part2)]
fn solve_part2(input: &Vec<Element>) -> i64 {
	let decryption_key: i64 = 811589153;
	let file: Vec<Element> = input
		.iter()
		.map(|&(i, e)| (i, decryption_key * e))
		.collect();
	let mut elements = file.clone();
	for _ in 0..10 {
		elements = file.iter().fold(elements, make_move)
	}
	let mixed = elements.iter().map(|e| e.1).collect();
	grove_coordinates(mixed)
}

fn make_move(mut elements: Vec<Element>, element: &Element) -> Vec<Element> {
	let position = elements.iter().position(|e| e == element).unwrap();
	elements.remove(position);
	let new_position = (element.1 + (position as i64)).rem_euclid(elements.len() as i64);
	elements.insert(new_position as usize, *element);
	elements
}

/// the grove coordinates can be found by looking at the 1000th, 2000th, and 3000th numbers after the value 0
fn grove_coordinates(numbers: Vec<i64>) -> i64 {
	let start = numbers.iter().position(|&n| n == 0i64).unwrap();
	let size = numbers.len();
	[1000, 2000, 3000]
		.iter()
		.map(|n| numbers[(start + n) % size])
		.sum()
}

fn encryption_file(input: &str) -> IResult<&str, Vec<i64>> {
	separated_list1(line_ending, nom_i64)(input)
}

#[cfg(test)]
mod test {
	use crate::day20::{grove_coordinates, read_input, solve_part1, solve_part2, Element};

	const EXAMPLE: &str = "1
2
-3
3
-2
0
4";

	#[test]
	fn parse_example_input() {
		let expected: Vec<Element> = vec![1, 2, -3, 3, -2, 0, 4]
			.iter()
			.enumerate()
			.map(|(i, &e)| (i, e))
			.collect();
		assert_eq!(read_input(EXAMPLE), expected);
	}

	#[test]
	fn example_coordinates() {
		let numbers = vec![1, 2, -3, 4, 0, 3, -2];
		assert_eq!(grove_coordinates(numbers), 3);
	}

	#[test]
	fn example_part1() {
		assert_eq!(solve_part1(&read_input(EXAMPLE)), 3);
	}

	#[test]
	fn example_part2() {
		assert_eq!(solve_part2(&read_input(EXAMPLE)), 1623178306);
	}
}
