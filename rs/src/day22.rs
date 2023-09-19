use itertools::Itertools;
use nom::branch::alt;
use nom::character::complete::u8 as nom_u8;
use nom::character::complete::{char as nom_char, newline, one_of};
use nom::combinator::map as nom_map;
use nom::multi::{many1, separated_list1};
use nom::sequence::{pair, separated_pair};
use nom::IResult;
use pathfinding::matrix::Matrix;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;
use yaah::*;

#[aoc(day22, part1)]
fn solve_part1(monkey_map: &MonkeyMap) -> usize {
	let (board, instructions) = monkey_map;

	let result = instructions
		.iter()
		.fold(board.initial_position(), |pos, instruction| {
			board.execute_instruction(instruction, pos)
		});

	result.password()
}

#[aoc(day22, part2)]
pub fn solve_part2(monkey_map: &MonkeyMap) -> usize {
	let (board, instructions) = monkey_map;

	let face_size = gcd(board.height, board.width);

	let cube: Cube = board.cube(face_size);

	instructions
		.iter()
		.fold(
			cube.initial_position(),
			|position, instruction| match instruction {
				Instruction::Rotate(r) => position.rotate(r),
				Instruction::Steps(steps) => cube.steps(position, *steps),
			},
		)
		.password(&cube)
}

type CubeEdges = [FaceEdgeMapping; 6];

const CUBE_EDGES_4X4: CubeEdges = [
	FaceEdgeMapping {
		// 0
		north: (1, FaceRotation::One80),
		east: (5, FaceRotation::One80),
		south: (3, FaceRotation::Same),
		west: (2, FaceRotation::CW),
	},
	FaceEdgeMapping {
		// 1
		north: (0, FaceRotation::One80),
		east: (2, FaceRotation::Same),
		south: (4, FaceRotation::One80),
		west: (5, FaceRotation::CCW),
	},
	FaceEdgeMapping {
		// 2
		north: (0, FaceRotation::CCW),
		east: (3, FaceRotation::Same),
		south: (4, FaceRotation::CW),
		west: (1, FaceRotation::Same),
	},
	FaceEdgeMapping {
		// 3
		north: (0, FaceRotation::Same),
		east: (5, FaceRotation::CCW),
		south: (4, FaceRotation::Same),
		west: (2, FaceRotation::Same),
	},
	FaceEdgeMapping {
		// 4
		north: (3, FaceRotation::Same),
		east: (5, FaceRotation::Same),
		south: (1, FaceRotation::One80),
		west: (2, FaceRotation::CCW),
	},
	FaceEdgeMapping {
		// 5
		north: (3, FaceRotation::CW),
		east: (0, FaceRotation::One80),
		south: (1, FaceRotation::CW),
		west: (4, FaceRotation::Same),
	},
];
const CUBE_EDGES_50X50: CubeEdges = [
	FaceEdgeMapping {
		// 0
		north: (5, FaceRotation::CCW),
		east: (1, FaceRotation::Same),
		south: (2, FaceRotation::Same),
		west: (3, FaceRotation::One80),
	},
	FaceEdgeMapping {
		// 1
		north: (5, FaceRotation::Same),
		east: (4, FaceRotation::One80),
		south: (2, FaceRotation::CCW),
		west: (0, FaceRotation::Same),
	},
	FaceEdgeMapping {
		// 2
		north: (0, FaceRotation::Same),
		east: (1, FaceRotation::CW),
		south: (4, FaceRotation::Same),
		west: (3, FaceRotation::CW),
	},
	FaceEdgeMapping {
		// 3
		north: (2, FaceRotation::CCW),
		east: (4, FaceRotation::Same),
		south: (5, FaceRotation::Same),
		west: (0, FaceRotation::One80),
	},
	FaceEdgeMapping {
		// 4
		north: (2, FaceRotation::Same),
		east: (1, FaceRotation::One80),
		south: (5, FaceRotation::CCW),
		west: (3, FaceRotation::Same),
	},
	FaceEdgeMapping {
		// 5
		north: (3, FaceRotation::Same),
		east: (4, FaceRotation::CW),
		south: (1, FaceRotation::Same),
		west: (0, FaceRotation::CW),
	},
];

