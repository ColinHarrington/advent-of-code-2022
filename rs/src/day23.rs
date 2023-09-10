use itertools::Itertools;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{Display, Formatter};
use std::ops::RangeInclusive;
use yaah::*;

#[aoc_generator(day23)]
fn elf_map(input: &'static str) -> Vec<Elf> {
	input
		.lines()
		.enumerate()
		.map(|(row, line)| {
			line.chars()
				.enumerate()
				.filter_map(|(column, c)| match c {
					'#' => Some(Elf::from((row, column))),
					_ => None,
				})
				.collect::<Vec<Elf>>()
		})
		.flatten()
		.collect::<Vec<Elf>>()
}

#[aoc(day23, part1)]
fn solve_part1(elves: &Vec<Elf>) -> i32 {
	let rounds = 10;
	let mut grove = Grove {
		field: HashSet::from_iter(elves.clone().into_iter()),
	};

	let mut directions: VecDeque<Direction> = [
		Direction::North,
		Direction::South,
		Direction::West,
		Direction::East,
	]
	.into();

	(1..=rounds).into_iter().for_each(|_| {
		grove
			.execute_round(&directions.clone())
			.into_iter()
			.for_each(|(new, old)| grove.swap(new, old));

		directions.rotate_left(1);
	});

	grove.empty_tiles()
}

#[aoc(day23, part2)]
fn solve_part2(elves: &Vec<Elf>) -> i32 {
	let mut grove = Grove {
		field: HashSet::from_iter(elves.clone().into_iter()),
	};

	let mut directions: VecDeque<Direction> = [
		Direction::North,
		Direction::South,
		Direction::West,
		Direction::East,
	]
	.into();

	for round in 1.. {
		let swaps = grove.execute_round(&directions.clone());
		if swaps.is_empty() {
			return round;
		}
		swaps
			.into_iter()
			.for_each(|(new, old)| grove.swap(new, old));

		directions.rotate_left(1);
	}
	-1
}

#[derive(Clone, Eq, Ord, PartialOrd, PartialEq, Hash, Debug)]
pub struct Elf {
	row: i32,
	column: i32,
}

impl From<(usize, usize)> for Elf {
	fn from((row, column): (usize, usize)) -> Self {
		Elf {
			row: row as i32,
			column: column as i32,
		}
	}
}

impl Display for Elf {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl Elf {
	/// During the first half of each round, each Elf considers the eight positions adjacent to themself.
	/// If no other Elves are in one of those eight positions, the Elf does not do anything during this round.
	/// Otherwise, the Elf looks in each of four directions in the following order and proposes moving one step in the first valid direction:
	///
	/// - If there is no Elf in the N, NE, or NW adjacent positions, the Elf proposes moving north one step.
	/// - If there is no Elf in the S, SE, or SW adjacent positions, the Elf proposes moving south one step.
	/// - If there is no Elf in the W, NW, or SW adjacent positions, the Elf proposes moving west one step.
	/// - If there is no Elf in the E, NE, or SE adjacent positions, the Elf proposes moving east one step.
	fn propose_move(self: &Self, order: &VecDeque<Direction>, grove: &HashSet<Elf>) -> Option<Elf> {
		//Maybe a bitmask translation?
		let neighbors: Vec<Elf> = vec![
			(-1, -1),
			(-1, 0),
			(-1, 1),
			(0, -1),
			(0, 1),
			(1, -1),
			(1, 0),
			(1, 1),
		]
		.into_iter()
		.map(|t| transform(self, t))
		.collect();
		let mask = neighbors
			.iter()
			.enumerate()
			.filter(|(_, e)| grove.contains(e))
			.map(|(i, _)| 1 << (i as u8))
			.fold(0u8, |acc, v| acc | v);
		if mask == 0u8 {
			return None;
		}
		order
			.iter()
			.find(|&d| (mask & direction_mask(d.clone())) == 0u8)
			.map(|direction| match *direction {
				Direction::North => (-1, 0),
				Direction::East => (0, 1),
				Direction::South => (1, 0),
				Direction::West => (0, -1),
			})
			.map(|t| transform(self, t))
	}
}

fn transform(elf: &Elf, (r, c): (i32, i32)) -> Elf {
	Elf {
		row: elf.row + r,
		column: elf.column + c,
	}
}

fn direction_mask(direction: Direction) -> u8 {
	match direction {
		Direction::North => 0b00000111,
		Direction::East => 0b10010100,
		Direction::South => 0b11100000,
		Direction::West => 0b00101001,
	}
}

#[derive(Clone, Debug)]
enum Direction {
	North,
	South,
	East,
	West,
}

impl Display for Direction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

pub struct Grove {
	field: HashSet<Elf>,
}

impl Display for Grove {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let (row_range, column_range) = self.range();
		let lines = row_range
			.map(|row| {
				column_range
					.clone()
					.map(|column| Elf { row, column })
					.map(|elf| self.field.get(&elf))
					.map(|elf| match elf {
						Some(_) => '#',
						None => '.',
					})
					.join("")
			})
			.join("\n");
		writeln!(f, "{lines}")
	}
}

