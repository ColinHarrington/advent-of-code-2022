use yaah::*;

#[aoc_generator(day3, part1)]
fn gen(input: &'static str) -> Vec<Rucksack> {
    input
        .lines()
        .map(Rucksack::new)
        .collect()
}


#[aoc(day3, part1)]
fn solve_part1(rucksacks: &Vec<Rucksack>) -> Option<i32> {
    Some(
        rucksacks
            .iter()
            .map(|rs| rs.shared_item())
            .map(|c| item_type_value(c))
            .sum()
    )
}
#[derive(Debug)]
pub struct Rucksack {
    pub compartment1: String,
    pub compartment2: String,
}

impl Rucksack {
    pub fn new(s: &str) -> Self {
        let (c1, c2) = s.split_at(s.len() / 2);
        Self { compartment1: c1.to_string(), compartment2: c2.to_string() }
    }

    pub fn shared_item(&self) -> char {
        shared_item(&self.compartment1, &self.compartment2)
    }
}

pub fn item_type_value(c: char) -> i32 {
    match c {
        'a'..='z' => c as i32 - 96,
        'A'..='Z' => c as i32 - 38,
        _ => 0
    }
}

pub fn shared_item(s1: &String, s2: &String) -> char {
    s1
        .chars()
        .find(|c| s2.contains(*c))
        .unwrap()
}

#[cfg(test)]
mod test {
    // use super::gen;
    use std::iter::zip;
    use crate::day3::{gen, item_type_value, parse_rucksack_items, Rucksack, shared_item, solve_part1};
    // use super::gen;

    const EXAMPLE: &str = r"vJrwpWtwJgWrhcsFMMfFFhFp
jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
PmmdzqPrVvPwwTWBwg
wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
ttgJtRGJQctTZtZT
CrZsJsPPZsGzwwsLwLmpwMDw";

    #[test]
    fn example_input() {
        let sacks = gen(EXAMPLE);
        assert_eq!(6, sacks.len());

        let expected = vec![
            ("vJrwpWtwJgWr", "hcsFMMfFFhFp", 'p'),
            ("jqHRNqRjqzjGDLGL", "rsFMfFZSrLrFZsSL", 'L'),
            ("PmmdzqPrV", "vPwwTWBwg", 'P')
        ];

        let mut iter = sacks.iter();

        for (c1, c2, shared) in expected {
            let sack = iter.next().unwrap();
            assert_eq!(c1, sack.compartment1);
            assert_eq!(c2, sack.compartment2);
            assert_eq!(shared, sack.shared_item());
        }

        assert_eq!('v', iter.next().unwrap().shared_item());
        assert_eq!('t', iter.next().unwrap().shared_item());
        assert_eq!('s', iter.next().unwrap().shared_item());
    }

    /// In the above example,
    /// the priority of the item type that appears in both compartments of each rucksack is 16 (p), 38 (L), 42 (P), 22 (v), 20 (t), and 19 (s);
    /// the sum of these is 157.
    #[test]
    fn example_solution() {
        assert_eq!(Some(157), solve_part1(&gen(EXAMPLE)))
    }


    /// To help prioritize item rearrangement, every item type can be converted to a priority:
    ///
    /// - Lowercase item types a through z have priorities 1 through 26.
    /// - Uppercase item types A through Z have priorities 27 through 52.
    ///
    /// In the above example, the priority of the item type that appears in both compartments of each rucksack is 16 (p), 38 (L), 42 (P), 22 (v), 20 (t), and 19 (s);
    #[test]
    fn test_item_value() {
        for (c, expected) in zip(('a'..='z').into_iter(), (1..=26).into_iter()).into_iter() {
            assert_eq!(expected, item_type_value(c));
        }
        for (c, expected) in zip(('A'..='Z').into_iter(), (27..=52).into_iter()).into_iter() {
            assert_eq!(expected, item_type_value(c));
        }

        assert_eq!(16, item_type_value('p'));
        assert_eq!(38, item_type_value('L'));
        assert_eq!(42, item_type_value('P'));
        assert_eq!(22, item_type_value('v'));
        assert_eq!(20, item_type_value('t'));
        assert_eq!(19, item_type_value('s'));
    }

    /// The first rucksack contains the items `vJrwpWtwJgWrhcsFMMfFFhFp`, which means its first compartment contains the items `vJrwpWtwJgWr`, while the second compartment contains the items `hcsFMMfFFhFp`.
    /// The only item type that appears in both compartments is lowercase `p`.
    #[test]
    fn first_rucksack<'a>() {
        let contents = "vJrwpWtwJgWrhcsFMMfFFhFp";
        let sack = Rucksack::new(contents);

        println!("{:?} :: {:?}", sack.compartment1, sack.compartment2);
        assert_eq!(sack.compartment1, "vJrwpWtwJgWr");
        assert_eq!(sack.compartment2, "hcsFMMfFFhFp");

        // assert_eq!(shared_item(sack.compartment1.to_string(), sack.compartment2.to_string()), 'p');
    }
}