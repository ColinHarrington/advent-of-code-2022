use nom::character::complete::{char, line_ending, one_of, u8 as nom_u8};
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::IResult;
use std::collections::HashSet;
use yaah::*;

#[aoc_generator(day9)]
fn gen(input: &'static str) -> Vec<Motion> {
	parse_motions(input).unwrap().1
}

#[aoc(day9, part1)]
fn solve_part1(motions: &Vec<Motion>) -> u32 {
	let mut rope = Rope::knots(vec![(0, 0), (0, 0)]);

	for motion in motions {
		rope.advance(motion);
	}
	#[cfg(feature = "debug")]
	rope.print_history();
	rope.history.len() as u32
}

#[aoc(day9, part2)]
fn solve_part2(motions: &Vec<Motion>) -> u32 {
	let mut rope = Rope::knots(vec![
		(0, 0),
		(0, 0),
		(0, 0),
		(0, 0),
		(0, 0),
		(0, 0),
		(0, 0),
		(0, 0),
		(0, 0),
		(0, 0),
	]);

	for motion in motions {
		rope.advance(motion);
	}
	#[cfg(feature = "debug")]
	rope.print_history();
	rope.history.len() as u32
}

#[derive(Debug, PartialEq)]
pub enum Motion {
	Up(u8),
	Down(u8),
	Left(u8),
	Right(u8),
}

type Position = (i32, i32);

#[derive(Debug, PartialEq)]
struct Rope {
	knots: Vec<Position>,
	history: HashSet<Position>,
}

impl Rope {
	pub fn knots(knots: Vec<Position>) -> Self {
		Self {
			knots,
			history: HashSet::from([(0, 0)]),
		}
	}

	fn advance(&mut self, motion: &Motion) {
		let step = translation_step(motion);

		let steps = match motion {
			Motion::Up(steps) => *steps as usize,
			Motion::Down(steps) => *steps as usize,
			Motion::Left(steps) => *steps as usize,
			Motion::Right(steps) => *steps as usize,
		};

		(0..steps).for_each(|_| self.execute_step(step));
	}

	fn execute_step(&mut self, step: Position) {
		for i in 0..(self.knots.len()) {
			let knot = self.knots[i];
			if i == 0 {
				self.knots[i] = translate(knot, step);
			} else {
				let previous = self.knots[i - 1];
				if let Some(new_position) = tail_move(previous, knot) {
					self.knots[i] = new_position;
					if i == self.knots.len() - 1 {
						self.history.insert(new_position);
					}
				}
			}
		}
		#[cfg(feature = "debug")]
		self.print();
	}

	/// Attempted to print the state of the Knots in the format given
	#[cfg(feature = "debug")]
	fn print(&self) {
		let head = self.knots[0];
		let tail = self.knots.last().unwrap();

		println!("=== H{:?} T{:?} ===", head, tail);
		let lines: Vec<String> = (-5..18)
			.into_iter()
			.map(|y| {
				(-11..15)
					.into_iter()
					.map(|x| match (x, y) {
						(x, y) if self.knots.contains(&(x, y)) => {
							match self.knots.iter().enumerate().find_map(|(i, knot)| {
								match (x, y) == *knot {
									true => Some(i),
									false => None,
								}
							}) {
								Some(i) => match i {
									0 => 'H',
									n if n == self.knots.len() - 1 => 'T',
									_ => (i).to_string().chars().next().unwrap(),
								},
								None => ' ',
							}
						}
						(0, 0) => 's',
						_ => '.',
					})
					.collect::<String>()
			})
			.rev()
			.collect();

		for line in lines {
			println!("{line}");
		}
		println!("");
	}

