use itertools::Itertools;
use nom::character::complete::{char as nom_char, line_ending, one_of};
use nom::combinator::map;
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;
use petgraph::algo::astar;
use petgraph::graphmap::DiGraphMap;
use std::collections::{HashMap, HashSet};
use yaah::*;

#[aoc_generator(day24)]
fn parse_basin(input: &'static str) -> Basin {
	basin(input).unwrap().1
}

fn exit(input: &str) -> IResult<&str, usize> {
	map(
		tuple((many1(nom_char('#')), nom_char('.'), many1(nom_char('#')))),
		|(walls, _, _)| walls.len() - 1,
	)(input)
}

fn valley_line(input: &str) -> IResult<&str, Vec<char>> {
	delimited(nom_char('#'), many1(one_of(".<>^v")), nom_char('#'))(input)
}

fn valley(input: &str) -> IResult<&str, Valley> {
	separated_list1(line_ending, valley_line)(input)
}

fn basin(input: &str) -> IResult<&str, Basin> {
	map(
		tuple((
			terminated(exit, line_ending),
			valley,
			preceded(line_ending, exit),
		)),
		|(start, valley, end)| Basin::from(start, valley, end),
	)(input)
}

#[aoc(day24, part1)]
fn solve_part1(basin: &Basin) -> i32 {
	let graph = build_graph(basin);

	let start = (0usize, -1, basin.start as i32);
	let end = (basin.height as i32, basin.end as i32);

	walk(&graph, start, end).0
}

#[aoc(day24, part2)]
fn solve_part2(basin: &Basin) -> i32 {
	let graph = build_graph(basin);

	let start = (-1, basin.start as i32);
	let end = (basin.height as i32, basin.end as i32);

	vec![(start, end), (end, start), (start, end)]
		.into_iter()
		.fold(0, |time, (from, to)| {
			time + walk(&graph, position_time(from, time as usize % basin.lcm), to).0
		})
}

fn position_time(position: Position, time: usize) -> PositionT {
	(time, position.0, position.1)
}

fn walk(
	graph: &DiGraphMap<PositionT, ()>,
	start: PositionT,
	end: Position,
) -> (i32, Vec<PositionT>) {
	astar(
		&graph,
		start,
		|(_, row, column)| (row, column) == end,
		|_| 1,
		|pos| manhattan_distance(&pos, &end),
	)
	.unwrap()
}

/// The blizzards operate in a cycle
/// - Horizontally by the width of the valley
/// - Vertically by the height of the valley
/// This means that the patten repeats by the lowest-common-multiple between the two.
///
/// I build a representation of the blizzards at each time in that loop
/// My valley was 120x25 which meant the LCD was 600.
/// That's 1.8 MiB of data which seems reasonable.
///
/// The next step is to build the list of open positions that we can navigate to over time.
/// in the example case there are 134 open positions over time.
///
/// The next step is create a graph of open positions, connecting nodes to their possible next moves.
fn build_graph(basin: &Basin) -> DiGraphMap<PositionT, ()> {
	let valley_sprite: Vec<Valley> = (0..(basin.lcm)).map(|time| basin.valley_at(time)).collect();

	let open_positons: HashSet<PositionT> = HashSet::from_iter(
		valley_sprite
			.iter()
			.enumerate()
			.flat_map(|(t, valley)| basin.open_positions(t, valley)),
	);

	let edges: Vec<(PositionT, PositionT)> = open_positons
		.iter()
		.flat_map(|position| possible_moves(position, &open_positons, basin.lcm))
		.collect();

	DiGraphMap::<PositionT, ()>::from_edges(edges)
}

fn manhattan_distance(from: &PositionT, to: &Position) -> i32 {
	(from.1 - to.0).abs() + (from.2 - to.1).abs()
}

fn possible_moves(
	position: &PositionT,
	open_positions: &HashSet<PositionT>,
	lcm: usize,
) -> Vec<(PositionT, PositionT)> {
	let (time, row, column) = *position;
	let time_next = (time + 1) % lcm;
	vec![(1, 0), (0, 1), (-1, 0), (0, -1), (0, 0)]
		.into_iter()
		.map(|(r1, c1)| (time_next, row + r1, column + c1))
		.filter(|p1| open_positions.contains(p1))
		.map(|p1| (*position, p1))
		.collect()
}

type Position = (i32, i32);
type PositionT = (usize, i32, i32);

