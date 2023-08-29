use std::cmp::max;
use std::collections::VecDeque;
use std::ops::{Add, Div, Rem, Sub};
use nom::bytes::complete::tag;
use nom::character::complete::{line_ending, u8 as nom_u8};
use nom::combinator::map;
use nom::IResult;
use nom::multi::separated_list1;
use nom::sequence::{delimited, separated_pair, tuple};
use yaah::*;


#[aoc(day19, part1)]
fn solve_part1(blueprints: &Vec<Blueprint>) -> u32 {
    blueprints.iter()
        .map(|blueprint| blueprint.quality_level(max_geodes(blueprint, 24)))
        .inspect(|ql| println!("{ql}"))
        .sum()
}

fn max_geodes(blueprint: &Blueprint, minutes: u8) -> u8 {
    let mut futures = VecDeque::from([State::initial(minutes)]);
    let mut max_geodes = 0u8;

    while let Some(state) = futures.pop_front() {
        // let geo = state.next_geode_bot(blueprint)
        //If a geode or obsidian is possible before anything else do it!
        let paths: Vec<State> = vec![
            state.next_geode_bot(blueprint),
            state.next_obsidian_bot(blueprint),
            state.next_clay_bot(blueprint),
            state.next_ore_bot(blueprint),
        ].into_iter()
            .filter_map(|x| x.ok())
            .collect();

        if paths.is_empty() {
            max_geodes = max(max_geodes, state.geode);
            // println!("MaxGEODES:{max_geodes}")
        }
        for future in paths {
            futures.push_back(future)
        }
        // println!("{:?}", futures.len())
    }


    max_geodes
}

fn div_ceil(dividend: u8, divisor: u8) -> u8{
    dividend / divisor + match dividend % divisor {
        0 => 0,
        _ => 1
    }
}
#[derive(Debug, Eq, PartialEq)]
pub struct Blueprint {
    id: u8,
    ore: u8,
    clay: u8,
    obsidian: (u8, u8),
    geode: (u8, u8),
}

impl Blueprint {
    fn max_ore(&self) -> u8 {
        max(max(self.ore, self.clay),
            max(self.obsidian.0, self.geode.0))
    }

    fn quality_level(&self, geodes: u8) -> u32 {
        self.id as u32 * geodes as u32
    }
}

#[derive(Debug)]
pub struct State {
    minutes_remaining: u8,
    ore: u8,
    clay: u8,
    obsidian: u8,
    geode: u8,
    ore_bots: u8,
    clay_bots: u8,
    obsidian_bots: u8,
}

impl State {
    fn initial(minutes: u8) -> Self {
        Self {
            minutes_remaining: minutes,
            ore: 0,
            clay: 0,
            obsidian: 0,
            geode: 0,
            ore_bots: 1,
            clay_bots: 0,
            obsidian_bots: 0,
        }
    }

    /// bots == 0 => Error or None
    /// Already have enough => 1
    /// Not enough need / bots (round up)
    fn next_bot(&self, resources:u8, cost:u8, bots:u8) -> Result<u8, ()>{
        if bots == 0 {
            return Err(())
        }
        let minutes = match cost.checked_sub(resources) {
            None => 1,
            Some(needed) => max(div_ceil(needed, bots), 1)
        };
        match minutes < self.minutes_remaining {
            true => Ok(minutes),
            false => Err(())
        }
    }
    fn next_bott(&self, resources:u8, cost:u8, bots:u8) -> Option<u8>{
        if bots == 0 {
            return None
        }
        let minutes = match cost.checked_sub(resources) {
            None => 1,
            Some(needed) => max(div_ceil(needed, bots), 1)
        };
        match minutes < self.minutes_remaining {
            true => Some(minutes),
            false => None
        }
    }

