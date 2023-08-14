use std::cmp;
use std::collections::{HashMap, HashSet};
use nom::character::complete::{char as nom_char, line_ending};
use nom::{IResult, Parser};
use nom::character::complete::u8 as nom_u8;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use yaah::*;


#[aoc_generator(day18)]
fn read_cubes(input: &'static str) -> Vec<Cube> {
    cubes(input).unwrap().1
}

#[aoc(day18, part1)]
fn solve_part1(cubes: &Vec<Cube>) -> u32 {
    let cube_set: HashSet<Cube> = HashSet::from_iter(cubes.into_iter().map(|&cube| cube));
    cube_set.iter()
        .map(|cube| cube.adjacent()
            .into_iter()
            .filter(|other| cube_set.contains(other))
            .count())
        .map(|neighbors| (6 - neighbors) as u32)
        .sum()
}

#[aoc(day18, part2)]
fn solve_part2(cubes: &Vec<Cube>) -> u32 {
    let cube_set: HashSet<Cube> = HashSet::from_iter(cubes.into_iter().map(|&cube| cube));
    let (xmax, ymax, zmax) = cube_set.iter()
        .fold((0u8, 0u8, 0u8), |(x, y, z), cube| (cmp::max(x, cube.x), cmp::max(y, cube.y), cmp::max(z, cube.z)));
    let (xmin, ymin, zmin) = cube_set.iter()
        .fold((xmax, ymax, zmax), |(x, y, z), cube| (cmp::min(x, cube.x), cmp::min(y, cube.y), cmp::min(z, cube.z)));
    dbg!((xmax, ymax, zmax), (xmin, ymin, zmin));
    /*
    Cube 20*20*20
    cached empty space?

    Center the cube (maybe a translation in lookups?)
    fill function to fill from the edges?
    x,y,z => reachable? True, false or unknown
    unknown = no entry in map?

    Confirmed empty



     */
    cube_set.iter()
        .map(|cube| (cube, cube.adjacent()
            .into_iter()
            .filter(|other| cube_set.contains(other))
            .count()))
        .map(|(cube, neighbors)| (6 - neighbors) as u32)
        .sum()
}

#[derive(Eq, PartialEq, Debug, Hash, Copy, Clone)]
pub struct Cube {
    x: u8,
    y: u8,
    z: u8,
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
            .filter_map(|t| self.translate(t))
            .collect()
    }
    fn neighbors(&self, field: &HashMap<Cube, u32>) -> Vec<Cube> {
        self.adjacent().into_iter()
            .filter(|other| field.contains_key(other))
            .collect()
    }

    fn translate(&self, (xdiff, ydiff, zdiff): (i32, i32, i32)) -> Option<Cube> {
        let x = xdiff + (self.x as i32);
        let y = ydiff + (self.y as i32);
        let z = zdiff + (self.z as i32);

        Cube::try_from((x, y, z)).ok()
    }
}

impl TryFrom<(i32, i32, i32)> for Cube {
    type Error = &'static str;

    fn try_from(value: (i32, i32, i32)) -> Result<Self, Self::Error> {
        match (u8::try_from(value.0), u8::try_from(value.1), u8::try_from(value.2)) {
            (Ok(x), Ok(y), Ok(z)) => Ok(Cube { x, y, z }),
            _ => Err("Cube Negative coordinates not supported")
        }
    }
}

fn cubes(input: &str) -> IResult<&str, Vec<Cube>> {
    separated_list1(line_ending, cube)(input)
}

fn cube(input: &str) -> IResult<&str, Cube> {
    separated_pair(nom_u8, nom_char(','),
                   separated_pair(nom_u8, nom_char(','), nom_u8))
        .parse(input)
        .map(|(tail, (x, (y, z)))| (tail, Cube { x, y, z }))
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