fn cube_edges(size: usize) -> CubeEdges {
	match size {
		4 => CUBE_EDGES_4X4,
		50 => CUBE_EDGES_50X50,
		_ => panic!("Not implemented"),
	}
}

type FaceEdgeRotation = (usize, FaceRotation);

#[derive(Debug)]
pub struct FaceEdgeMapping {
	north: FaceEdgeRotation,
	east: FaceEdgeRotation,
	south: FaceEdgeRotation,
	west: FaceEdgeRotation,
}

impl FaceEdgeMapping {
	fn next(&self, direction: Direction) -> FaceEdgeRotation {
		match direction {
			Direction::Up => self.north.clone(),
			Direction::Down => self.south.clone(),
			Direction::Left => self.west.clone(),
			Direction::Right => self.east.clone(),
		}
	}
}

#[derive(Debug, Clone)]
pub enum FaceRotation {
	CW,
	CCW,
	One80,
	Same,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Position3D {
	facing: Direction,
	face: usize,
	row: usize,
	column: usize,
}

/// Not going to reinvent the wheel here.
/// https://rustp.org/number-theory/lcm/
fn gcd(mut a: usize, mut b: usize) -> usize {
	if a == b {
		return a;
	}
	if b > a {
		let temp = a;
		a = b;
		b = temp;
	}
	while b > 0 {
		let temp = a;
		a = b;
		b = temp % b;
	}
	return a;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CubePosition {
	face: usize,
	row: usize,
	column: usize,
	facing: Direction,
}

impl Display for CubePosition {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"[{}:({},{}) {}]",
			self.face, self.row, self.column, self.facing
		)
	}
}

impl CubePosition {
	fn rotate(&self, rotation: &Rotation) -> CubePosition {
		let facing = self.facing.rotate(rotation);
		CubePosition { facing, ..*self }
	}
	fn next_face(&self, cube: &Cube) -> CubePosition {
		let (face, rotation): FaceEdgeRotation = cube.edges[self.face].next(self.facing);
		let (row, column, facing) = self.map_around_edge(rotation.clone(), cube.size.sub(1));
		CubePosition {
			face,
			facing,
			row,
			column,
		}
	}
	fn map_around_edge(&self, rotation: FaceRotation, max: usize) -> (usize, usize, Direction) {
		match (self.facing, rotation) {
			(Direction::Up, FaceRotation::Same) => (max, self.column, Direction::Up),
			(Direction::Up, FaceRotation::CCW) => (self.column, 0, Direction::Right),
			(Direction::Up, FaceRotation::CW) => (max.sub(self.column), max, Direction::Left),
			(Direction::Up, FaceRotation::One80) => (0, max.sub(self.column), Direction::Down),

			(Direction::Down, FaceRotation::Same) => (0, self.column, Direction::Down),
			(Direction::Down, FaceRotation::CCW) => (self.column, max, Direction::Left),
			(Direction::Down, FaceRotation::CW) => (max.sub(self.column), 0, Direction::Right),
			(Direction::Down, FaceRotation::One80) => (max, max.sub(self.column), Direction::Up),

			(Direction::Right, FaceRotation::Same) => (self.row, 0, Direction::Right),
			(Direction::Right, FaceRotation::CCW) => (0, max.sub(self.row), Direction::Down),
			(Direction::Right, FaceRotation::CW) => (max, self.row, Direction::Up),
			(Direction::Right, FaceRotation::One80) => (max.sub(self.row), max, Direction::Left),

			(Direction::Left, FaceRotation::Same) => (self.row, max, Direction::Left),
			(Direction::Left, FaceRotation::CCW) => (max, max.sub(self.row), Direction::Up),
			(Direction::Left, FaceRotation::CW) => (0, self.row, Direction::Down),
			(Direction::Left, FaceRotation::One80) => (max.sub(self.row), 0, Direction::Right),
		}
	}

