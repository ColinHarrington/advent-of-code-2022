use yaah::*;

#[aoc_generator(day24)]
fn parse_basin(input: &'static str) -> () {}

#[aoc(day24, part1)]
fn solve_part1() -> u32 {
    0
}

struct Basin {
    width: u32,
    height: u32,
}

enum Step {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    WAIT,
}
