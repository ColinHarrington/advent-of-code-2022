use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{
	alpha1, char as complete_char, digit1, multispace1, newline, space1,
};
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, preceded};
use nom::IResult;
use yaah::*;

type Stacks = Vec<Vec<char>>;

#[aoc_generator(day5)]
fn gen(input: &'static str) -> (Stacks, Vec<Move>) {
	let (_, (stacks, moves)) = parse_input(input).unwrap();
	(stacks, moves)
}

#[aoc(day5, part1)]
fn solve_part1(input: &(Stacks, Vec<Move>)) -> String {
	let (stx, moves) = input;

	let stacks = crane_single_items(stx.clone(), moves.clone());

	stacks.iter().filter_map(|s| s.last()).collect()
}

#[aoc(day5, part2)]
fn solve_part2(input: &(Stacks, Vec<Move>)) -> String {
	let (stx, moves) = input;

	let stacks = crane_multiple_items(stx.clone(), moves.clone());

	stacks.iter().filter_map(|s| s.last()).collect()
}

fn crane_single_items(mut stacks: Stacks, moves: Vec<Move>) -> Stacks {
	#[cfg(feature = "debug")]
	print_stacks(&stacks);
	for Move { count, from, to } in moves.iter() {
		for _ in 0..*count {
			let c = stacks[*from as usize - 1].pop().unwrap();
			stacks[*to as usize - 1].push(c);
		}
	}
	#[cfg(feature = "debug")]
	print_stacks(&stacks);
	stacks
}

fn crane_multiple_items(mut stacks: Stacks, moves: Vec<Move>) -> Stacks {
	#[cfg(feature = "debug")]
	print_stacks(&stacks);
	for Move { count, from, to } in moves.iter() {
		let mut load: Vec<char> = vec![];
		for _ in 0..*count {
			load.push(stacks[*from as usize - 1].pop().unwrap())
		}
		load.iter()
			.rev()
			.for_each(|c| stacks[*to as usize - 1].push(*c));
	}
	#[cfg(feature = "debug")]
	print_stacks(&stacks);
	stacks
}

#[cfg(feature = "debug")]
fn print_stacks(stacks: &Stacks) {
	let mut sorted_stacks = stacks.clone();
	sorted_stacks.sort_by_key(|s| s.len());
	let tallest = sorted_stacks.last().unwrap();
	let stack_lines: Vec<String> = tallest
		.iter()
		.enumerate()
		.map(|(idx, _)| {
			stacks
				.into_iter()
				.map(move |stack| stack.get(idx))
				.map(|cr| match cr {
					Some(c) => format!("[{}]", c),
					None => "   ".to_string(),
				})
				.collect::<Vec<String>>()
				.join(" ")
		})
		.rev()
		.collect();

	for stack_line in stack_lines.iter() {
		println!("{:?}", stack_line.as_str());
	}

	let label_line = stacks
		.iter()
		.enumerate()
		.map(|(i, _)| format!(" { } ", i))
		.collect::<Vec<String>>()
		.join(" ");

	println!("{:?}\n", label_line);
}

///Nom parser function for parsing crate values
/// - `   ` is an empty slot (`None`)
/// - `[X]` is a Crate with the value `Some(X)`
pub fn parse_crate(input: &str) -> IResult<&str, Option<&str>> {
	let (input, c) = alt((
		tag("   "),
		delimited(complete_char('['), alpha1, complete_char(']')),
	))(input)?;

	Ok((
		input,
		match c {
			"   " => None,
			v => Some(v),
		},
	))
}

/// Parse Crate lines

pub fn crate_line(input: &str) -> IResult<&str, Vec<Option<&str>>> {
	let (input, crates) = separated_list1(tag(" "), parse_crate)(input)?;
	Ok((input, crates))
}

pub fn crate_lines(input: &str) -> IResult<&str, Vec<Vec<Option<&str>>>> {
	let (input, crates) = separated_list1(newline, crate_line)(input)?;
	Ok((input, crates))
}

