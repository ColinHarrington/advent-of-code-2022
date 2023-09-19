use std::str::{FromStr, SplitWhitespace};
use yaah::*;

#[aoc_generator(day2, part1)]
fn gen(input: &'static str) -> Vec<Round> {
	input
		.lines()
		.filter_map(|s| Round::from_part1(s).ok())
		.collect()
}

#[aoc_generator(day2, part2)]
fn gen_day2(input: &'static str) -> Vec<Round> {
	input
		.lines()
		.filter_map(|s| Round::from_part2(s).ok())
		.collect()
}

#[aoc(day2, part1)]
#[aoc(day2, part2)]
fn solve_both_parts(rounds: &[Round]) -> Option<i32> {
	Some(rounds.iter().map(|r| r.score()).sum())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Round {
	pub opponent: Hand,
	pub choice: Hand,
	pub result: RoundResult,
}

impl Round {
	pub fn from_part1(s: &str) -> Result<Self, RoundParseError> {
		let mut iter: SplitWhitespace = s.split_whitespace();

		let opponent = iter.next().unwrap().parse::<Hand>();
		let choice = iter.next().unwrap().parse::<Hand>();

		match (opponent, choice) {
			(Ok(o), Ok(c)) => Ok(Round::from_hands(o, c)),
			(_, _) => Err(RoundParseError),
		}
	}

	pub fn from_hands(opponent: Hand, choice: Hand) -> Self {
		let result = RoundResult::from_hands(choice, opponent);
		Self {
			opponent,
			choice,
			result,
		}
	}

	pub fn from_part2(s: &str) -> Result<Round, HandParseError> {
		let mut iter: SplitWhitespace = s.split_whitespace();
		let opponent_str = iter.next().unwrap();
		let round_result_str = iter.next().unwrap();

		let opponent: Hand = opponent_str.parse::<Hand>().unwrap();
		let result: RoundResult = round_result_str.parse::<RoundResult>().unwrap();
		Ok(Round::from_result(opponent, result))
	}

	pub fn from_result(opponent: Hand, result: RoundResult) -> Self {
		let choice = match result {
			RoundResult::Draw => opponent,
			RoundResult::Win => match opponent {
				Hand::Rock => Hand::Paper,
				Hand::Paper => Hand::Scissors,
				Hand::Scissors => Hand::Rock,
			},
			RoundResult::Lose => match opponent {
				Hand::Rock => Hand::Scissors,
				Hand::Paper => Hand::Rock,
				Hand::Scissors => Hand::Paper,
			},
		};
		Self {
			opponent,
			choice,
			result,
		}
	}

	fn score(&self) -> i32 {
		self.choice.value() + self.result.value()
	}
}

#[derive(Debug, Clone, Copy)]
pub struct RoundParseError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RoundResult {
	Win,
	Draw,
	Lose,
}

impl RoundResult {
	pub fn from_hands(h1: Hand, h2: Hand) -> Self {
		match h1 {
			Hand::Rock => match h2 {
				Hand::Rock => RoundResult::Draw,
				Hand::Paper => RoundResult::Lose,
				Hand::Scissors => RoundResult::Win,
			},
			Hand::Paper => match h2 {
				Hand::Rock => RoundResult::Win,
				Hand::Paper => RoundResult::Draw,
				Hand::Scissors => RoundResult::Lose,
			},
			Hand::Scissors => match h2 {
				Hand::Rock => RoundResult::Lose,
				Hand::Paper => RoundResult::Win,
				Hand::Scissors => RoundResult::Draw,
			},
		}
	}
	fn value(&self) -> i32 {
		match *self {
			RoundResult::Win => 6,
			RoundResult::Draw => 3,
			RoundResult::Lose => 0,
		}
	}
}

impl FromStr for RoundResult {
	type Err = RoundParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"X" => Ok(RoundResult::Lose),
			"Y" => Ok(RoundResult::Draw),
			"Z" => Ok(RoundResult::Win),
			_ => Err(RoundParseError),
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hand {
	Rock,
	Paper,
	Scissors,
}

impl Hand {
	fn value(&self) -> i32 {
		match *self {
			Hand::Rock => 1,
			Hand::Paper => 2,
			Hand::Scissors => 3,
		}
	}
}

#[derive(Debug, Clone)]
pub struct HandParseError;

impl FromStr for Hand {
	type Err = HandParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"A" => Ok(Hand::Rock),
			"B" => Ok(Hand::Paper),
			"C" => Ok(Hand::Scissors),
			"X" => Ok(Hand::Rock),
			"Y" => Ok(Hand::Paper),
			"Z" => Ok(Hand::Scissors),
			_ => Err(HandParseError),
		}
	}
}

