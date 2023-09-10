use itertools::Itertools;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::iter::Cycle;
use std::str::Chars;
use std::{cmp, fmt, iter};
use yaah::*;

#[aoc(day17, part1)]
fn solve_part1(jet_pattern: &'static str) -> usize {
	let mut chamber = Chamber::default();

	let mut shapes = vec![Shape::Minus, Shape::Plus, Shape::L, Shape::I, Shape::Box]
		.into_iter()
		.cycle();
	let mut jet_cycle = jet_pattern.trim().chars().cycle();

	for _ in 0..2022 {
		chamber.drop_shape(jet_cycle.borrow_mut(), &shapes.next().unwrap(), false);
	}
	chamber.height()
}

#[aoc(day17, part2)]
fn solve_part2(jet_pattern: &'static str) -> u64 {
	let mut chamber = Chamber::default();

	let mut shapes = vec![Shape::Minus, Shape::Plus, Shape::L, Shape::I, Shape::Box]
		.into_iter()
		.cycle();
	let mut jet_cycle = jet_pattern.trim().chars().cycle();

	let jet_pattern_size = jet_pattern.len();
	let rocks: u64 = (jet_pattern_size * 3) as u64;

	let heights: Vec<u64> = (0u64..rocks)
		.map(|_| {
			chamber.drop_shape(jet_cycle.borrow_mut(), &shapes.next().unwrap(), false);
			chamber.height() as u64
		})
		.collect();

	let height_map: HashMap<u64, u64> = HashMap::from_iter(
		heights
			.iter()
			.enumerate()
			.map(|(i, &height)| (i + 1, height))
			.map(|(rock, height)| (rock as u64, height)),
	);

	let (init_height, pattern_height) = chamber.identify_cyclic_pattern().unwrap();

	let init_steps = height_map
		.iter()
		.filter(|(_, &height)| height <= init_height)
		.map(|(&rock, _)| rock)
		.max()
		.unwrap();

	let pattern_steps = height_map
		.iter()
		.filter(|(_, &height)| height == (pattern_height + init_height))
		.map(|(&rock, _)| rock)
		.max()
		.unwrap()
		- init_steps;

	let cycle_space = 1_000_000_000_000u64 - init_steps;
	let cycles = cycle_space / pattern_steps;

	let extra_steps = cycle_space % pattern_steps;
	let extra_height_position = init_steps + extra_steps;
	let extra_height = height_map.get(&extra_height_position).unwrap() - init_height;

	cycles * pattern_height + init_height + extra_height
}

fn match_distance(matches: &[usize]) -> Option<usize> {
	let distances: Vec<usize> = matches
		.iter()
		.tuple_windows()
		.map(|(left, right)| right - left)
		.unique()
		.collect();
	match distances.len() {
		1 => Some(distances[0]),
		_ => None,
	}
}

#[derive(Debug, Eq, PartialEq, Default)]
struct Chamber {
	grid: Vec<Vec<char>>,
}

impl fmt::Display for Chamber {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let lines = self
			.grid
			.iter()
			.map(|row| row.iter().collect::<String>())
			.map(|row| format!("|{row}|"))
			.rev()
			.chain(iter::once("+-------+".to_string()))
			.join("\n");
		write!(f, "{lines}")
	}
}

impl Chamber {
	fn height(&self) -> usize {
		self.grid
			.iter()
			.enumerate()
			.rev()
			.find(|(_, row)| row.contains(&'#'))
			.map(|(row, _)| row + 1)
			.unwrap_or(0)
	}

	fn extend_height(&mut self, extension: usize) {
		let height = self.height() + extension;
		// println!("Height: {height} = {} + {extension}", self.height());
		while self.grid.len() <= height {
			self.grid.push(".......".chars().collect());
		}
	}

