use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::{char as nom_char, i64 as nom_i64, line_ending, one_of};
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::{delimited, separated_pair, tuple};
use nom::IResult;
use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Sub};
use yaah::*;

#[aoc(day21, part1)]
fn solve_part1(monkeys: &Vec<Monkey>) -> i64 {
	Computer::from(monkeys).eval(['r', 'o', 'o', 't'])
}

#[aoc(day21, part2)]
fn solve_part2(monkeys: &Vec<Monkey>) -> i64 {
	let mut computer = Computer::from(monkeys);

	computer.write(['h', 'u', 'm', 'n'], Expression::Variable(5));

	computer.solve_equation(['r', 'o', 'o', 't'])
}

pub struct Computer {
	registers: HashMap<Symbol, Expression>,
}

impl From<&Vec<Monkey>> for Computer {
	fn from(monkeys: &Vec<Monkey>) -> Self {
		Self {
			registers: HashMap::from_iter(monkeys.iter().map(|m| (m.symbol, m.expression.clone()))),
		}
	}
}

impl Computer {
	fn eval(&self, symbol: Symbol) -> i64 {
		let &node = self.registers.get(&symbol).unwrap();
		match node {
			Expression::Value(value) => value,
			Expression::Binary(left, op, right) => {
				let lhs = self.eval(left);
				let rhs = self.eval(right);
				match op {
					MathOp::ADD => lhs.add(rhs),
					MathOp::SUBTRACT => lhs.sub(rhs),
					MathOp::MULTIPLY => lhs.mul(rhs),
					MathOp::DIVIDE => lhs.div(rhs),
				}
			}
			Expression::Variable(_) => panic!("cannot eval a variable"),
		}
	}

	fn uneval(&self, symbol: Symbol, other: i64) -> i64 {
		let &node = self.registers.get(&symbol).unwrap();
		match node {
			Expression::Value(value) => value,
			Expression::Binary(lhs, op, rhs) => {
				match (self.contains_variable(lhs), self.contains_variable(rhs)) {
					(false, false) => self.eval(symbol),
					(true, false) => match op {
						MathOp::ADD => self.uneval(lhs, other.sub(self.eval(rhs))), // humn + x = var => humn = var - x
						MathOp::SUBTRACT => self.uneval(lhs, other.add(self.eval(rhs))), // humn - x = var => humn = var + x,
						MathOp::MULTIPLY => self.uneval(lhs, other.div(self.eval(rhs))), // humn * x = var => humn = var / x
						MathOp::DIVIDE => self.uneval(lhs, other.mul(self.eval(rhs))),   // humn / x = var => humn = var * x
					},
					(false, true) => match op {
						MathOp::ADD => self.uneval(rhs, other.sub(self.eval(lhs))), // x + humn = var => humn = var - x
						MathOp::SUBTRACT => self.uneval(rhs, self.eval(lhs).sub(other)), // x - humn = var => humn = x - var,
						MathOp::MULTIPLY => self.uneval(rhs, other.div(self.eval(lhs))), // x * humn = var => humn = var / x
						MathOp::DIVIDE => self.uneval(rhs, other.div(self.eval(lhs))),   // x / humn = var => humn = var / x
					},
					(true, true) => panic!("Sorry Dave"),
				}
			}
			Expression::Variable(_) => other,
		}
	}

	fn read(&self, symbol: Symbol) -> Expression {
		self.registers.get(&symbol).unwrap().clone()
	}

	fn write(&mut self, symbol: Symbol, expression: Expression) {
		self.registers.insert(symbol, expression);
	}

	fn contains_variable(&self, symbol: Symbol) -> bool {
		match self.read(symbol) {
			Expression::Variable(_) => true,
			Expression::Value(_) => false,
			Expression::Binary(left, _, right) => {
				self.contains_variable(left) || self.contains_variable(right)
			}
		}
	}

	fn solve_equation(&self, symbol: Symbol) -> i64 {
		if let Expression::Binary(left, _, right) = self.read(symbol) {
			let (variabled, valued) = if self.contains_variable(left) {
				(left, right)
			} else {
				(right, left)
			};
			self.uneval(variabled, self.eval(valued))
		} else {
			panic!("Expected Binary expression to solve")
		}
	}
}

type Symbol = [char; 4];

