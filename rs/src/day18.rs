use std::collections::{HashSet, VecDeque};
use std::ops::{Add, Not, Sub};
use itertools::Itertools;
use nom::character::complete::{char as nom_char, line_ending};
use nom::IResult;
use nom::character::complete::i8 as nom_i8;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use yaah::*;


#[aoc_generator(day18)]
fn read_cubes(input: &'static str) -> Vec<Cube> {
    cubes(input).unwrap().1
}

#[aoc(day18, part1)]
fn solve_part1(cubes: &Vec<Cube>) -> usize {
    let cube_set: HashSet<Cube> = HashSet::from_iter(cubes.into_iter().map(|&cube| cube));
    cube_set.iter()
        .map(|cube| cube.adjacent()
            .into_iter()
            .filter(|other| cube_set.contains(other).not())
            .count())
        .sum()
}

#[aoc(day18, part2)]
fn solve_part2(cubes: &Vec<Cube>) -> usize {
    let cube_set: HashSet<Cube> = HashSet::from_iter(cubes.into_iter().map(|&cube| cube));

    let (lower, upper) = bounds(&cube_set);

    let flood = fill(lower, upper, &cube_set);

    cube_set.iter()
        .map(|cube| cube.adjacent()
            .into_iter()
            .filter(|other| flood.contains(other))
            .count())
        .sum()
}

fn bounds(cubes: &HashSet<Cube>) -> (Cube, Cube) {
    let (xmin, xmax) = cubes.iter().map(|cube| cube.x).minmax().into_option().unwrap();
    let (ymin, ymax) = cubes.iter().map(|cube| cube.y).minmax().into_option().unwrap();
    let (zmin, zmax) = cubes.iter().map(|cube| cube.z).minmax().into_option().unwrap();

    (Cube { x: xmin.sub(1), y: ymin.sub(1), z: zmin.sub(1) },
     Cube { x: xmax.add(1), y: ymax.add(1), z: zmax.add(1) })
}

fn fill(min: Cube, max: Cube, cubes: &HashSet<Cube>) -> HashSet<Cube> {
    let mut flood: HashSet<Cube> = HashSet::from([min]);
    let mut queue = VecDeque::from([min]);

    while let Some(cube) = queue.pop_front() {
        let new_fill: Vec<Cube> = cube.adjacent()
            .into_iter()
            .filter(|c| flood.contains(c).not())
            .filter(|c| c.in_bounds(&min, &max))
            .filter(|c| cubes.contains(c).not())
            .collect();
        for fill_cube in new_fill {
            flood.insert(fill_cube.clone());
            queue.push_back(fill_cube);
        }
    }
    flood
}

#[derive(Eq, PartialEq, Debug, Hash, Copy, Clone)]
pub struct Cube {
    x: i8,
    y: i8,
    z: i8,
}

impl Cube {
    fn adjacent(&self) -> Vec<Cube> {
        vec![
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ].into_iter()
            .map(|t| self.translate(t))
            .collect()
    }

    fn translate(&self, (xdiff, ydiff, zdiff): (i8, i8, i8)) -> Cube {
        Cube {
            x: self.x.add(xdiff),
            y: self.y.add(ydiff),
            z: self.z.add(zdiff),
        }
    }

    fn in_bounds(&self, min: &Cube, max: &Cube) -> bool {
        self.x >= min.x
            && self.y >= min.y
            && self.z >= min.z
            && self.x <= max.x
            && self.y <= max.y
            && self.z <= max.z
    }
}

fn cubes(input: &str) -> IResult<&str, Vec<Cube>> {
    separated_list1(line_ending, cube)(input)
}

fn cube(input: &str) -> IResult<&str, Cube> {
    map(separated_pair(nom_i8, nom_char(','),
                       separated_pair(nom_i8, nom_char(','), nom_i8)),
        |(x, (y, z))| Cube { x, y, z })(input)
}

#[cfg(test)]
mod test {
    use crate::day18::{Cube, cube, cubes, read_cubes, solve_part1, solve_part2};

    const EXAMPLE: &str = r"2,2,2
1,2,2
3,2,2
2,1,2
2,3,2
2,2,1
2,2,3
2,2,4
2,2,6
1,2,5
3,2,5
2,1,5
2,3,5";

    #[test]
    fn parse_cube() {
        assert_eq!(Ok(("", Cube { x: 1, y: 2, z: 3 })), cube("1,2,3"));
    }

    #[test]
    fn parse_cubes() {
        let (tail, cubes) = cubes(EXAMPLE).unwrap();

        assert_eq!(13, cubes.len());
        assert_eq!("", tail);
    }

    #[test]
    fn part1() {
        assert_eq!(64, solve_part1(&read_cubes(EXAMPLE)));
    }

    #[test]
    fn part2() {
        assert_eq!(58, solve_part2(&read_cubes(EXAMPLE)));
    }
}
