use std::cmp::max;
use std::collections::{HashMap, VecDeque};
use std::ops::{Add, Div, Rem, Sub};
use nom::bytes::complete::tag;
use nom::character::complete::{line_ending, u8 as nom_u8};
use nom::combinator::map;
use nom::IResult;
use nom::multi::separated_list1;
use nom::sequence::{delimited, separated_pair, tuple};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use yaah::*;


#[aoc(day19, part1)]
fn solve_part1(blueprints: &Vec<Blueprint>) -> u32 {
    blueprints
        .par_iter()
        .map(|blueprint| blueprint.quality_level(max_geodes(blueprint, 24)))
        .sum()
}

#[aoc(day19, part2)]
fn solve_part2(blueprints: &Vec<Blueprint>) -> u32 {
    blueprints[0..3]
        .par_iter()
        .map(|blueprint| blueprint.quality_level(max_geodes(blueprint, 32)))
        .inspect(|n|println!("N={n}"))
        .product()
}

fn max_geodes(blueprint: &Blueprint, minutes: u8) -> u8 {
    let mut queue = VecDeque::from([(0, State::default())]);
    let mut geode_cache: HashMap<u8, u8> = HashMap::from_iter((0..=minutes).into_iter().map(|m| (m, 0)));

    while let Some((minute, state)) = queue.pop_front() {
        let &winning = geode_cache.get(&minute).unwrap();

        if (state.geode + 2) < winning {
            continue;
        }
        geode_cache.insert(minute, winning.max(state.geode));

        if minute == minutes {
            continue;
        }

        //If we can build a geode bot every time Then math & Cache it out
        if let Some(next_state) = blueprint.build_geode_bot(&state) {

            if (next_state.ore_bots >= blueprint.geode.0 && next_state.obsidian_bots >= blueprint.geode.1) {
                (minute..=minutes).into_iter()
                    .fold(next_state, |s,m|{
                        let ns = blueprint.build_geode_bot(&s).unwrap();
                        let &best = geode_cache.get(&m).unwrap();
                        geode_cache.insert(m, best.max(ns.geode));
                        ns
                    });
            } else {
                queue.push_back((minute + 1, next_state));
            }

            continue;
        }

        let build_candidates: Vec<(Bot, State)> = vec![Bot::OBSIDIAN, Bot::CLAY, Bot::ORE]
            .into_iter()
            .filter(|bot| !state.skipped.contains(bot))
            .map(|bot| (bot, match bot {
                Bot::OBSIDIAN => blueprint.build_obsidian_bot(&state),
                Bot::CLAY => blueprint.build_clay_bot(&state),
                Bot::ORE => blueprint.build_ore_bot(&state),
            }))
            .filter(|(b_, state)| state.is_some())
            .map(|(bot_type, state)| (bot_type, state.unwrap()))
            .collect();

        let next_skipped: Vec<Bot> = build_candidates.iter()
            .map(|(bot, _)| *bot)
            .collect();
        queue.push_back((minute + 1, blueprint.build_nothing(&state, next_skipped)));

        build_candidates.into_iter()
            .for_each(|(_, state)| queue.push_back((minute + 1, state)));
    }

    *geode_cache.get(&minutes).unwrap()
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Bot { ORE, CLAY, OBSIDIAN }

#[derive(Debug, Eq, PartialEq)]
pub struct Blueprint {
    id: u8,
    ore: u8,
    clay: u8,
    obsidian: (u8, u8),
    geode: (u8, u8),
}

impl Blueprint {

    fn max_geodes(&self, minutes: u8) -> u8 {
        dfs(self, )
        0
    }
    fn max_ore(&self) -> u8 {
        max(max(self.ore, self.clay),
            max(self.obsidian.0, self.geode.0))
    }
    fn max_r(&self, minutes: u8) -> u32 {
        (minutes..0).into_iter()
            .fold((1, 0), |(bots, r), min|(bots, r + bots)).1
    }
    fn quality_level(&self, geodes: u8) -> u32 {
        self.id as u32 * geodes as u32
    }

    fn build_nothing(&self, state: &State, skipped: Vec<Bot>) -> State {
        State {
            ore: state.ore.add(state.ore_bots),
            clay: state.clay.add(state.clay_bots),
            obsidian: state.obsidian.add(state.obsidian_bots),
            geode: state.geode.add(state.geode_bots),
            skipped,
            ..*state
        }
    }
    fn build_ore_bot(&self, state: &State) -> Option<State> {
        if state.ore >= self.ore
            && state.ore_bots < self.max_ore() {
            Some(State {
                ore: state.ore.add(state.ore_bots).sub(self.ore),
                clay: state.clay.add(state.clay_bots),
                obsidian: state.obsidian.add(state.obsidian_bots),
                geode: state.geode.add(state.geode_bots),
                ore_bots: state.ore_bots + 1,
                skipped: vec![],
                ..*state
            })
        } else { None }
    }

    fn build_clay_bot(&self, state: &State) -> Option<State> {
        if state.ore >= self.clay
            && state.clay_bots < self.obsidian.1 {
            Some(State {
                ore: state.ore.add(state.ore_bots).sub(self.clay),
                clay: state.clay.add(state.clay_bots),
                obsidian: state.obsidian.add(state.obsidian_bots),
                geode: state.geode.add(state.geode_bots),
                clay_bots: state.clay_bots + 1,
                skipped: vec![],
                ..*state
            })
        } else { None }
    }

    fn build_obsidian_bot(&self, state: &State) -> Option<State> {
        if state.ore >= self.obsidian.0 && state.clay >= self.obsidian.1
            && state.obsidian_bots < self.geode.1 {
            Some(State {
                ore: state.ore.add(state.ore_bots).sub(self.obsidian.0),
                clay: state.clay.add(state.clay_bots).sub(self.obsidian.1),
                obsidian: state.obsidian.add(state.obsidian_bots),
                geode: state.geode.add(state.geode_bots),
                obsidian_bots: state.obsidian_bots + 1,
                skipped: vec![],
                ..*state
            })
        } else { None }
    }

    fn build_geode_bot(&self, state: &State) -> Option<State> {
        if state.ore >= self.geode.0 && state.obsidian >= self.geode.1 {
            Some(State {
                ore: state.ore.add(state.ore_bots).sub(self.geode.0),
                clay: state.clay.add(state.clay_bots),
                obsidian: state.obsidian.add(state.obsidian_bots).sub(self.geode.1),
                geode: state.geode.add(state.geode_bots),
                geode_bots: state.geode_bots + 1,
                skipped: vec![],
                ..*state
            })
        } else { None }
    }
}

#[derive(Debug, Clone)]
pub struct State {
    ore: u8,
    clay: u8,
    obsidian: u8,
    geode: u8,
    ore_bots: u8,
    clay_bots: u8,
    obsidian_bots: u8,
    geode_bots: u8,
    skipped: Vec<Bot>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            ore: 0,
            clay: 0,
            obsidian: 0,
            geode: 0,
            ore_bots: 1,
            clay_bots: 0,
            obsidian_bots: 0,
            geode_bots: 0,
            skipped: vec![],
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
    use crate::day19::{Blueprint, max_geodes, read_blueprints, solve_part1};

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
        assert_eq!(9, max_geodes(&blueprint, 24))
    }
    #[test]
    fn blueprint2_part1() {
        let blueprint = Blueprint {
            id: 2,
            ore: 2,
            clay: 3,
            obsidian: (3, 8),
            geode: (3, 12),
        };
        assert_eq!(12, max_geodes(&blueprint, 24))
    }

    #[test]
    fn blueprint1_part2() {
        let blueprint = Blueprint {
            id: 1,
            ore: 4,
            clay: 2,
            obsidian: (3, 14),
            geode: (2, 7),
        };
        assert_eq!(56, max_geodes(&blueprint, 32))
    }
    #[test]
    fn blueprint2_part2() {
        let blueprint = Blueprint {
            id: 2,
            ore: 2,
            clay: 3,
            obsidian: (3, 8),
            geode: (3, 12),
        };
        assert_eq!(62, max_geodes(&blueprint, 32))
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