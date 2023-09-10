use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char as nom_char, line_ending, newline, one_of, u64 as nom_u64};
use nom::multi::{count, separated_list1};
use nom::sequence::{preceded, separated_pair, terminated};
use nom::IResult;
use yaah::*;

#[aoc_generator(day11)]
fn gen(input: &'static str) -> Vec<Monkey> {
	monkeys(input).unwrap().1
}

#[aoc(day11, part1)]
fn solve_part1(monkeys: &Vec<Monkey>) -> u64 {
	monkey_business(monkeys, 20, true)
}

#[aoc(day11, part2)]
fn solve_part2(monkeys: &Vec<Monkey>) -> u64 {
	monkey_business(monkeys, 10000, false)
}

fn monkey_business(monkeys: &Vec<Monkey>, rounds: usize, div3: bool) -> u64 {
	let mut items: Vec<Vec<u64>> = monkeys
		.iter()
		.map(|monkey| monkey.starting_items.clone())
		.collect();

	let common_product: u64 = monkeys
		.iter()
		.map(|monkey| monkey.test.divisible_by)
		.product::<u64>();

	let mut inspections: Vec<u64> = monkeys.iter().map(|_| 0).collect();
	for _ in 0..rounds {
		for monkey in monkeys {
			for item in items[monkey.number].clone() {
				inspections[monkey.number] += 1;
				let other_value = match monkey.operation.operand {
					Operand::Old => item,
					Operand::Value(value) => value,
				};
				let worry_level: u64 = match monkey.operation.operator {
					Operator::Plus => item + other_value,
					Operator::Times => item * other_value,
				};
				let worry_level = match div3 {
					true => worry_level / 3,
					false => worry_level % common_product,
				};
				let thrown_to = match worry_level % monkey.test.divisible_by == 0 {
					true => monkey.test.if_true as usize,
					false => monkey.test.if_false as usize,
				};
				items[thrown_to].push(worry_level);
			}
			items[monkey.number].clear();
		}
	}
	inspections.into_iter().sorted().rev().take(2).product()
}

fn starting_items(input: &str) -> IResult<&str, Vec<u64>> {
	preceded(
		tag("  Starting items: "),
		separated_list1(tag(", "), nom_u64),
	)(input)
}

fn operator(input: &str) -> IResult<&str, Operator> {
	let (input, op) = one_of("+*")(input)?;
	Ok((
		input,
		match op {
			'*' => Operator::Times,
			_ => Operator::Plus,
		},
	))
}

fn operand(input: &str) -> IResult<&str, Operand> {
	alt((parse_old, operand_value))(input)
}

fn parse_old(input: &str) -> IResult<&str, Operand> {
	let (input, _) = tag("old")(input)?;
	Ok((input, Operand::Old))
}

fn operand_value(input: &str) -> IResult<&str, Operand> {
	let (input, value) = nom_u64(input)?;
	Ok((input, Operand::Value(value)))
}

fn operation(input: &str) -> IResult<&str, Operation> {
	let (input, (operator, operand)) = preceded(
		tag("  Operation: new = old "),
		separated_pair(operator, nom_char(' '), operand),
	)(input)?;

	Ok((input, Operation { operator, operand }))
}

fn test_expression(input: &str) -> IResult<&str, TestExpression> {
	let (input, divisible_by) = preceded(
		tag("  Test: divisible by "),
		terminated(nom_u64, line_ending),
	)(input)?;
	let (input, if_true) = preceded(
		tag("    If true: throw to monkey "),
		terminated(nom_u64, line_ending),
	)(input)?;
	let (input, if_false) = preceded(tag("    If false: throw to monkey "), nom_u64)(input)?;

	Ok((
		input,
		TestExpression {
			divisible_by,
			if_true,
			if_false,
		},
	))
}

fn monkey(input: &str) -> IResult<&str, Monkey> {
	let (input, number) = preceded(
		tag("Monkey "),
		terminated(nom_u64, terminated(nom_char(':'), line_ending)),
	)(input)?;
	let (input, starting_items) = terminated(starting_items, line_ending)(input)?;
	let (input, operation) = terminated(operation, line_ending)(input)?;
	let (input, test) = test_expression(input)?;

	Ok((
		input,
		Monkey {
			number: number as usize,
			starting_items,
			operation,
			test,
		},
	))
}

