use bitflags::bitflags;
use nom::bytes::complete::tag;
use nom::character::complete::{line_ending, u16 as nom_u16};
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::{delimited, separated_pair, tuple};
use nom::IResult;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::cmp::max;
use std::convert::identity;
use std::fmt::{Display, Formatter};
use std::iter;
use std::ops::{Add, Div, Mul, Sub};
use yaah::*;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct Flag: u8 {
        const NEED_ORE = 1;
        const NEED_CLAY = 1 << 1;
        const NEED_OBSIDIAN = 1 << 2;
    }
}

#[aoc(day19, part1)]
fn solve_part1(blueprints: &Vec<Blueprint>) -> u32 {
    blueprints
        .par_iter()
        .map(|blueprint| blueprint.quality_level(blueprint.max_geodes(24)))
        .sum()
}

#[aoc(day19, part2)]
fn solve_part2(blueprints: &Vec<Blueprint>) -> u32 {
    blueprints[0..3]
        .par_iter()
        .map(|blueprint| blueprint.max_geodes(32) as u32)
        .product()
}

fn dfs(blueprint: &Blueprint, state: State) -> u16 {
    let next_states: Vec<State> = vec![
        blueprint.next_obsidian_bot(&state),
        blueprint.next_clay_bot(&state),
        blueprint.next_ore_bot(&state),
    ]
    .into_iter()
    .filter_map(identity)
    .collect();
    if let Some(next_geode_state) = blueprint.next_geode_bot(&state) {
        let threshold = next_geode_state.minutes_remaining;
        next_states
            .into_iter()
            .filter(|next_state| next_state.minutes_remaining > threshold)
            .chain(iter::once(next_geode_state))
            .map(|next_state| {
                if next_state.minutes_remaining > 0 {
                    dfs(blueprint, next_state)
                } else {
                    next_state.geode
                }
            })
            .max()
    } else {
        next_states
            .into_iter()
            .map(|next_state| dfs(blueprint, next_state))
            .max()
    }
    .unwrap_or(state.geode)
}