#[allow(dead_code)]
fn print_valley(valley: &Valley, basin: &Basin) {
	let start: String = (0..basin.width)
		.map(|n| if n == basin.start { '.' } else { '#' })
		.collect();
	println!("#{start}#");
	valley
		.iter()
		.map(|row| row.iter().collect::<String>())
		.for_each(|line| println!("#{line}#"));
	let end: String = (0..basin.width)
		.map(|n| if n == basin.end { '.' } else { '#' })
		.collect();
	println!("#{end}#");
}

type Index2d = (usize, usize);

type Valley = Vec<Vec<char>>;
type Blizzards = HashMap<Index2d, Direction>;

#[derive(Debug)]
pub struct Basin {
	width: usize,
	height: usize,
	start: usize,
	end: usize,
	lcm: usize,
	blizzards: Blizzards,
}

impl Basin {
	fn from(start: usize, valley: Valley, end: usize) -> Self {
		let width = valley.first().unwrap().len();
		let height = valley.len();
		let lcm = lcm(height, width);
		let blizzards = HashMap::from_iter(
			(0..height)
				.cartesian_product(0..width)
				.map(|(row, column)| (row, column, valley[row][column]))
				.filter_map(|(row, column, c)| match c {
					'>' => Some(((row, column), Direction::Right)),
					'<' => Some(((row, column), Direction::Left)),
					'^' => Some(((row, column), Direction::Up)),
					'v' => Some(((row, column), Direction::Down)),
					_ => None,
				}),
		);
		Basin {
			width,
			height,
			start,
			end,
			lcm,
			blizzards,
		}
	}
	fn move_up(&self, time: usize, (row, column): Index2d) -> Index2d {
		let adjustment = (time + (self.height - row)) % self.height;
		((self.height - adjustment) % self.height, column)
	}
	fn move_down(&self, time: usize, (row, column): Index2d) -> Index2d {
		((time + row) % self.height, column)
	}
	fn move_right(&self, time: usize, (row, column): Index2d) -> Index2d {
		(row, (time + column) % self.width)
	}
	fn move_left(&self, time: usize, (row, column): Index2d) -> Index2d {
		let adjustment = (time + (self.width - column)) % self.width;
		(row, (self.width - adjustment) % self.width)
	}

	fn valley_at(&self, time: usize) -> Valley {
		let mut valley = vec![vec!['.'; self.width]; self.height];
		self.blizzards.iter().for_each(|(position, direction)| {
			let (row, column) = match direction {
				Direction::Up => self.move_up(time, *position),
				Direction::Down => self.move_down(time, *position),
				Direction::Left => self.move_left(time, *position),
				Direction::Right => self.move_right(time, *position),
			};
			valley[row][column] = match valley[row][column] {
				'.' => direction.to_char(),
				c => match c.to_digit(10) {
					Some(digit) => char::from_digit(digit + 1, 10).unwrap(),
					None => '2',
				},
			}
		});
		valley
	}

	fn open_positions(&self, time: usize, valley: &Valley) -> Vec<PositionT> {
		let entrances = vec![
			(-1, self.start as i32),
			(self.height as i32, self.end as i32),
		];
		(0..self.height)
			.cartesian_product(0..self.width)
			.filter_map(|(row, column)| match valley[row][column] {
				'.' => Some((row as i32, column as i32)),
				_ => None,
			})
			.chain(entrances.into_iter())
			.map(|(row, column)| (time, row, column))
			.collect()
	}
}

#[derive(Debug)]
enum Direction {
	Up,
	Down,
	Left,
	Right,
}

impl Direction {
	fn to_char(&self) -> char {
		match self {
			Direction::Up => '^',
			Direction::Down => 'v',
			Direction::Left => '<',
			Direction::Right => '>',
		}
	}
}

impl TryFrom<char> for Direction {
	type Error = ();

	fn try_from(value: char) -> Result<Self, Self::Error> {
		match value {
			'^' => Ok(Direction::Up),
			'v' => Ok(Direction::Down),
			'<' => Ok(Direction::Left),
			'>' => Ok(Direction::Right),
			_ => Err(()),
		}
	}
}

/// https://www.hackertouch.com/least-common-multiple-in-rust.html
/// Lets not waste time reimplementing basics.
fn lcm(first: usize, second: usize) -> usize {
	first * second / gcd(first, second)
}

fn gcd(first: usize, second: usize) -> usize {
	let mut max = first;
	let mut min = second;
	if min > max {
		std::mem::swap(&mut max, &mut min);
	}

	loop {
		let res = max % min;
		if res == 0 {
			return min;
		}

		max = min;
		min = res;
	}
}