	fn password(&self, cube: &Cube) -> usize {
		let (row, column) = cube.board_position(self);
		1000 * (row + 1) + 4 * (column + 1) + self.facing.value()
	}
}

#[derive(Debug)]
struct Cube {
	size: usize,
	faces: [Face; 6],
	edges: CubeEdges,
}
impl Cube {
	fn initial_position(&self) -> CubePosition {
		CubePosition {
			face: 0,
			row: 0,
			column: 0,
			facing: Direction::Right,
		}
	}

	fn steps(&self, position: CubePosition, steps: usize) -> CubePosition {
		if steps == 0 {
			position
		} else {
			let next = self.step(position.clone());
			if self.position_open(&next) {
				return self.steps(next, steps.sub(1));
			} else {
				return position;
			}
		}
	}
	fn step(&self, position: CubePosition) -> CubePosition {
		let max = self.size - 1;
		match (position.facing, position.row, position.column) {
			(Direction::Up, 0, _) => position.next_face(self),
			(Direction::Up, _, _) => CubePosition {
				row: position.row.sub(1),
				..position
			},
			(Direction::Down, row, _) if row == max => position.next_face(self),
			(Direction::Down, _, _) => CubePosition {
				row: position.row.add(1),
				..position
			},
			(Direction::Left, _, 0) => position.next_face(self),
			(Direction::Left, _, _) => CubePosition {
				column: position.column.sub(1),
				..position
			},
			(Direction::Right, _, col) if col == max => position.next_face(self),
			(Direction::Right, _, _) => CubePosition {
				column: position.column.add(1),
				..position
			},
		}
	}

	fn position_open(&self, position: &CubePosition) -> bool {
		match self.faces[position.face]
			.data
			.get((position.row, position.column))
		{
			Some(('.', _)) => true,
			_ => false,
		}
	}
	fn board_position(&self, position: &CubePosition) -> Point2D {
		self.faces[position.face]
			.data
			.get((position.row, position.column))
			.unwrap()
			.1
	}
}

type Point2D = (usize, usize);
type TileLocation = (char, Point2D);

#[derive(Debug, Clone)]
struct Face {
	data: Matrix<TileLocation>,
}
impl Display for Face {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let lines = (0..self.data.rows)
			.cartesian_product(0..self.data.columns)
			.into_iter()
			.map(|point| self.data.get(point).unwrap())
			.map(|(tile, _)| tile.to_owned())
			.into_iter()
			.collect_vec()
			.chunks(self.data.columns)
			.into_iter()
			.map(|chunk| chunk.to_vec())
			.map(|chunk| String::from_iter(chunk.into_iter()))
			.collect_vec();
		writeln!(f, "{}", lines.join("\n"))
	}
}

impl Face {
	fn has_tiles(&self) -> bool {
		matches!(self.data.get((0, 0)), Some(('.', _)) | Some(('#', _)))
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

		let map = lines
			.into_iter()
			.map(|line| line.into_iter().pad_using(width, |_| ' ').collect_vec())
			.collect_vec();

		Board { map, width, height }
	}
}

impl Board {
	fn initial_position(&self) -> Position {
		let first_row = self.map.get(0).unwrap();
		let initial = first_row.iter().position(|&t| t == '.').unwrap();
		Position {
			facing: Direction::Right,
			row: 0,
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

	fn faces(&self, size: usize) -> [Face; 6] {
		let grid = self.grid();
		(0..self.height.div(size))
			.cartesian_product(0..self.width.div(size))
			.map(|(row, column)| (row.mul(size), column.mul(size)))
			.map(|(row, column)| {
				grid.slice(row..(row.add(size)), column..(column.add(size)))
					.unwrap()
			})
			.map(|data| Face { data })
			.filter(|face| face.has_tiles())
			.collect_vec()
			.try_into()
			.unwrap_or_else(|faces: Vec<Face>| {
				panic!("Expected a Vec of length 6 but it was {}", faces.len())
			})
	}

	fn grid(&self) -> Matrix<TileLocation> {
		let indexed_grid: Vec<TileLocation> = self
			.map
			.iter()
			.enumerate()
			.flat_map(|(r, row)| {
				row.iter()
					.enumerate()
					.map(move |(c, tile)| (tile.to_owned(), (r, c)))
			})
			.collect_vec();

		Matrix::from_vec(self.height, self.width, indexed_grid).unwrap()
	}
	fn cube(&self, size: usize) -> Cube {
		Cube {
			size,
			faces: self.faces(size),
			edges: cube_edges(size),
		}
	}

	fn execute_instruction(&self, instruction: &Instruction, position: Position) -> Position {
		match instruction {
			Instruction::Rotate(r) => position.rotate(r),
			Instruction::Steps(steps) => position.steps(*steps, self),
		}
	}
}

// pub type Facing3d = (i8, i8, i8);

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
impl Display for Direction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Direction::Up => '^',
				Direction::Down => 'v',
				Direction::Left => '<',
				Direction::Right => '>',
			}
		)
	}
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