	#[cfg(feature = "debug")]
	fn print_history(&self) {
		let xs: Vec<i32> = self.history.iter().map(|p| p.0).collect();
		let xmin = *xs.iter().min().unwrap() - 1;
		let xmax = *xs.iter().max().unwrap() + 2;

		let ys: Vec<i32> = self.history.iter().map(|p| p.1).collect();
		let ymin = *ys.iter().min().unwrap() - 1;
		let ymax = *ys.iter().max().unwrap() + 2;

		(ymin..ymax)
			.into_iter()
			.map(|y| {
				(xmin..xmax)
					.into_iter()
					.map(|x| match (x, y) {
						(0, 0) => 's',
						(x, y) if self.history.contains(&(x, y)) => '#',
						_ => '.',
					})
					.collect::<String>()
			})
			.rev()
			.for_each(|line| println!("{line}"))
	}
}

fn tail_move(head: Position, tail: Position) -> Option<Position> {
	tail_movement(head, tail).map(|position| translate(tail, position))
}

fn tail_movement(head: Position, tail: Position) -> Option<Position> {
	let diff = diff(tail, head);
	match (diff.0.abs(), diff.1.abs()) {
		(x, y) if x <= 1 && y <= 1 => None,
		(1, 2) => Some((diff.0, diff.1 / 2)),
		(2, 1) => Some((diff.0 / 2, diff.1)),
		_ => Some((diff.0 / 2, diff.1 / 2)),
	}
}

fn diff(a: Position, b: Position) -> Position {
	(b.0 - a.0, b.1 - a.1)
}

fn translate(position: Position, translation: Position) -> Position {
	(position.0 + translation.0, position.1 + translation.1)
}

fn translation_step(motion: &Motion) -> Position {
	match motion {
		Motion::Up(_) => (0, 1),
		Motion::Down(_) => (0, -1),
		Motion::Left(_) => (-1, 0),
		Motion::Right(_) => (1, 0),
	}
}

fn parse_motions(input: &str) -> IResult<&str, Vec<Motion>> {
	separated_list1(line_ending, parse_motion)(input)
}

fn parse_motion(input: &str) -> IResult<&str, Motion> {
	let (input, (direction, steps)) = separated_pair(one_of("UDLR"), char(' '), nom_u8)(input)?;
	match direction {
		'U' => Ok((input, Motion::Up(steps))),
		'D' => Ok((input, Motion::Down(steps))),
		'L' => Ok((input, Motion::Left(steps))),
		_ => Ok((input, Motion::Right(steps))),
	}
}

#[cfg(test)]
mod test {
	use crate::day9::{gen, parse_motion, parse_motions, solve_part1, solve_part2, Motion};

	const EXAMPLE: &str = r"R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2";

	const EXAMPLE2: &str = r"R 5
U 8
L 8
D 3
R 17
D 10
L 25
U 20";

	#[test]
	fn parsing_motion() {
		assert_eq!(Ok(("", Motion::Up(1))), parse_motion("U 1"));
		assert_eq!(Ok(("", Motion::Down(2))), parse_motion("D 2"));
		assert_eq!(Ok(("", Motion::Left(3))), parse_motion("L 3"));
		assert_eq!(Ok(("", Motion::Right(4))), parse_motion("R 4"));

		assert!(parse_motion("G 4").is_err());
		assert!(parse_motion("U -1").is_err());
		assert!(parse_motion("😎 7").is_err());
	}

	#[test]
	fn parsing_motions() {
		let expected = vec![
			Motion::Right(4),
			Motion::Up(4),
			Motion::Left(3),
			Motion::Down(1),
			Motion::Right(4),
			Motion::Down(1),
			Motion::Left(5),
			Motion::Right(2),
		];

		let (tail, motions) = parse_motions(EXAMPLE).unwrap();

		assert!(tail.is_empty());
		assert_eq!(expected, motions);
	}

	#[test]
	fn part1() {
		let motions = gen(EXAMPLE);
		assert_eq!(13, solve_part1(&motions))
	}

	#[test]
	fn part2() {
		let motions = gen(EXAMPLE2);
		assert_eq!(36, solve_part2(&motions))
	}
}