#[cfg(test)]
mod test {
	use crate::day24::{basin, exit, solve_part1, solve_part2, valley, valley_line, Index2d};
	use itertools::Itertools;

	const EXAMPLE_1: &str = r"#.#####
#.....#
#>....#
#.....#
#...v.#
#.....#
#####.#";

	const EXAMPLE_2: &str = r"#.######
#>>.<^<#
#.<..<<#
#>v.><>#
#<^v^^>#
######.#";

	#[test]
	fn part1() {
		let basin = basin(EXAMPLE_2).unwrap().1;
		assert_eq!(18, solve_part1(&basin));
	}

	#[test]
	fn part2() {
		let basin = basin(EXAMPLE_2).unwrap().1;
		assert_eq!(54, solve_part2(&basin));
	}

	#[test]
	fn test_move_up() {
		let basin = basin(EXAMPLE_2).unwrap().1;

		let tests: Vec<(usize, Index2d)> = vec![
			(0, (1, 1)),
			(1, (0, 1)),
			(2, (3, 1)),
			(3, (2, 1)),
			(4, (1, 1)),
			(5, (0, 1)),
			(40, (1, 1)),
		];
		let position = (1, 1);

		for (time, expected) in tests {
			assert_eq!(expected, basin.move_up(time, position))
		}
	}

	#[test]
	fn test_move_down() {
		let basin = basin(EXAMPLE_2).unwrap().1;

		let tests: Vec<(usize, Index2d)> = vec![
			(0, (1, 1)),
			(1, (2, 1)),
			(2, (3, 1)),
			(3, (0, 1)),
			(4, (1, 1)),
			(5, (2, 1)),
			(44, (1, 1)),
		];
		let position = (1, 1);

		for (time, expected) in tests {
			assert_eq!(expected, basin.move_down(time, position))
		}
	}

	#[test]
	fn test_move_right() {
		let basin = basin(EXAMPLE_2).unwrap().1;

		let position = (1, 1);
		let tests: Vec<(usize, Index2d)> = vec![
			(0, (1, 1)),
			(1, (1, 2)),
			(2, (1, 3)),
			(3, (1, 4)),
			(4, (1, 5)),
			(5, (1, 0)),
			(6, (1, 1)),
			(66, (1, 1)),
		];

		for (time, expected) in tests {
			assert_eq!(expected, basin.move_right(time, position))
		}
	}

	#[test]
	fn test_move_left() {
		let basin = basin(EXAMPLE_2).unwrap().1;

		let position = (1, 1);
		let tests: Vec<(usize, Index2d)> = vec![
			(0, (1, 1)),
			(1, (1, 0)),
			(2, (1, 5)),
			(3, (1, 4)),
			(4, (1, 3)),
			(5, (1, 2)),
			(6, (1, 1)),
			(66, (1, 1)),
		];

		for (time, expected) in tests {
			assert_eq!(expected, basin.move_left(time, position))
		}
	}

	#[test]
	fn parse_exit() {
		let (tail, start) = exit(EXAMPLE_1).unwrap();
		assert_eq!(start, 0);

		let last_line = tail.lines().last().unwrap();
		let (_, end) = exit(last_line).unwrap();
		assert_eq!(end, 4);
	}

	#[test]
	fn parse_valley_line() {
		let line = "#.....#";
		let expected = vec!['.', '.', '.', '.', '.'];
		assert_eq!(expected, valley_line(line).unwrap().1);
	}

	#[test]
	fn parse_valley() {
		let valley_lines = EXAMPLE_1.lines().skip(1).join("\n");
		let expected = vec![
			vec!['.', '.', '.', '.', '.'],
			vec!['>', '.', '.', '.', '.'],
			vec!['.', '.', '.', '.', '.'],
			vec!['.', '.', '.', 'v', '.'],
			vec!['.', '.', '.', '.', '.'],
		];
		assert_eq!(expected, valley(&valley_lines).unwrap().1);
	}

	#[test]
	fn parse_basin() {
		let result = basin(EXAMPLE_1);
		println!("{:?}", result);
		let (_, basin) = result.unwrap();
		assert_eq!(basin.start, 0);
		assert_eq!(basin.end, 4);

		assert_eq!(basin.width, 5);
		assert_eq!(basin.height, 5);

		assert_eq!(basin.blizzards.len(), 2);
	}
}