fn monkeys(input: &str) -> IResult<&str, Vec<Monkey>> {
	separated_list1(count(newline, 2), monkey)(input)
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Operator {
	Plus,
	Times,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Operand {
	Old,
	Value(u64),
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct Operation {
	operator: Operator,
	operand: Operand,
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct TestExpression {
	divisible_by: u64,
	if_true: u64,
	if_false: u64,
}

#[derive(Debug, PartialEq)]
pub struct Monkey {
	number: usize,
	starting_items: Vec<u64>,
	operation: Operation,
	test: TestExpression,
}

#[cfg(test)]
mod test {
	use crate::day11::{
		gen, monkey, monkeys, operation, solve_part1, solve_part2, starting_items, test_expression,
		Monkey, Operand, Operation, Operator, TestExpression,
	};

	const EXAMPLE: &str = r"Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0

Monkey 2:
  Starting items: 79, 60, 97
  Operation: new = old * old
  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3

Monkey 3:
  Starting items: 74
  Operation: new = old + 3
  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1";

	#[test]
	fn parsing_starting_items() {
		assert_eq!(
			Ok(("", vec![79, 98])),
			starting_items("  Starting items: 79, 98")
		);
		assert_eq!(
			Ok(("", vec![54, 65, 75, 74])),
			starting_items("  Starting items: 54, 65, 75, 74")
		);
		assert_eq!(
			Ok(("", vec![79, 60, 97])),
			starting_items("  Starting items: 79, 60, 97")
		);
		assert_eq!(Ok(("", vec![74])), starting_items("  Starting items: 74"));
	}

	#[test]
	fn parsing_tests() {
		let monkey0 = r"  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3";
		assert_eq!(
			Ok((
				"",
				TestExpression {
					divisible_by: 23,
					if_true: 2,
					if_false: 3
				}
			)),
			test_expression(monkey0)
		);

		let monkey1 = r"  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0";
		assert_eq!(
			Ok((
				"",
				TestExpression {
					divisible_by: 19,
					if_true: 2,
					if_false: 0
				}
			)),
			test_expression(monkey1)
		);

		let monkey2 = r"  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3";
		assert_eq!(
			Ok((
				"",
				TestExpression {
					divisible_by: 13,
					if_true: 1,
					if_false: 3
				}
			)),
			test_expression(monkey2)
		);

		let monkey3 = r"  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1";
		assert_eq!(
			Ok((
				"",
				TestExpression {
					divisible_by: 17,
					if_true: 0,
					if_false: 1
				}
			)),
			test_expression(monkey3)
		);
	}

	#[test]
	fn parsing_operation() {
		assert_eq!(
			Ok((
				"",
				Operation {
					operator: Operator::Times,
					operand: Operand::Value(19)
				}
			)),
			operation("  Operation: new = old * 19")
		);
		assert_eq!(
			Ok((
				"",
				Operation {
					operator: Operator::Plus,
					operand: Operand::Value(6)
				}
			)),
			operation("  Operation: new = old + 6")
		);
		assert_eq!(
			Ok((
				"",
				Operation {
					operator: Operator::Times,
					operand: Operand::Old
				}
			)),
			operation("  Operation: new = old * old")
		);
	}

	#[test]
	fn test_monkey() {
		let monkey0 = r"Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3";
		let expected = Monkey {
			number: 0,
			starting_items: vec![79, 98],
			operation: Operation {
				operator: Operator::Times,
				operand: Operand::Value(19),
			},
			test: TestExpression {
				divisible_by: 23,
				if_true: 2,
				if_false: 3,
			},
		};
		assert_eq!(Ok(("", expected)), monkey(monkey0));
	}

	#[test]
	fn test_monkeys() {
		let (tail, barrel) = monkeys(EXAMPLE).unwrap();

		let last_monkey = barrel.into_iter().last().unwrap();
		let expected = Monkey {
			number: 3,
			starting_items: vec![74],
			operation: Operation {
				operator: Operator::Plus,
				operand: Operand::Value(3),
			},
			test: TestExpression {
				divisible_by: 17,
				if_true: 0,
				if_false: 1,
			},
		};

		assert_eq!(expected, last_monkey);
		assert_eq!("", tail);
	}

	#[test]
	fn test_example1() {
		let barrel = gen(EXAMPLE);
		assert_eq!(4, barrel.len());
	}

	#[test]
	fn part1() {
		assert_eq!(10605, solve_part1(&gen(EXAMPLE)));
	}

	#[test]
	fn part2() {
		assert_eq!(2713310158, solve_part2(&gen(EXAMPLE)));
	}
}