#[derive(Debug, Eq, PartialEq)]
pub struct Monkey {
	symbol: Symbol,
	expression: Expression,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Expression {
	Value(i64),
	Binary(Symbol, MathOp, Symbol),
	Variable(i64),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum MathOp {
	ADD,
	SUBTRACT,
	MULTIPLY,
	DIVIDE,
}

impl From<char> for MathOp {
	fn from(value: char) -> Self {
		match value {
			'+' => MathOp::ADD,
			'-' => MathOp::SUBTRACT,
			'*' => MathOp::MULTIPLY,
			'/' => MathOp::DIVIDE,
			_ => panic!(),
		}
	}
}

#[aoc_generator(day21)]
fn read_monkeys(input: &'static str) -> Vec<Monkey> {
	monkeys(input).unwrap().1
}

fn monkeys(input: &str) -> IResult<&str, Vec<Monkey>> {
	separated_list1(line_ending, monkey)(input)
}

fn monkey(input: &str) -> IResult<&str, Monkey> {
	map(
		separated_pair(symbol, tag(": "), expression),
		|(symbol, expression)| Monkey { symbol, expression },
	)(input)
}

fn expression(input: &str) -> IResult<&str, Expression> {
	alt((binary_expression, expression_value))(input)
}

fn binary_expression(input: &str) -> IResult<&str, Expression> {
	map(
		tuple((
			symbol,
			delimited(nom_char(' '), one_of("+-*/"), nom_char(' ')),
			symbol,
		)),
		|(left, op, right)| Expression::Binary(left, op.into(), right),
	)(input)
}

fn expression_value(input: &str) -> IResult<&str, Expression> {
	map(nom_i64, |value| Expression::Value(value))(input)
}

fn symbol(input: &str) -> IResult<&str, Symbol> {
	map(take(4usize), |chars: &str| {
		chars.chars().collect::<Vec<char>>().try_into().unwrap()
	})(input)
}

#[cfg(test)]
mod test {
	use crate::day21::Expression::{Binary, Value};
	use crate::day21::MathOp::{ADD, DIVIDE, MULTIPLY, SUBTRACT};
	use crate::day21::{
		binary_expression, monkey, read_monkeys, solve_part1, solve_part2, symbol, Expression,
		MathOp, Monkey,
	};

	const EXAMPLE: &str = r"root: pppw + sjmn
dbpl: 5
cczh: sllz + lgvd
zczc: 2
ptdq: humn - dvpt
dvpt: 3
lfqf: 4
humn: 5
ljgn: 2
sjmn: drzm * dbpl
sllz: 4
pppw: cczh / lfqf
lgvd: ljgn * ptdq
drzm: hmdt - zczc
hmdt: 32";

	#[test]
	fn part1() {
		let monkeys = read_monkeys(EXAMPLE);
		assert_eq!(152, solve_part1(&monkeys));
	}

	#[test]
	fn part2() {
		let monkeys = read_monkeys(EXAMPLE);
		assert_eq!(301, solve_part2(&monkeys));
	}

	#[test]
	fn parse_symbol() {
		let expected = ['a'; 4];
		assert_eq!(Ok(("", expected)), symbol("aaaa"));
	}

	#[test]
	fn parse_monkey() {
		let expected = Monkey {
			symbol: ['d', 'b', 'p', 'l'],
			expression: Expression::Value(5),
		};
		assert_eq!(Ok(("", expected)), monkey("dbpl: 5"));
	}

	#[test]
	fn parse_binary_expression() {
		let expected = Expression::Binary(['s', 'l', 'l', 'z'], MathOp::ADD, ['l', 'g', 'v', 'd']);
		assert_eq!(Ok(("", expected)), binary_expression("sllz + lgvd"));
	}

	#[test]
	fn parse_monkeys() {
		let expected: Vec<Monkey> = vec![
			Monkey {
				symbol: ['r', 'o', 'o', 't'],
				expression: Binary(['p', 'p', 'p', 'w'], ADD, ['s', 'j', 'm', 'n']),
			},
			Monkey {
				symbol: ['d', 'b', 'p', 'l'],
				expression: Value(5),
			},
			Monkey {
				symbol: ['c', 'c', 'z', 'h'],
				expression: Binary(['s', 'l', 'l', 'z'], ADD, ['l', 'g', 'v', 'd']),
			},
			Monkey {
				symbol: ['z', 'c', 'z', 'c'],
				expression: Value(2),
			},
			Monkey {
				symbol: ['p', 't', 'd', 'q'],
				expression: Binary(['h', 'u', 'm', 'n'], SUBTRACT, ['d', 'v', 'p', 't']),
			},
			Monkey {
				symbol: ['d', 'v', 'p', 't'],
				expression: Value(3),
			},
			Monkey {
				symbol: ['l', 'f', 'q', 'f'],
				expression: Value(4),
			},
			Monkey {
				symbol: ['h', 'u', 'm', 'n'],
				expression: Value(5),
			},
			Monkey {
				symbol: ['l', 'j', 'g', 'n'],
				expression: Value(2),
			},
			Monkey {
				symbol: ['s', 'j', 'm', 'n'],
				expression: Binary(['d', 'r', 'z', 'm'], MULTIPLY, ['d', 'b', 'p', 'l']),
			},
			Monkey {
				symbol: ['s', 'l', 'l', 'z'],
				expression: Value(4),
			},
			Monkey {
				symbol: ['p', 'p', 'p', 'w'],
				expression: Binary(['c', 'c', 'z', 'h'], DIVIDE, ['l', 'f', 'q', 'f']),
			},
			Monkey {
				symbol: ['l', 'g', 'v', 'd'],
				expression: Binary(['l', 'j', 'g', 'n'], MULTIPLY, ['p', 't', 'd', 'q']),
			},
			Monkey {
				symbol: ['d', 'r', 'z', 'm'],
				expression: Binary(['h', 'm', 'd', 't'], SUBTRACT, ['z', 'c', 'z', 'c']),
			},
			Monkey {
				symbol: ['h', 'm', 'd', 't'],
				expression: Value(32),
			},
		];
		assert_eq!(expected, read_monkeys(EXAMPLE));
	}
}