fn next_bot(have: u16, cost: u16, bots: u16) -> Option<u16> {
    match bots {
        0 => None,
        _ => match cost.checked_sub(have) {
            None => Some(1),
            Some(needed) => match needed.add(bots).sub(1).div(bots) {
                0 => Some(1),
                n => Some(n + 1),
            },
        },
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Blueprint {
    id: u16,
    ore_cost: u16,
    clay_cost: u16,
    obsidian_cost: (u16, u16),
    geode_cost: (u16, u16),
}

impl Blueprint {
    fn max_geodes(&self, minutes: u16) -> u16 {
        dfs(self, State::initial(minutes))
    }
    fn max_ore(&self) -> u16 {
        max(
            max(self.ore_cost, self.clay_cost),
            max(self.obsidian_cost.0, self.geode_cost.0),
        )
    }
    fn quality_level(&self, geodes: u16) -> u32 {
        self.id as u32 * geodes as u32
    }

    fn next_ore_bot(&self, state: &State) -> Option<State> {
        if state.flags.contains(Flag::NEED_ORE) {
            if let Some(minutes) = next_bot(state.ore, self.ore_cost, state.ore_bots) {
                if let Some(minutes_remaining) = state.minutes_remaining.checked_sub(minutes) {
                    if minutes_remaining > 0 {
                        let ore_bots = state.ore_bots + 1;
                        let flags: Flag = if ore_bots >= self.max_ore() {
                            state.flags - Flag::NEED_ORE
                        } else {
                            state.flags
                        };
                        Some(State {
                            ore: state
                                .ore
                                .add(state.ore_bots.mul(minutes))
                                .sub(self.ore_cost),
                            clay: state.clay.add(state.clay_bots.mul(minutes)),
                            obsidian: state.obsidian.add(state.obsidian_bots.mul(minutes)),
                            ore_bots,
                            minutes_remaining,
                            flags,
                            ..*state
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn next_clay_bot(&self, state: &State) -> Option<State> {
        if state.flags.contains(Flag::NEED_CLAY) {
            if let Some(minutes) = next_bot(state.ore, self.clay_cost, state.ore_bots) {
                if let Some(minutes_remaining) = state.minutes_remaining.checked_sub(minutes) {
                    if minutes_remaining > 0 {
                        let clay_bots = state.clay_bots + 1;
                        let flags: Flag = if clay_bots >= self.obsidian_cost.1 {
                            state.flags - Flag::NEED_CLAY
                        } else {
                            state.flags
                        };
                        Some(State {
                            ore: state
                                .ore
                                .add(state.ore_bots.mul(minutes))
                                .sub(self.clay_cost),
                            clay: state.clay.add(state.clay_bots.mul(minutes)),
                            obsidian: state.obsidian.add(state.obsidian_bots.mul(minutes)),
                            clay_bots,
                            minutes_remaining,
                            flags,
                            ..*state
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn next_obsidian_bot(&self, state: &State) -> Option<State> {
        if state.flags.contains(Flag::NEED_OBSIDIAN) {
            if let Some(ore_minutes) = next_bot(state.ore, self.obsidian_cost.0, state.ore_bots) {
                if let Some(clay_minutes) =
                    next_bot(state.clay, self.obsidian_cost.1, state.clay_bots)
                {
                    let minutes = clay_minutes.max(ore_minutes);
                    if let Some(minutes_remaining) = state.minutes_remaining.checked_sub(minutes) {
                        if minutes_remaining > 0 {
                            let obsidian_bots = state.obsidian_bots + 1;
                            let flags: Flag = if obsidian_bots >= self.geode_cost.1 {
                                state.flags - Flag::NEED_OBSIDIAN
                            } else {
                                state.flags
                            };
                            Some(State {
                                ore: state
                                    .ore
                                    .add(state.ore_bots.mul(minutes))
                                    .sub(self.obsidian_cost.0),
                                clay: state
                                    .clay
                                    .add(state.clay_bots.mul(minutes))
                                    .sub(self.obsidian_cost.1),
                                obsidian: state.obsidian.add(state.obsidian_bots.mul(minutes)),
                                obsidian_bots,
                                minutes_remaining,
                                flags,
                                ..*state
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn next_geode_bot(&self, state: &State) -> Option<State> {
        if let Some(ore_minutes) = next_bot(state.ore, self.geode_cost.0, state.ore_bots) {
            if let Some(obsidian_minutes) =
                next_bot(state.obsidian, self.geode_cost.1, state.obsidian_bots)
            {
                let minutes = obsidian_minutes.max(ore_minutes);
                if let Some(minutes_remaining) = state.minutes_remaining.checked_sub(minutes) {
                    if minutes_remaining > 0 {
                        Some(State {
                            ore: state
                                .ore
                                .add(state.ore_bots.mul(minutes))
                                .sub(self.geode_cost.0),
                            clay: state.clay.add(state.clay_bots.mul(minutes)),
                            obsidian: state
                                .obsidian
                                .add(state.obsidian_bots.mul(minutes))
                                .sub(self.geode_cost.1),
                            geode: state.geode.add(minutes_remaining),
                            minutes_remaining,
                            ..*state
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct State {
    ore: u16,
    clay: u16,
    obsidian: u16,
    geode: u16,
    ore_bots: u16,
    clay_bots: u16,
    obsidian_bots: u16,
    flags: Flag,
    minutes_remaining: u16,
}

impl State {
    fn initial(minutes_remaining: u16) -> Self {
        Self {
            ore: 0,
            clay: 0,
            obsidian: 0,
            geode: 0,
            ore_bots: 1,
            clay_bots: 0,
            obsidian_bots: 0,
            flags: Flag::NEED_ORE | Flag::NEED_CLAY | Flag::NEED_OBSIDIAN,
            minutes_remaining,
        }
    }

    #[cfg(feature = "debug")]
    fn print(&self) {
        if self.ore_bots > 0 {
            let robot = if self.ore_bots > 1 { "robots" } else { "robot" };
            println!(
                "{} ore-collecting {robot} collects {} ore; you now have {} ore.",
                self.ore_bots, self.ore_bots, self.ore
            )
        }
        if self.clay_bots > 0 {
            let robot = if self.clay_bots > 1 {
                "robots"
            } else {
                "robot"
            };
            println!(
                "{} clay-collecting {robot} collect {} clay; you now have {} clay.",
                self.clay_bots, self.clay_bots, self.clay
            )
        }
        if self.obsidian_bots > 0 {
            let robot = if self.obsidian_bots > 1 {
                "robots"
            } else {
                "robot"
            };
            println!(
                "{} obsidian-collecting {robot} collect {} obsidian; you now have {} obsidian.",
                self.obsidian_bots, self.obsidian_bots, self.obsidian
            )
        }
        if self.geode > 0 {
            println!(
                "X geode-cracking robots crack X geodes; you now have {} open geodes.",
                self.geode
            )
        }
    }
    #[cfg(feature = "debug")]
    fn advance(&self) -> State {
        State {
            ore: self.ore.add(self.ore_bots),
            clay: self.clay.add(self.clay_bots),
            obsidian: self.obsidian.add(self.obsidian_bots),
            minutes_remaining: self.minutes_remaining.sub(1),
            ..*self
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "t={}, [{}, {}, {}, {}] ({},{},{}) |{:?}| ",
            self.minutes_remaining,
            self.geode,
            self.obsidian,
            self.clay,
            self.ore,
            self.obsidian_bots,
            self.clay_bots,
            self.ore_bots,
            self.flags
        )
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
    map(
        tuple((blueprint_id, ore_cost, clay_cost, obsidian_cost, geode_cost)),
        |(id, ore, clay, obsidian, geode)| Blueprint {
            id,
            ore_cost: ore,
            clay_cost: clay,
            obsidian_cost: obsidian,
            geode_cost: geode,
        },
    )(input)
}

fn blueprint_id(input: &str) -> IResult<&str, u16> {
    delimited(tag("Blueprint "), nom_u16, tag(": "))(input)
}

fn ore_cost(input: &str) -> IResult<&str, u16> {
    delimited(tag("Each ore robot costs "), nom_u16, tag(" ore. "))(input)
}

fn clay_cost(input: &str) -> IResult<&str, u16> {
    delimited(tag("Each clay robot costs "), nom_u16, tag(" ore. "))(input)
}

fn obsidian_cost(input: &str) -> IResult<&str, (u16, u16)> {
    delimited(
        tag("Each obsidian robot costs "),
        separated_pair(nom_u16, tag(" ore and "), nom_u16),
        tag(" clay. "),
    )(input)
}

fn geode_cost(input: &str) -> IResult<&str, (u16, u16)> {
    delimited(
        tag("Each geode robot costs "),
        separated_pair(nom_u16, tag(" ore and "), nom_u16),
        tag(" obsidian."),
    )(input)
}

#[cfg(test)]
mod test {
    use crate::day19::{dfs, next_bot, read_blueprints, solve_part1, Blueprint, Flag, State};
    #[cfg(feature = "debug")]
    use std::collections::HashMap;

    const EXAMPLE: &str = r"Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.
Blueprint 2: Each ore robot costs 2 ore. Each clay robot costs 3 ore. Each obsidian robot costs 3 ore and 8 clay. Each geode robot costs 3 ore and 12 obsidian.";

    #[cfg(feature = "debug")]
    #[derive(Debug, Eq, PartialEq)]
    enum Bot {
        ORE,
        CLAY,
        OBSIDIAN,
        GEODE,
    }

    #[test]
    fn part1() {
        let blueprints = read_blueprints(EXAMPLE);
        assert_eq!(33, solve_part1(&blueprints))
    }

    #[test]
    fn blueprint1() {
        let blueprint = Blueprint {
            id: 1,
            ore_cost: 4,
            clay_cost: 2,
            obsidian_cost: (3, 14),
            geode_cost: (2, 7),
        };
        assert_eq!(9, blueprint.max_geodes(24))
    }

    /// Test that helped iron out corner cases
    #[cfg(feature = "debug")]
    #[test]
    fn blueprint1_walkthrough() {
        let blueprint = Blueprint {
            id: 1,
            ore_cost: 4,
            clay_cost: 2,
            obsidian_cost: (3, 14),
            geode_cost: (2, 7),
        };

        let path = vec![
            Bot::CLAY,
            Bot::CLAY,
            Bot::CLAY,
            Bot::OBSIDIAN,
            Bot::CLAY,
            Bot::OBSIDIAN,
            Bot::GEODE,
            Bot::GEODE,
        ];
        let initial_state = State::initial(24);
        let mut scenario: HashMap<u16, State> =
            HashMap::from([(initial_state.minutes_remaining, initial_state.clone())]);
        let final_state = path.into_iter().fold(initial_state.clone(), |state, bot| {
            let next_state = match bot {
                Bot::ORE => blueprint.next_ore_bot(&state).unwrap(),
                Bot::CLAY => blueprint.next_clay_bot(&state).unwrap(),
                Bot::OBSIDIAN => blueprint.next_obsidian_bot(&state).unwrap(),
                Bot::GEODE => blueprint.next_geode_bot(&state).unwrap(),
            };
            scenario.insert(next_state.minutes_remaining, next_state.clone());
            next_state
        });

        for (min, remaining) in (0u16..=24).into_iter().rev().enumerate() {
            println!("== Minute {min} ==");
            if let Some(state) = scenario.get(&remaining) {
                state.print();
            } else {
                let previous = remaining + 1;
                let state = scenario.get(&previous).unwrap().advance();
                scenario.insert(remaining, state.clone());
                state.print();
            }
            println!("");
        }

        assert_eq!(9, final_state.geode);
    }

    #[test]
    fn blueprint2_part1() {
        let blueprint = Blueprint {
            id: 2,
            ore_cost: 2,
            clay_cost: 3,
            obsidian_cost: (3, 8),
            geode_cost: (3, 12),
        };
        assert_eq!(12, blueprint.max_geodes(24))
    }

    #[test]
    fn blueprint1_part2() {
        let blueprint = Blueprint {
            id: 1,
            ore_cost: 4,
            clay_cost: 2,
            obsidian_cost: (3, 14),
            geode_cost: (2, 7),
        };
        assert_eq!(56, blueprint.max_geodes(32))
    }

    #[test]
    fn blueprint2_part2() {
        let blueprint = Blueprint {
            id: 2,
            ore_cost: 2,
            clay_cost: 3,
            obsidian_cost: (3, 8),
            geode_cost: (3, 12),
        };
        assert_eq!(62, blueprint.max_geodes(32))
    }

    #[test]
    fn example_input() {
        let expected = vec![
            Blueprint {
                id: 1,
                ore_cost: 4,
                clay_cost: 2,
                obsidian_cost: (3, 14),
                geode_cost: (2, 7),
            },
            Blueprint {
                id: 2,
                ore_cost: 2,
                clay_cost: 3,
                obsidian_cost: (3, 8),
                geode_cost: (3, 12),
            },
        ];

        let blueprints = read_blueprints(EXAMPLE);
        assert_eq!(expected, blueprints);
    }

    #[test]
    fn test_next_bot() {
        assert_eq!(None, next_bot(0, 0, 0));
        assert_eq!(None, next_bot(10, 1, 0));
        assert_eq!(Some(1), next_bot(10, 2, 1));
        assert_eq!(Some(5), next_bot(0, 4, 1));
        assert_eq!(Some(4), next_bot(1, 4, 1));
        assert_eq!(Some(3), next_bot(2, 4, 1));
    }

    #[test]
    fn simple_dfs() {
        let blueprint = Blueprint {
            id: 1,
            ore_cost: 4,
            clay_cost: 2,
            obsidian_cost: (3, 14),
            geode_cost: (2, 7),
        };
        let state = State {
            ore: 3,
            clay: 13,
            obsidian: 8,
            geode: 0,
            ore_bots: 1,
            clay_bots: 4,
            obsidian_bots: 2,
            flags: Flag::NEED_ORE | Flag::NEED_CLAY | Flag::NEED_OBSIDIAN,
            minutes_remaining: 7,
        };
        assert_eq!(9, dfs(&blueprint, state));
    }
}
