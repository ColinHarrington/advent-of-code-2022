use std::cmp::Ordering;
use std::num::ParseIntError;
use std::str::FromStr;
use yaah::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Elf {
    pub calories: Vec<i32>,
}

impl Elf {
    fn total_calories(&self) -> i32 {
        self.calories.iter().sum()
    }
}

impl FromStr for Elf {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let calories: Vec<i32> = s.lines().filter_map(|s| s.parse().ok()).collect();
        Ok(Elf { calories })
    }
}

impl Eq for Elf {}

impl Ord for Elf {
    fn cmp(&self, other: &Self) -> Ordering {
        self.total_calories().cmp(&other.total_calories())
    }
}
impl PartialOrd for Elf {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[aoc_generator(day1)]
fn gen(input: &'static str) -> Vec<Elf> {
    input
        .split_terminator("\n\n")
        .filter_map(|s| s.parse().ok())
        .collect()
}

#[aoc(day1, part1)]
fn solve_part1(elves: &[Elf]) -> Option<i32> {
    elves.iter().map(|e| e.total_calories()).max()
}

#[aoc(day1, part2)]
fn solve_part2(all_elves: &[Elf]) -> Option<i32> {
    let mut elves = all_elves.to_vec();
    elves.sort_by(|a, b| b.cmp(a));
    Some(elves.iter().take(3).map(|e| e.total_calories()).sum())
}

#[cfg(test)]
mod test {
    use super::gen;
    use crate::day1::{solve_part1, Elf};

    const EXAMPLE: &str = r"1000
2000
3000

4000

5000
6000

7000
8000
9000

10000";

    /// This list represents the Calories of the food carried by five Elves:
    ///
    /// * The first Elf is carrying food with 1000, 2000, and 3000 Calories, a total of 6000 Calories.
    /// * The second Elf is carrying one food item with 4000 Calories.
    /// * The third Elf is carrying food with 5000 and 6000 Calories, a total of 11000 Calories.
    /// * The fourth Elf is carrying food with 7000, 8000, and 9000 Calories, a total of 24000 Calories.
    /// * The fifth Elf is carrying one food item with 10000 Calories.
    #[test]
    fn example_input() {
        let elves: Vec<Elf> = gen(EXAMPLE);

        let first = &elves[0];
        assert_eq!(vec![1000, 2000, 3000], first.calories);
        assert_eq!(6000, first.total_calories());

        let second = &elves[1];
        assert_eq!(1, second.calories.len());
        assert_eq!(4000, second.total_calories());

        let third = &elves[2];
        assert_eq!(vec![5000, 6000], third.calories);
        assert_eq!(11000, third.total_calories());

        let fourth = &elves[3];
        assert_eq!(vec![7000, 8000, 9000], fourth.calories);
        assert_eq!(24000, fourth.total_calories());

        let fifth = &elves[4];
        assert_eq!(1, fifth.calories.len());
        assert_eq!(10000, fifth.total_calories());
    }

    /// In the example above, this is 24000 (carried by the fourth Elf).
    #[test]
    fn part1_example() {
        assert_eq!(Some(24000), solve_part1(&gen(EXAMPLE)));
    }
}
