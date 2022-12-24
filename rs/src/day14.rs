use std::{cmp, fmt};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Range;
use std::str::Chars;
use itertools::{Itertools, Product};
use nom::bytes::complete::tag;
use nom::character::complete::{char as nom_char, line_ending, not_line_ending, u32 as nom_u32};
use nom::IResult;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use yaah::*;


#[aoc_generator(day14)]
fn generate_structures(input: &'static str) -> Vec<RockStructure> {
    separated_list1(line_ending, rock_structure)(input).unwrap().1
}

#[aoc(day14, part1)]
fn solve_part1(structures: &Vec<RockStructure>) -> u32 {
    let mut cave = Cave::from_structures(structures);

    while let Some(grain) = cave.drop_grain() {
        cave.add_grain(grain);
    }

    let r = cave.map.into_iter().filter(|(k, v)| *v == 'o').count();
    r as u32
}


fn map_rocks(structure: &RockStructure) -> Vec<Position> {
    structure.0.iter()
        .tuple_windows()
        .map(|(a, b)| map_rock_line(a, b))
        .flatten()
        .dedup()
        .collect()
}

fn map_rock_line(from: &Position, to: &Position) -> Vec<Position> {
    let xmin = cmp::min(from.0, to.0);
    let xmax = cmp::max(from.0, to.0) + 1;
    let xrange = (xmin..xmax);

    let ymin = cmp::min(from.1, to.1);
    let ymax = cmp::max(from.1, to.1) + 1;
    let yrange = (ymin..ymax);

    xrange.cartesian_product(yrange)
        .map(|(x, y)| Position(x, y))
        .collect()
}

#[derive(Debug, Eq, PartialEq)]
pub struct Cave {
    map: HashMap<Position, char>,
    xrange: Range<u32>,
    yrange: Range<u32>,
}

impl Cave {
    fn from_structures(structures: &Vec<RockStructure>) -> Self {
        let rocks = structures.iter()
            .map(|structure| map_rocks(structure))
            .flatten()
            .dedup()
            .collect::<Vec<Position>>();

        let mut map: HashMap<Position, char> = HashMap::new();

        for rock in rocks {
            map.insert(rock, '#');
        }

        let xmin = map.keys().map(|p| p.0).min().unwrap();
        let xmax = map.keys().map(|p| p.0).max().unwrap();
        let xrange = (xmin..(xmax + 1));
        // let ymin = map.keys().map(|p| p.1).min().unwrap();
        let ymax = map.keys().map(|p| p.1).max().unwrap();
        let yrange = (0..(ymax + 1));

        Self { map, xrange, yrange }
    }

    fn in_range(&self, x: u32, y: u32) -> bool {
        self.xrange.contains(&x) && self.yrange.contains(&y)
    }

    /// Drops a grain and returns it's final destiny.
    /// Returns Some(Position) where it lands
    /// or None if it's gone to the abyss
    fn drop_grain(&mut self) -> Option<Position> {
        let mut x = 500;
        self.yrange.clone().find_map(|y| match self.move_down(x, y) {
            Some(p) => {
                x = p.0;
                None
            }
            None => Some(Position(x, y))
        })
    }
    fn add_grain(&mut self, grain: Position) {
        self.map.insert(grain, 'o');
    }

    fn is_open(&self, x: u32, y: u32) -> bool {
        self.map.get(&Position(x, y)).is_none()
    }
    fn move_down(&self, x: u32, y: u32) -> Option<Position> {
        vec![
            (x, y + 1),
            (x - 1, y + 1),
            (x + 1, y + 1),
        ].into_iter()
            .map(|(x1, y1)| Position(x1, y1))
            .find(|p| self.map.get(p).is_none())
    }
}

impl fmt::Display for Cave {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let xmin = self.map.keys().map(|p| p.0).min().unwrap();
        let xmax = self.map.keys().map(|p| p.0).max().unwrap();

        let labels: Vec<String> = vec![
            format!("{xmin}"),
            "500".to_string(),
            format!("{xmax}"),
        ];
        let label_lines: Vec<String> = (0..(labels.iter().map(|label| label.len()).max().unwrap()))
            // .inspect(|i|println!("inspect {i}"))
            .map(|lr| self.xrange.clone()
                .map(|x| match x {
                    _ if x == xmin => labels[0].chars().nth(lr).unwrap(),
                    _ if x == 500 => labels[1].chars().nth(lr).unwrap(),
                    _ if x == xmax => labels[2].chars().nth(lr).unwrap(),
                    _ => ' '
                }).collect::<String>())
            .map(|label_line| format!("  {label_line}"))
            .collect::<Vec<String>>();


        let lines: Vec<String> = self.yrange.clone()
            .map(|y| format!("{y} {}", self.xrange.clone()
                .map(|x| self.map.get(&Position(x, y))
                    .unwrap_or(&'.'))
                .collect::<String>())
            )
            .collect();

        write!(f, "\n{}\n{}", label_lines.join("\n"), lines.join("\n"))
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Position(u32, u32);

#[derive(Debug, Eq, PartialEq)]
pub struct RockStructure(Vec<Position>);

fn position(input: &str) -> IResult<&str, Position> {
    let (input, (x, y)) = separated_pair(nom_u32, nom_char(','), nom_u32)(input)?;
    Ok((input, Position(x, y)))
}

fn rock_structure(input: &str) -> IResult<&str, RockStructure> {
    let (input, points) = separated_list1(tag(" -> "), position)(input)?;
    Ok((input, RockStructure(points)))
}

#[cfg(test)]
mod test {
    use crate::day14::{Cave, generate_structures, Position, RockStructure, solve_part1};

    const EXAMPLE: &str = r"498,4 -> 498,6 -> 496,6
503,4 -> 502,4 -> 502,9 -> 494,9";

    #[test]
    fn parse_rocks() {
        let expected = vec![
            RockStructure(vec![
                Position(498, 4),
                Position(498, 6),
                Position(496, 6),
            ]),
            RockStructure(vec![
                Position(503, 4),
                Position(502, 4),
                Position(502, 9),
                Position(494, 9),
            ]),
        ];

        assert_eq!(expected, generate_structures(EXAMPLE));
    }

    #[test]
    fn cave_display() {
        let structures = generate_structures(EXAMPLE);
        let cave = Cave::from_structures(&structures);

        let expected: String = r"
  4     5  5
  9     0  0
  4     0  3
0 ..........
1 ..........
2 ..........
3 ..........
4 ....#...##
5 ....#...#.
6 ..###...#.
7 ........#.
8 ........#.
9 #########.".to_string();
        let display = format!("{cave}");
        assert_eq!(expected, display);
    }

    #[test]
    fn part1() {
        assert_eq!(24, solve_part1(&generate_structures(EXAMPLE)));
    }
}