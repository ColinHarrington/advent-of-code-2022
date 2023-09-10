use itertools::Itertools;
use nom::bytes::complete::tag;
use nom::character::complete::{char as nom_char, line_ending, u32 as nom_u32};
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::IResult;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Range;
use std::{cmp, fmt};
use yaah::*;

#[aoc_generator(day14)]
fn generate_structures(input: &'static str) -> Vec<RockStructure> {
	separated_list1(line_ending, rock_structure)(input)
		.unwrap()
		.1
}

#[aoc(day14, part1)]
fn solve_part1(structures: &Vec<RockStructure>) -> u32 {
	let mut cave = Cave::from_structures(structures, false);

	while let Some(grain) = cave.drop_grain() {
		cave.add_grain(grain);
	}

	cave.map.into_iter().filter(|(_, v)| *v == 'o').count() as u32
}

#[aoc(day14, part2)]
fn solve_part2(structures: &Vec<RockStructure>) -> u32 {
	let mut cave = Cave::from_structures(structures, true);

	while cave.is_open(500, 0) {
		let grain = cave.drop_grain_with_floor().unwrap();
		cave.add_grain(grain);
	}
	#[cfg(feature = "stdout")]
	println!("{cave}");

	cave.map.into_iter().filter(|(_, v)| *v == 'o').count() as u32
}

fn map_rocks(structure: &RockStructure) -> Vec<Position> {
	structure
		.0
		.iter()
		.tuple_windows()
		.map(|(a, b)| map_rock_line(a, b))
		.flatten()
		.dedup()
		.collect()
}

fn map_rock_line(from: &Position, to: &Position) -> Vec<Position> {
	let xmin = cmp::min(from.0, to.0);
	let xmax = cmp::max(from.0, to.0) + 1;
	let xrange = xmin..xmax;

	let ymin = cmp::min(from.1, to.1);
	let ymax = cmp::max(from.1, to.1) + 1;
	let yrange = ymin..ymax;

	xrange
		.cartesian_product(yrange)
		.map(|(x, y)| Position(x, y))
		.collect()
}

#[derive(Debug, Eq, PartialEq)]
pub struct Cave {
	map: HashMap<Position, char>,
	floor: Option<u32>,
}

impl Cave {
	fn from_structures(structures: &Vec<RockStructure>, has_floor: bool) -> Self {
		let rocks = structures
			.iter()
			.map(|structure| map_rocks(structure))
			.flatten()
			.dedup()
			.collect::<Vec<Position>>();

		let mut map: HashMap<Position, char> = HashMap::new();

		for rock in rocks {
			map.insert(rock, '#');
		}

		let ymax = map.keys().map(|p| p.1).max().unwrap();

		let floor = match has_floor {
			true => Some(ymax + 2),
			false => None,
		};
		Self { map, floor }
	}

	fn xrange(&self) -> Range<u32> {
		let xmin = self.map.keys().map(|p| p.0).min().unwrap();
		let xmax = self.map.keys().map(|p| p.0).max().unwrap();
		xmin..(xmax + 1)
	}

	fn yrange(&self) -> Range<u32> {
		let ymax = self.map.keys().map(|p| p.1).max().unwrap();
		0..(ymax + 1)
	}

	/// Drops a grain and returns it's final destiny.
	/// Returns Some(Position) where it lands
	/// or None if it's gone to the abyss
	fn drop_grain(&mut self) -> Option<Position> {
		let mut x = 500;
		self.yrange().find_map(|y| match self.move_down(x, y) {
			Some(p) => {
				x = p.0;
				None
			}
			None => Some(Position(x, y)),
		})
	}

	/// Drops a grain, but if the floor is reached, it will return it's resting place.
	fn drop_grain_with_floor(&mut self) -> Option<Position> {
		let mut x = 500;
		let floor = self.floor.unwrap() - 1;
		(0..(floor + 1)).find_map(|y| match self.move_down(x, y) {
			Some(p) if y != floor => {
				x = p.0;
				None
			}
			_ => Some(Position(x, y)),
		})
	}

