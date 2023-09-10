use nom::branch::alt;
use nom::character::complete::u8 as nom_u8;
use nom::character::complete::{char as nom_char, newline, one_of};
use nom::combinator::map as nom_map;
use nom::multi::{many1, separated_list1};
use nom::sequence::{pair, separated_pair};
use nom::IResult;
use std::str::FromStr;
// use pathfinding::matrix::Matrix;
use yaah::*;

#[aoc(day22, part1)]
fn solve_part1(monkey_map: &MonkeyMap) -> usize {
	let (board, instructions) = monkey_map;

	let result = instructions
		.iter()
		.fold(board.initial_position(), |pos, instruction| {
			execute_instruction(instruction, pos, board)
		});

	result.password()
}

fn execute_instruction(instruction: &Instruction, position: Position, board: &Board) -> Position {
	match instruction {
		Instruction::Rotate(r) => position.rotate(r),
		Instruction::Steps(steps) => position.steps(*steps, board),
	}
}

type MonkeyMap = (Board, Instructions);

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Position {
	facing: Direction,
	row: usize,
	column: usize,
}

impl Position {
	fn rotate(&self, rotation: &Rotation) -> Position {
		Position {
			facing: self.facing.rotate(rotation),
			..*self
		}
	}

	fn password(&self) -> usize {
		1000 * (self.row + 1) + 4 * (self.column + 1) + self.facing.value()
	}

	fn next_step(&self, board: &Board) -> Position {
		match self.facing {
			Direction::Up => self.next_up(board),
			Direction::Down => self.next_down(board),
			Direction::Left => self.next_left(board),
			Direction::Right => self.next_right(board),
		}
	}

	fn next_right(&self, board: &Board) -> Position {
		let column = (self.column + 1) % board.width;
		Position { column, ..*self }
	}

	fn next_left(&self, board: &Board) -> Position {
		let column = match self.column {
			0 => board.width - 1,
			r => r - 1,
		};
		Position { column, ..*self }
	}

	fn next_down(&self, board: &Board) -> Position {
		let row = (self.row + 1) % board.height;
		Position { row, ..*self }
	}

	fn next_up(&self, board: &Board) -> Position {
		let row = match self.row {
			0 => board.height - 1,
			r => r - 1,
		};
		Position { row, ..*self }
	}

	fn steps(&self, steps: usize, board: &Board) -> Position {
		if steps == 0 {
			return *self;
		}
		match self.next(board) {
			None => *self,
			Some(p) => p.steps(steps - 1, board),
		}
	}

	fn next(&self, board: &Board) -> Option<Position> {
		let next = self.next_step(board);
		let tile = board.tile(&next);
		match tile {
			Tile::Open => Some(next),
			Tile::Closed => next.next(board),
			Tile::Wall => None,
		}
	}
}

#[derive(Debug, Eq, PartialEq)]
pub struct Board {
	map: Vec<Vec<char>>,
	width: usize,
	height: usize,
}

impl From<Vec<Vec<char>>> for Board {
	fn from(lines: Vec<Vec<char>>) -> Self {
		let width = lines.iter().map(|line| line.len()).max().unwrap();
		let height = lines.len();
		Board {
			map: lines,
			width,
			height,
		}
	}
}

impl Board {
	fn initial_position(&self) -> Position {
		let first_row = self.map.get(0).unwrap();
		let initial = first_row.iter().position(|&t| t == '.').unwrap();
		Position {
			facing: Direction::Right,
			row: initial / self.width,
			column: initial % self.width,
		}
	}

	fn tile(&self, position: &Position) -> Tile {
		(*self
			.map
			.get(position.row)
			.unwrap()
			.get(position.column)
			.unwrap_or(&' '))
		.try_into()
		.unwrap()
	}
}

pub enum Tile {
	Wall,
	Open,
	Closed,
}

impl TryFrom<char> for Tile {
	type Error = ();

	fn try_from(value: char) -> Result<Self, Self::Error> {
		match value {
			'#' => Ok(Tile::Wall),
			'.' => Ok(Tile::Open),
			' ' => Ok(Tile::Closed),
			_ => Err(()),
		}
	}
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Direction {
	Up,
	Down,
	Left,
	Right,
}

impl Direction {
	fn rotate(&self, rotation: &Rotation) -> Direction {
		match rotation {
			Rotation::Right => match self {
				Direction::Up => Direction::Right,
				Direction::Right => Direction::Down,
				Direction::Down => Direction::Left,
				Direction::Left => Direction::Up,
			},
			Rotation::Left => match self {
				Direction::Up => Direction::Left,
				Direction::Left => Direction::Down,
				Direction::Down => Direction::Right,
				Direction::Right => Direction::Up,
			},
		}
	}

	/// Facing is 0 for right (>), 1 for down (v), 2 for left (<), and 3 for up (^).
	fn value(&self) -> usize {
		match self {
			Direction::Right => 0,
			Direction::Down => 1,
			Direction::Left => 2,
			Direction::Up => 3,
		}
	}
}

#[derive(Debug, Eq, PartialEq)]
pub enum Rotation {
	Right,
	Left,
}

pub type Instructions = Vec<Instruction>;

#[derive(Debug, Eq, PartialEq)]
pub enum Instruction {
	Rotate(Rotation),
	Steps(usize),
}

impl FromStr for Rotation {
	type Err = ();