	fn drop_shape(&mut self, jets: &mut Cycle<Chars>, shape: &Shape, debug: bool) {
		self.extend_height(3 + shape.height());
		let mut position = self.initial_position(shape);
		let sprite = shape.sprite();

		loop {
			if debug {
				self.draw_progress(shape, position);
			}
			position = self.jet(jets.next().unwrap(), &sprite, position);
			// self.draw_progress(shape, position);
			if self.at_rest(&sprite, position) {
				break;
			}
			position = self.fall(&sprite, position);
		}
		self.draw(shape, position);
	}

	fn initial_position(&self, shape: &Shape) -> (usize, usize) {
		match self.height() {
			0 => (shape.height() + 2, 2usize),
			_ => (self.height() + 2 + shape.height(), 2usize),
		}
	}
	fn at_rest(&self, sprite: &Vec<Vec<char>>, position: (usize, usize)) -> bool {
		let (row, column) = position;
		if row <= (sprite.len() - 1) {
			true
		} else {
			!self.available(sprite, (row - 1, column))
		}
	}

	fn available(&self, sprite: &Vec<Vec<char>>, position: (usize, usize)) -> bool {
		let (grid_row, grid_column) = position;
		grid_row >= (sprite.len() - 1)
			&& !sprite.iter().enumerate().any(|(row, line)| {
				line.iter()
					.enumerate()
					.filter(|(_, &ch)| ch == '#')
					.map(|(column, &ch)| (ch, self.grid[grid_row - row][grid_column + column]))
					.any(|(left, right)| left == right)
			})
	}

	fn fall(&self, sprite: &Vec<Vec<char>>, position: (usize, usize)) -> (usize, usize) {
		if position.0 == 0 {
			position
		} else {
			let new_position = (position.0 - 1, position.1);
			match self.available(sprite, new_position) {
				true => new_position,
				false => position,
			}
		}
	}

	fn jet(
		&self,
		direction: char,
		sprite: &Vec<Vec<char>>,
		position: (usize, usize),
	) -> (usize, usize) {
		let column = match direction {
			'>' => position.1 as i32 + 1,
			'<' => position.1 as i32 - 1,
			_ => panic!(),
		};

		let bounded = cmp::max(0, cmp::min(column, 7i32 - (sprite[0].len() as i32)));
		let new_position = (position.0, bounded as usize);
		if self.available(sprite, new_position) {
			new_position
		} else {
			position
		}
	}

	fn draw(&mut self, shape: &Shape, position: (usize, usize)) {
		let (row, column) = position;

		for (r, line) in shape.sprite().iter().enumerate() {
			line.iter()
				.enumerate()
				.filter(|(_, &ch)| ch == '#')
				.for_each(|(c, ch)| self.grid[row - r][column + c] = *ch)
		}
	}

	fn draw_progress(&mut self, shape: &Shape, position: (usize, usize)) {
		let (row, column) = position;

		for (r, line) in shape.sprite().iter().enumerate() {
			line.iter()
				.enumerate()
				.filter(|(_, &ch)| ch == '#')
				.for_each(|(c, _)| self.grid[row - r][column + c] = '@')
		}
		println!("{self}");

		for (r, line) in shape.sprite().iter().enumerate() {
			line.iter()
				.enumerate()
				.filter(|(_, &ch)| ch == '#')
				.for_each(|(c, _)| self.grid[row - r][column + c] = '.')
		}
	}

	fn matches(&self, pattern: &[Vec<char>]) -> Vec<usize> {
		self.grid
			.windows(pattern.len())
			.enumerate()
			.filter_map(|(i, window)| match window == pattern {
				true => Some(i),
				false => None,
			})
			.collect()
	}

	fn cyclic_pattern(&self, pattern: &[Vec<char>]) -> Option<(usize, usize)> {
		let matches = self.matches(pattern);
		match_distance(&matches).map(|dist| (matches[0], dist))
	}