/// Parse Crates from input lines. These lines come in top down, so they need to be rotated.
pub fn crate_stacks(input: &str) -> IResult<&str, Stacks> {
	let (input, crate_lines) = crate_lines(input)?;
	let stacks: Stacks = crate_lines
		.first()
		.unwrap()
		.iter()
		.enumerate()
		.map(|(i, _)| {
			crate_lines
				.iter()
				.filter_map(|cl| cl[i])
				.filter_map(|s| s.chars().next())
				.rev()
				.collect::<Vec<char>>()
		})
		.collect();
	Ok((input, stacks))
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Move {
	count: u8,
	from: u8,
	to: u8,
}

/// Parsing move line like `move 1 from 2 to 1`
pub fn parse_move(input: &str) -> IResult<&str, Move> {
	let (input, _) = tag("move ")(input)?;
	let (input, count) = complete::u8(input)?;
	let (input, _) = tag(" from ")(input)?;
	let (input, from) = complete::u8(input)?;
	let (input, _) = tag(" to ")(input)?;
	let (input, to) = complete::u8(input)?;

	Ok((input, Move { count, from, to }))
}

// Parsing moves from input with nom
pub fn moves(input: &str) -> IResult<&str, Vec<Move>> {
	Ok(separated_list1(newline, parse_move)(input)?)
}

/// Parsing input with nom
fn parse_input(input: &str) -> IResult<&str, (Stacks, Vec<Move>)> {
	let (input, stacks) = crate_stacks(input)?;

	let (input, _) = newline(input)?;
	let (input, _) = many1(preceded(space1, digit1))(input)?;
	let (input, _) = multispace1(input)?;

	let (input, moves) = moves(input)?;

	Ok((input, (stacks, moves)))
}

#[cfg(test)]
mod test {
	use crate::day5::{gen, parse_crate, parse_move, solve_part1, solve_part2, Move};

	const EXAMPLE: &str = concat!(
		"    [D]    \n",
		"[N] [C]    \n",
		"[Z] [M] [P]\n",
		" 1   2   3 \n",
		"\n",
		"move 1 from 2 to 1\n",
		"move 3 from 1 to 3\n",
		"move 2 from 2 to 1\n",
		"move 1 from 1 to 2\n"
	);

	#[test]
	fn example() {
		let (stacks, moves) = gen(EXAMPLE);

		let expected_stacks = vec![vec!['Z', 'N'], vec!['M', 'C', 'D'], vec!['P']];
		assert_eq!(stacks, expected_stacks);

		let expected_moves = vec![
			Move {
				count: 1,
				from: 2,
				to: 1,
			},
			Move {
				count: 3,
				from: 1,
				to: 3,
			},
			Move {
				count: 2,
				from: 2,
				to: 1,
			},
			Move {
				count: 1,
				from: 1,
				to: 2,
			},
		];
		assert_eq!(moves, expected_moves);
	}

	#[test]
	fn test_crate_parsing() {
		assert_eq!(parse_crate("   "), Ok(("", None)));
		assert_eq!(parse_crate("[A]"), Ok(("", Some("A"))));
	}

	#[test]
	fn test_move_parsing() {
		assert_eq!(
			parse_move("move 1 from 2 to 1"),
			Ok((
				"",
				Move {
					count: 1,
					from: 2,
					to: 1
				}
			))
		);
	}

	/// The Elves just need to know which crate will end up on top of each stack; in this example,
	/// the top crates are C in stack 1, M in stack 2, and Z in stack 3,
	/// so you should combine these together and give the Elves the message `CMZ`.
	#[test]
	fn part1() {
		assert_eq!(solve_part1(&gen(EXAMPLE)), "CMZ")
	}

	#[test]
	fn part2() {
		assert_eq!(solve_part2(&gen(EXAMPLE)), "MCD")
	}
}