	fn from_str(input: &str) -> Result<Rotation, Self::Err> {
		match input {
			"R" => Ok(Rotation::Right),
			"L" => Ok(Rotation::Left),
			_ => Err(()),
		}
	}
}
// type Point2D = (usize,usize);
// pub struct CubeFace {
//     grid: Matrix<Point2D>
// }

// #[derive(Debug, Eq, PartialEq)]
// pub enum FaceOrientation {
//     TOP, NORTH, EAST, SOUTH, WEST, BOTTOM
// }

#[aoc_generator(day22)]
fn read_monkey_map(input: &'static str) -> MonkeyMap {
	parse_input(input).unwrap().1
}

fn rotation(input: &str) -> IResult<&str, Instruction> {
	nom_map(
		alt((rotate_right_instruction, rotate_left_instruction)),
		Instruction::Rotate,
	)(input)
}

fn rotate_right_instruction(input: &str) -> IResult<&str, Rotation> {
	nom_map(nom_char('R'), |_| Rotation::Right)(input)
}

fn rotate_left_instruction(input: &str) -> IResult<&str, Rotation> {
	nom_map(nom_char('L'), |_| Rotation::Left)(input)
}

fn steps(input: &str) -> IResult<&str, Instruction> {
	nom_map(nom_u8, |steps| Instruction::Steps(steps as usize))(input)
}

fn instructions(input: &str) -> IResult<&str, Vec<Instruction>> {
	many1(alt((rotation, steps)))(input)
}

fn map_line(input: &str) -> IResult<&str, Vec<char>> {
	many1(one_of(" .#"))(input)
}

fn map_board(input: &str) -> IResult<&str, Board> {
	nom_map(separated_list1(newline, map_line), |lines| {
		Board::from(lines)
	})(input)
}

fn parse_input(input: &str) -> IResult<&str, (Board, Instructions)> {
	separated_pair(map_board, pair(newline, newline), instructions)(input)
}

#[cfg(test)]
mod test {
	use crate::day22::Instruction::{Rotate, Steps};
	use crate::day22::Rotation::{Left, Right};
	use crate::day22::{
		execute_instruction, instructions, parse_input, read_monkey_map, solve_part1, Direction,
		Instruction, Position,
	};
	use itertools::Itertools;
	use std::iter;

	const EXAMPLE: &str = r"        ...#
        .#..
        #...
        ....
...#.......#
........#...
..#....#....
..........#.
        ...#....
        .....#..
        .#......
        ......#.

10R5L5R10L4R5L5";

	fn example_instructions() -> Vec<Instruction> {
		vec![
			Steps(10),
			Rotate(Right),
			Steps(5),
			Rotate(Left),
			Steps(5),
			Rotate(Right),
			Steps(10),
			Rotate(Left),
			Steps(4),
			Rotate(Right),
			Steps(5),
			Rotate(Left),
			Steps(5),
		]
	}

	///
	/**
		```
		>>v#
		.#v.
		#.v.
		..v.
	...#...v..v#
	>>>v...>#.>>
	..#v...#....
	...>>>>v..#.
			...#....
			.....#..
			.#......
			......#.
	```
	 */
	#[test]
	fn example_steps() {
		let (board, instructions) = read_monkey_map(EXAMPLE);

		let expected_positions: Vec<(usize, usize, Direction)> = vec![
			(0, 10, Direction::Right),
			(0, 10, Direction::Down),
			(5, 10, Direction::Down),
			(5, 10, Direction::Right),
			(5, 3, Direction::Right),
			(5, 3, Direction::Down),
			(7, 3, Direction::Down),
			(7, 3, Direction::Right),
			(7, 7, Direction::Right),
			(7, 7, Direction::Down),
			(5, 7, Direction::Down),
			(5, 7, Direction::Right),
			(5, 7, Direction::Right),
		];

		let mut position = board.initial_position();
		assert_eq!(
			position,
			Position {
				row: 0,
				column: 8,
				facing: Direction::Right
			}
		);

		for (instruction, (row, column, facing)) in iter::zip(instructions, expected_positions) {
			let expected = Position {
				facing,
				row,
				column,
			};
			position = execute_instruction(&instruction, position, &board);
			// println!("{:?} == {:?}", expected, position);
			assert_eq!(position, expected);
		}
	}

	/// In the above example, the final row is 6, the final column is 8, and the final facing is 0.
	/// So, the final password is 1000 * 6 + 4 * 8 + 0: 6032.
	#[test]
	fn part1() {
		let monkey_map = read_monkey_map(EXAMPLE);
		assert_eq!(solve_part1(&monkey_map), 6032)
	}

	/// Facing is 0 for right (>), 1 for down (v), 2 for left (<), and 3 for up (^).
	#[test]
	fn facing_value() {
		assert_eq!(Direction::Right.value(), 0);
		assert_eq!(Direction::Down.value(), 1);
		assert_eq!(Direction::Left.value(), 2);
		assert_eq!(Direction::Up.value(), 3);
	}

	/// The final password is the sum of 1000 times the row, 4 times the column, and the facing.
	#[test]
	fn test_final_password() {
		let position = Position {
			facing: Direction::Right,
			row: 5,
			column: 7,
		};
		assert_eq!(position.password(), 6032);
	}

	#[test]
	fn read_input() {
		let (board, instructions) = read_monkey_map(EXAMPLE);
		assert_eq!(instructions, example_instructions());

		let pos = board.initial_position();
		let expected_initial_position = Position {
			row: 0,
			column: 8,
			facing: Direction::Right,
		};
		assert_eq!(pos, expected_initial_position)
	}

	#[test]
	fn parse_example() {
		let (tail, (board, instructions)) = parse_input(EXAMPLE).unwrap();
		assert_eq!("", tail);
		assert_eq!(instructions, example_instructions());
		assert_eq!(board.map.len(), 12);
		assert_eq!(board.map[1], "        .#..".chars().collect_vec());
	}

	#[test]
	fn parse_instructions() {
		let example = EXAMPLE.lines().last().unwrap();
		let (tail, instructions) = instructions(example).unwrap();
		assert_eq!(instructions, example_instructions());
		assert_eq!("", tail);
	}
}