impl Display for Rotation {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Rotation::Right => write!(f, "R"),
			Rotation::Left => write!(f, "L"),
		}
	}
}

pub type Instructions = Vec<Instruction>;

#[derive(Debug, Eq, PartialEq)]
pub enum Instruction {
	Rotate(Rotation),
	Steps(usize),
}

impl Display for Instruction {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Instruction::Rotate(rotation) => write!(f, "Rotate({rotation})"),
			Instruction::Steps(steps) => write!(f, "Steps({steps})"),
		}
	}
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
	use crate::day22::Direction::{Down, Left, Right, Up};
	use crate::day22::FaceRotation::{One80, Same, CCW, CW};
	use crate::day22::Instruction::{Rotate, Steps};
	use crate::day22::Rotation;
	use crate::day22::{
		instructions, parse_input, read_monkey_map, solve_part1, solve_part2, Cube, CubePosition,
		Direction, Instruction, Position, Position3D,
	};
	use itertools::{assert_equal, Itertools};
	use std::iter;
	use std::ops::Sub;

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
			Rotate(Rotation::Right),
			Steps(5),
			Rotate(Rotation::Left),
			Steps(5),
			Rotate(Rotation::Right),
			Steps(10),
			Rotate(Rotation::Left),
			Steps(4),
			Rotate(Rotation::Right),
			Steps(5),
			Rotate(Rotation::Left),
			Steps(5),
		]
	}

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
				facing: Direction::Right,
			}
		);

		for (instruction, (row, column, facing)) in iter::zip(instructions, expected_positions) {
			let expected = Position {
				facing,
				row,
				column,
			};
			position = board.execute_instruction(&instruction, position);
			assert_eq!(position, expected);
		}
	}

	#[test]
	fn test_next_face() {
		let (board, instructions) = read_monkey_map(EXAMPLE);
		let cube = board.cube(4);

		let max = cube.size.sub(1);
		let tests = vec![
			((Up, Same, 0usize, 0usize), (max, 0usize, Up)),
			((Up, Same, 0, 3), (3, 3, Up)),
			((Up, CCW, 0, 0), (0, 0, Right)),
			((Up, CCW, 0, 3), (3, 0, Right)),
			((Up, CW, 0, 0), (3, 3, Left)),
			((Up, CW, 0, 3), (0, 3, Left)),
			((Up, One80, 0, 0), (0, 3, Down)),
			((Up, One80, 0, 3), (0, 0, Down)),
			((Down, Same, 3, 0), (0, 0, Down)),
			((Down, Same, 3, 3), (0, 3, Down)),
			((Down, CCW, 3, 0), (0, 3, Left)),
			((Down, CCW, 3, 3), (3, 3, Left)),
			((Down, CW, 3, 0), (3, 0, Right)),
			((Down, CW, 3, 3), (0, 0, Right)),
			((Down, One80, 3, 0), (3, 3, Up)),
			((Down, One80, 3, 3), (3, 0, Up)),
			((Right, Same, 0, 3), (0, 0, Right)),
			((Right, Same, 3, 3), (3, 0, Right)),
			((Right, CCW, 0, 3), (0, 3, Down)),
			((Right, CCW, 3, 3), (0, 0, Down)),
			((Right, CW, 0, 3), (3, 0, Up)),
			((Right, CW, 3, 3), (3, 3, Up)),
			((Right, One80, 0, 3), (3, 3, Left)),
			((Right, One80, 3, 3), (0, 3, Left)),
			((Left, Same, 0, 0), (0, 3, Left)),
			((Left, Same, 3, 0), (3, 3, Left)),
			((Left, CCW, 0, 0), (3, 3, Up)),
			((Left, CCW, 3, 0), (3, 0, Up)),
			((Left, CW, 0, 0), (0, 0, Down)),
			((Left, CW, 3, 0), (0, 3, Down)),
			((Left, One80, 0, 0), (3, 0, Right)),
			((Left, One80, 3, 0), (0, 0, Right)),
		];
		for ((facing, rotation, row, column), expected) in tests {
			let position = CubePosition {
				face: 1,
				facing,
				row,
				column,
			};
			assert_eq!(expected, position.map_around_edge(rotation, max))
		}
	}
	#[test]
	fn example_steps_part2() {
		let (board, instructions) = read_monkey_map(EXAMPLE);
		let cube: Cube = board.cube(4);

		let expected_positions: Vec<(usize, usize, usize, Direction)> = vec![
			(0, 0, 2, Right),
			(0, 0, 2, Down),
			(3, 1, 2, Down),
			(3, 1, 2, Right),
			(5, 2, 2, Down),
			(5, 2, 2, Left),
			(4, 2, 2, Left),
			(4, 2, 2, Down),
			(1, 1, 1, Up),
			(1, 1, 1, Right),
			(2, 1, 2, Right),
			(2, 1, 2, Up),
			(2, 0, 2, Up),
			(2, 0, 2, Up),
		];

		let mut position = cube.initial_position();
		assert_eq!(
			position,
			CubePosition {
				face: 0,
				row: 0,
				column: 0,
				facing: Direction::Right,
			}
		);

		for (instruction, (step, (face, row, column, facing))) in
			iter::zip(instructions, expected_positions.into_iter().enumerate())
		{
			let expected = CubePosition {
				face,
				facing,
				row,
				column,
			};
			println!("step:{step} | {position} => {instruction}");
			position = cube.execute_instruction(position, &instruction);
			println!("step:{step} | {expected} == {position}");
			assert_eq!(position, expected);
		}
		assert_eq!(5031, position.password(&cube));
	}

	/// In the above example, the final row is 6, the final column is 8, and the final facing is 0.
	/// So, the final password is 1000 * 6 + 4 * 8 + 0: 6032.
	#[test]
	fn part1() {
		let monkey_map = read_monkey_map(EXAMPLE);
		assert_eq!(solve_part1(&monkey_map), 6032)
	}

	#[test]
	fn part2() {
		let monkey_map = read_monkey_map(EXAMPLE);
		assert_eq!(solve_part2(&monkey_map), 5031)
	}

	/// Facing is 0 for right (>), 1 for down (v), 2 for left (<), and 3 for up (^).
	#[test]
	fn facing_value() {
		assert_eq!(Right.value(), 0);
		assert_eq!(Down.value(), 1);
		assert_eq!(Left.value(), 2);
		assert_eq!(Up.value(), 3);
	}

	/// The final password is the sum of 1000 times the row, 4 times the column, and the facing.
	#[test]
	fn test_final_password() {
		let position = Position {
			facing: Right,
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
			facing: Right,
		};
		assert_eq!(pos, expected_initial_position)
	}

	#[test]
	fn parse_example() {
		let (tail, (board, instructions)) = parse_input(EXAMPLE).unwrap();
		assert_eq!("", tail);
		assert_eq!(instructions, example_instructions());
		assert_eq!(board.map.len(), 12);
		assert_eq!(board.map[1], "        .#..    ".chars().collect_vec());
	}

	#[test]
	fn parse_instructions() {
		let example = EXAMPLE.lines().last().unwrap();
		let (tail, instructions) = instructions(example).unwrap();
		assert_eq!(instructions, example_instructions());
		assert_eq!("", tail);
	}
}