    fn next_geode(&self, blueprint: &Blueprint) -> Result<u8, ()>  {
        let ore_minutes = self.next_bot(self.ore, blueprint.geode.0, self.ore_bots);
        let obsidian_minutes = self.next_bot(self.obsidian, blueprint.geode.1, self.obsidian_bots);
        if ore_minutes.is_ok() && obsidian_minutes.is_ok(){
            Ok(max(ore_minutes?, obsidian_minutes?))
        } else {
            Err(())
        }
    }

    /// Next
    fn next_geode_bot(&self, blueprint: &Blueprint) -> Result<State, ()> {
        match self.next_geode(blueprint) {
            Err(_) => Err(()),
            Ok(minutes) => Ok(State {
                minutes_remaining: self.minutes_remaining - minutes,
                ore: self.ore + (self.ore_bots * minutes) - blueprint.geode.0,
                clay: self.clay + (self.clay_bots * minutes),
                obsidian: self.obsidian + (self.obsidian_bots * minutes) - blueprint.geode.1,
                geode: self.geode + (self.minutes_remaining - minutes),

                ore_bots: self.ore_bots,
                clay_bots: self.clay_bots,
                obsidian_bots: self.obsidian_bots,
            })
        }
    }

    fn next_obsidian(&self, blueprint: &Blueprint) -> Result<u8, ()> {
        let ore_minutes = self.next_bot(self.ore, blueprint.obsidian.0, self.ore_bots);
        let clay_minutes = self.next_bot(self.clay, blueprint.obsidian.1, self.clay_bots);
        if ore_minutes.is_ok() && clay_minutes.is_ok() && self.obsidian_bots < blueprint.geode.1 {
            Ok(max(ore_minutes?, clay_minutes?))
        } else {
            Err(())
        }
    }
    fn next_obsidian_bot(&self, blueprint: &Blueprint) -> Result<State, ()> {
        if let Some(minutes) = self.next_obsidian(blueprint).ok() {
            let clay = self.clay + (self.clay_bots * minutes);
            match minutes >= self.minutes_remaining {
                true => Err(()),
                false => Ok(State {
                    minutes_remaining: self.minutes_remaining - minutes,
                    ore: self.ore + (self.ore_bots * minutes) - blueprint.obsidian.0,
                    clay: clay - blueprint.obsidian.1,
                    obsidian: self.obsidian + (self.obsidian_bots * minutes),
                    geode: self.geode,
                    ore_bots: self.ore_bots,
                    clay_bots: self.clay_bots,
                    obsidian_bots: self.obsidian_bots + 1,
                })
            }
        } else {
            Err(())
        }
    }
    fn next_clay(&self, blueprint: &Blueprint) -> Result<u8, ()> {
        match self.clay_bots < blueprint.obsidian.1 {
            true =>  self.next_bot(self.ore, blueprint.clay, self.ore_bots),
            false => Err(())
        }
    }
    fn next_clay_bot(&self, blueprint: &Blueprint) -> Result<State, ()> {
        let minutes = self.next_clay(blueprint)?;
        match minutes >= self.minutes_remaining {
            true => Err(()),
            false => Ok(State {
                minutes_remaining: self.minutes_remaining - minutes,
                ore: (self.ore + (self.ore_bots * minutes)) - blueprint.clay,
                clay: self.clay + (self.clay_bots * minutes),
                obsidian: self.obsidian + (self.obsidian_bots * minutes),
                geode: self.geode,
                ore_bots: self.ore_bots,
                clay_bots: self.clay_bots + 1,
                obsidian_bots: self.obsidian_bots,
            })
        }
    }
    fn next_ore(&self, blueprint: &Blueprint) -> Result<u8, ()> {
        match self.ore_bots < blueprint.max_ore() {
            true => self.next_bot(self.ore, blueprint.ore, self.ore_bots),
            false => Err(())
        }
    }
    fn next_orre(&self, blueprint: &Blueprint) -> Option<u8> {
        match self.ore_bots < blueprint.max_ore() {
            true => self.next_bott(self.ore, blueprint.ore, self.ore_bots),
            false => None
        }
    }
    fn next_ore_bot(&self, blueprint: &Blueprint) -> Result<State, ()> {
        match self.next_ore(blueprint) {
            Err(_) => Err(()),
            Ok(minutes) => Ok(State {
                minutes_remaining: self.minutes_remaining - minutes,
                ore: (self.ore + (self.ore_bots * minutes)) - blueprint.ore,
                clay: self.clay + (self.clay_bots * minutes),
                obsidian: self.obsidian + (self.obsidian_bots * minutes),
                geode: self.geode,
                ore_bots: self.ore_bots + 1,
                clay_bots: self.clay_bots,
                obsidian_bots: self.obsidian_bots,
            })
        }
    }
}

