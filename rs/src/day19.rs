use yaah::*;

#[aoc_generator(day19)]
fn read_blueprints(input: &'static str) -> Vec<Blueprint> {
    vec![]
}

#[aoc(day19, part1)]
fn solve_part1(blueprints: &Vec<Blueprint>) -> u32 {
    0
}

struct Blueprint {
    ore_cost: u32,
    clay_cost: u32,
    obsidian_cost: (u32, u32),
    geode_cost: (u32, u32),
}

#[cfg(test)]
mod test {

}