	fn identify_cyclic_pattern(&self) -> Option<(u64, u64)> {
		self.grid
			.windows(16)
			.filter_map(|window| self.cyclic_pattern(window))
			.map(|(init, pattern_size)| init..(init + pattern_size))
			.map(|range| &self.grid[range])
			.find_map(|pattern| {
				self.cyclic_pattern(pattern)
					.map(|(init, dist)| (init as u64, dist as u64))
			})
	}
}

#[derive(Clone)]
enum Shape {
	Minus,
	Plus,
	L,
	I,
	Box,
}

impl Shape {
	fn height(&self) -> usize {
		self.sprite().len()
	}

	fn sprite(&self) -> Vec<Vec<char>> {
		match self {
			Shape::Minus => vec!["####"],
			Shape::Plus => vec![".#.", "###", ".#."],
			Shape::L => vec!["..#", "..#", "###"],
			Shape::I => vec!["#", "#", "#", "#"],
			Shape::Box => vec!["##", "##"],
		}
		.into_iter()
		.map(|s| s.chars().collect())
		.collect()
	}
}

#[cfg(test)]
mod test {
	use crate::day17::{solve_part1, solve_part2, Chamber};

	const EXAMPLE: &str = r">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>";

	#[test]
	fn part1() {
		assert_eq!(3068, solve_part1(EXAMPLE));
	}

	#[test]
	fn part2() {
		assert_eq!(1514285714288, solve_part2(EXAMPLE));
	}

	#[test]
	fn chamber_height() {
		let empty = Chamber { grid: vec![] };
		assert_eq!(0, empty.height());

		let small_grid: Vec<Vec<char>> = vec!["..####.".chars().collect()];
		let small = Chamber { grid: small_grid };
		assert_eq!(1, small.height());

		let grid: Vec<Vec<char>> = vec![
			"..####.", "...#...", "..###..", "####...", "..#....", "..#....",
		]
		.iter()
		.map(|s| s.chars().collect())
		.collect();

		let chamber = Chamber { grid };
		assert_eq!(6, chamber.height());
	}

	#[test]
	fn chamber_display_empty() {
		let chamber = Chamber { grid: vec![] };
		assert_eq!("+-------+", format!("{chamber}"));
	}

	#[test]
	fn chamber_display_simple() {
		let grid: Vec<Vec<char>> = vec!["..####.".chars().collect()];
		let chamber = Chamber { grid };
		let expected = r"|..####.|
+-------+";
		assert_eq!(expected, format!("{chamber}"));
	}

	#[test]
	fn chamber_display_example() {
		let grid: Vec<Vec<char>> = vec![
			"..####.", "...#...", "..###..", "####...", "..#....", "..#....",
		]
		.iter()
		.map(|s| s.chars().collect())
		.collect();

		let chamber = Chamber { grid };
		let expected = r"|..#....|
|..#....|
|####...|
|..###..|
|...#...|
|..####.|
+-------+";
		println!("{chamber}");
		assert_eq!(expected, format!("{chamber}"));
	}

	#[test]
	fn test_slice_matching() {
		let grid: Vec<Vec<char>> = vec![
			"..####.", "...#...", "..###..", "####...", "..#....", "..#....", "..####.", "...#...",
			"..###..", "####...", "..#....", "..#....", "..####.", "...#...", "..###..", "####...",
			"..#....", "..#....",
		]
		.iter()
		.map(|s| s.chars().collect())
		.collect();

		let chamber = Chamber { grid };

		let pattern1: &[Vec<char>] = &chamber.grid[4..=5];
		assert_eq!(vec![4, 10, 16], chamber.matches(pattern1));

		let pattern2: &[Vec<char>] = &chamber.grid[4..5];
		assert_eq!(vec![4, 5, 10, 11, 16, 17], chamber.matches(pattern2));

		let pattern3: &[Vec<char>] = &chamber.grid[0..5];
		assert_eq!(vec![0, 6, 12], chamber.matches(pattern3));
	}
}