	fn add_grain(&mut self, grain: Position) {
		self.map.insert(grain, 'o');
	}

	fn is_open(&self, x: u32, y: u32) -> bool {
		self.map.get(&Position(x, y)).is_none()
	}
	fn move_down(&self, x: u32, y: u32) -> Option<Position> {
		vec![(x, y + 1), (x - 1, y + 1), (x + 1, y + 1)]
			.into_iter()
			.map(|(x1, y1)| Position(x1, y1))
			.find(|p| self.map.get(p).is_none())
	}
}

impl fmt::Display for Cave {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let xmin = self.map.keys().map(|p| p.0).min().unwrap();
		let xmax = self.map.keys().map(|p| p.0).max().unwrap();

		let labels: Vec<String> = vec![format!("{xmin}"), "500".to_string(), format!("{xmax}")];
		let label_lines: Vec<String> = (0..(labels.iter().map(|label| label.len()).max().unwrap()))
			.map(|lr| {
				self.xrange()
					.map(|x| match x {
						_ if x == xmin => labels[0].chars().nth(lr).unwrap(),
						_ if x == 500 => labels[1].chars().nth(lr).unwrap(),
						_ if x == xmax => labels[2].chars().nth(lr).unwrap(),
						_ => ' ',
					})
					.collect::<String>()
			})
			.map(|label_line| format!("  {label_line}"))
			.collect::<Vec<String>>();

		let lines: Vec<String> = self
			.yrange()
			.map(|y| {
				format!(
					"{y} {}",
					self.xrange()
						.map(|x| self.map.get(&Position(x, y)).unwrap_or(&'.'))
						.collect::<String>()
				)
			})
			.collect();

		write!(f, "\n{}\n{}", label_lines.join("\n"), lines.join("\n"))
	}
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Position(u32, u32);

#[derive(Debug, Eq, PartialEq)]
pub struct RockStructure(Vec<Position>);

fn position(input: &str) -> IResult<&str, Position> {
	let (input, (x, y)) = separated_pair(nom_u32, nom_char(','), nom_u32)(input)?;
	Ok((input, Position(x, y)))
}

fn rock_structure(input: &str) -> IResult<&str, RockStructure> {
	let (input, points) = separated_list1(tag(" -> "), position)(input)?;
	Ok((input, RockStructure(points)))
}

#[cfg(test)]
mod test {
	use crate::day14::{
		generate_structures, solve_part1, solve_part2, Cave, Position, RockStructure,
	};

	const EXAMPLE: &str = r"498,4 -> 498,6 -> 496,6
503,4 -> 502,4 -> 502,9 -> 494,9";

	#[test]
	fn parse_rocks() {
		let expected = vec![
			RockStructure(vec![Position(498, 4), Position(498, 6), Position(496, 6)]),
			RockStructure(vec![
				Position(503, 4),
				Position(502, 4),
				Position(502, 9),
				Position(494, 9),
			]),
		];

		assert_eq!(expected, generate_structures(EXAMPLE));
	}

	#[test]
	fn cave_display() {
		let structures = generate_structures(EXAMPLE);
		let cave = Cave::from_structures(&structures, false);

		let expected: String = r"
  4     5  5
  9     0  0
  4     0  3
0 ..........
1 ..........
2 ..........
3 ..........
4 ....#...##
5 ....#...#.
6 ..###...#.
7 ........#.
8 ........#.
9 #########."
			.to_string();
		let display = format!("{cave}");
		assert_eq!(expected, display);
	}

	#[test]
	fn part1() {
		assert_eq!(24, solve_part1(&generate_structures(EXAMPLE)));
	}

	#[test]
	fn part2() {
		assert_eq!(93, solve_part2(&generate_structures(EXAMPLE)));
	}
}