impl Grove {
	fn elf_count(self: &Self) -> usize {
		self.field.len()
	}

	fn range(self: &Self) -> (RangeInclusive<i32>, RangeInclusive<i32>) {
		(
			RangeInclusive::new(
				self.field.iter().map(|e| e.row).min().unwrap(),
				self.field.iter().map(|e| e.row).max().unwrap(),
			),
			RangeInclusive::new(
				self.field.iter().map(|e| e.column).min().unwrap(),
				self.field.iter().map(|e| e.column).max().unwrap(),
			),
		)
	}

	fn empty_tiles(self: &Self) -> i32 {
		let (row_range, column_range) = self.range();
		// dbg!(&row_range,&column_range);
		(row_range.end() + 1 - row_range.start()) * (column_range.end() + 1 - column_range.start())
			- (self.elf_count() as i32)
	}

	fn execute_round(&mut self, operations: &VecDeque<Direction>) -> HashMap<Elf, Elf> {
		let mut proposal_map: HashMap<Elf, Vec<Elf>> = HashMap::new();

		self.field
			.iter()
			.map(|e| (e.propose_move(&operations.clone(), &self.field), e.clone()))
			.filter(|(proposed, _)| proposed.is_some())
			.map(|(proposed, elf)| (proposed.unwrap(), elf))
			.for_each(|(proposed, elf)| {
				proposal_map.entry(proposed).or_default().push(elf.clone())
			});
		proposal_map.retain(|_, proposers| proposers.len() == 1);
		proposal_map
			.into_iter()
			.map(|(new, proposers)| (new, proposers.first().unwrap().clone()))
			.collect()
	}

	fn swap(&mut self, new: Elf, old: Elf) {
		self.field.insert(new);
		self.field.remove(&old);
	}
}

///
/// Grove => hashset of points.
///
/// Elf could just be the tuple of an entry in a hashset?
///
/// Elf = Point2D(x,y)
///
/// Open ground = area - number of elves
/// 12*11 - 22 = 110 in the example
///
/// Cycle of precedence {North, South, West, East}
/// loop through each 'elf'
///
///
#[cfg(test)]
mod test {
	use crate::day23::{elf_map, solve_part1, solve_part2, Elf, Grove};
	use std::collections::HashSet;

	const EXAMPLE: &str = r"....#..
..###.#
#...#.#
.#...##
#.###..
##.#.##
.#..#..";
	const SMALLER_EXAMPLE: &str = r".....
..##.
..#..
.....
..##.
.....";

	#[test]
	fn part1() {
		let elves = elf_map(EXAMPLE);
		assert_eq!(110, solve_part1(&elves));
	}

	#[test]
	fn part2() {
		let elves = elf_map(EXAMPLE);
		assert_eq!(20, solve_part2(&elves));
	}

	#[test]
	fn small_batch() {
		let elves = elf_map(SMALLER_EXAMPLE);
		assert_eq!(25, solve_part1(&elves));
	}

	#[test]
	fn parse_input() {
		let elves = elf_map(EXAMPLE);
		// dbg!(&elves);
		assert_eq!(22, elves.len())
	}

	#[test]
	fn grove_bounds() {
		let stringy_grove = r".......#......
...........#..
..#.#..#......
......#.......
...#.....#..#.
.#......##....
.....##.......
..#........#..
....#.#..#....
..............
....#..#..#...
..............";
		let elves = elf_map(stringy_grove);
		assert_eq!(22, elves.len());

		let field: HashSet<Elf> = HashSet::from_iter(elves.clone().into_iter());
		let grove = Grove { field };
		assert_eq!(grove.range(), (0..=10, 1..=12));

		assert_eq!(110, grove.empty_tiles())
	}
}