#[cfg(test)]
mod test {
	use super::gen;
	use crate::day2::{gen_day2, solve_both_parts, Hand, Round, RoundParseError, RoundResult};

	const EXAMPLE: &str = r"A Y
B X
C Z";

	/// This strategy guide predicts and recommends the following:
	///
	/// * In the first round, your opponent will choose Rock (A), and you should choose Paper (Y). This ends in a win for you with a score of 8 (2 because you chose Paper + 6 because you won).
	/// * In the second round, your opponent will choose Paper (B), and you should choose Rock (X). This ends in a loss for you with a score of 1 (1 + 0).
	/// * The third round is a draw with both players choosing Scissors, giving you a score of 3 + 3 = 6.
	/// In this example, if you were to follow the strategy guide, you would get a total score of 15 (8 + 1 + 6).
	#[test]
	fn example_input() {
		let expected = vec![
			(Hand::Rock, Hand::Paper, 8),
			(Hand::Paper, Hand::Rock, 1),
			(Hand::Scissors, Hand::Scissors, 6),
		];

		let rounds = gen(EXAMPLE);
		let mut iter = rounds.iter();

		for (expected_opponent, expected_choice, expected_score) in expected {
			let round = iter.next().unwrap();
			assert_eq!(round.opponent, expected_opponent);
			assert_eq!(round.choice, expected_choice);
			assert_eq!(round.score(), expected_score);
		}
		assert!(iter.next().is_none());
	}

	/// In this example, if you were to follow the strategy guide, you would get a total score of 15 (8 + 1 + 6).
	#[test]
	fn part1_example() {
		assert_eq!(Some(15), solve_both_parts(&gen(EXAMPLE)));
	}

	#[test]
	fn hand_from_str() {
		let definition: Vec<(&str, Hand)> = vec![
			("A", Hand::Rock),
			("B", Hand::Paper),
			("C", Hand::Scissors),
			("X", Hand::Rock),
			("Y", Hand::Paper),
			("Z", Hand::Scissors),
		];

		for (value, hand) in definition.iter() {
			assert_eq!(value.parse::<Hand>().ok().unwrap(), *hand)
		}

		assert!("D".parse::<Hand>().is_err());
		assert!("a".parse::<Hand>().is_err());
		assert!("Q".parse::<Hand>().is_err());
	}

	#[test]
	fn round_from_str() {
		let round_result: Result<Round, RoundParseError> = Round::from_part1("A Z");
		assert!(round_result.is_ok());
		assert!(round_result.ok().is_some());
		let round = round_result.ok().unwrap();

		assert_eq!(Hand::Rock, round.opponent);
		assert_eq!(Hand::Scissors, round.choice);
	}

	/// * In the first round, your opponent will choose Rock (A), and you need the round to end in a draw (Y), so you also choose Rock. This gives you a score of 1 + 3 = 4.
	/// * In the second round, your opponent will choose Paper (B), and you choose Rock so you lose (X) with a score of 1 + 0 = 1.
	/// * In the third round, you will defeat your opponent's Scissors with Rock for a score of 1 + 6 = 7.
	#[test]
	fn part2_example_input() {
		let rounds = gen_day2(EXAMPLE);
		let mut iter = rounds.iter();

		let definition = [
			(Hand::Rock, Hand::Rock, RoundResult::Draw, 4),
			(Hand::Paper, Hand::Rock, RoundResult::Lose, 1),
			(Hand::Scissors, Hand::Rock, RoundResult::Win, 7),
		];

		for (expected_hand, expected_choice, expected_result, expected_score) in definition.iter() {
			let round = iter.next().unwrap();
			assert_eq!(*expected_hand, round.opponent);
			assert_eq!(*expected_choice, round.choice);
			assert_eq!(*expected_result, round.result);
			assert_eq!(*expected_score, round.score());
		}

		assert!(iter.next().is_none());
	}

	/// Now that you're correctly decrypting the ultra top secret strategy guide, you would get a total score of 12.
	#[test]
	fn part2_example() {
		assert_eq!(Some(12), solve_both_parts(&gen_day2(EXAMPLE)));
	}
}