#[aoc_generator(day19)]
fn read_blueprints(input: &'static str) -> Vec<Blueprint> {
    blueprints(input).unwrap().1
}

fn blueprints(input: &str) -> IResult<&str, Vec<Blueprint>> {
    separated_list1(line_ending, blueprint)(input)
}

fn blueprint(input: &str) -> IResult<&str, Blueprint> {
    map(tuple((blueprint_id, ore_cost, clay_cost, obsidian_cost, geode_cost)),
        |(id, ore, clay, obsidian, geode)| Blueprint {
            id,
            ore,
            clay,
            obsidian,
            geode,
        })(input)
}

fn blueprint_id(input: &str) -> IResult<&str, u8> {
    delimited(tag("Blueprint "), nom_u8, tag(": "))(input)
}

fn ore_cost(input: &str) -> IResult<&str, u8> {
    delimited(tag("Each ore robot costs "), nom_u8, tag(" ore. "))(input)
}

fn clay_cost(input: &str) -> IResult<&str, u8> {
    delimited(tag("Each clay robot costs "), nom_u8, tag(" ore. "))(input)
}

fn obsidian_cost(input: &str) -> IResult<&str, (u8, u8)> {
    delimited(tag("Each obsidian robot costs "), separated_pair(nom_u8, tag(" ore and "), nom_u8), tag(" clay. "))(input)
}

fn geode_cost(input: &str) -> IResult<&str, (u8, u8)> {
    delimited(tag("Each geode robot costs "), separated_pair(nom_u8, tag(" ore and "), nom_u8), tag(" obsidian."))(input)
}


#[cfg(test)]
mod test {
    use crate::day19::{Blueprint, blueprints, max_geodes, read_blueprints, solve_part1, State};

    const EXAMPLE: &str = r"Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.
Blueprint 2: Each ore robot costs 2 ore. Each clay robot costs 3 ore. Each obsidian robot costs 3 ore and 8 clay. Each geode robot costs 3 ore and 12 obsidian.";

    #[test]
    fn part1() {
        let blueprints = read_blueprints(EXAMPLE);
        assert_eq!(33, solve_part1(&blueprints))
    }

    #[test]
    fn blueprint1() {
        let blueprint = Blueprint {
            id: 1,
            ore: 4,
            clay: 2,
            obsidian: (3, 14),
            geode: (2, 7),
        };
        assert_eq!(9, max_geodes(&blueprint, 23))
    }

    #[test]
    fn next(){
        let state = State::initial(24);
        let blueprint = Blueprint {
            id: 1,
            ore: 4,
            clay: 2,
            obsidian: (3, 14),
            geode: (2, 7),
        };

        assert_eq!(Ok(4), state.next_ore(&blueprint));
        assert_eq!(Ok(2), state.next_clay(&blueprint));
        assert_eq!(Err(()), state.next_obsidian(&blueprint));
        // assert_eq!(Ok(2), state.next_(&blueprint));
    }

    #[test]
    fn example_input() {
        let expected = vec![
            Blueprint {
                id: 1,
                ore: 4,
                clay: 2,
                obsidian: (3, 14),
                geode: (2, 7),
            },
            Blueprint {
                id: 2,
                ore: 2,
                clay: 3,
                obsidian: (3, 8),
                geode: (3, 12),
            },
        ];

        let blueprints = read_blueprints(EXAMPLE);
        assert_eq!(expected, blueprints);
    }
}