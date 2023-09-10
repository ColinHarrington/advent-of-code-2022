use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{i32 as nom_i32, newline};
use nom::multi::separated_list1;
use nom::sequence::preceded;
use nom::IResult;
use yaah::*;

#[aoc_generator(day10)]
fn gen(input: &'static str) -> Vec<Instruction> {
    parse_instructions(input).unwrap().1
}

#[aoc(day10, part1)]
fn solve_part1(instructions: &Vec<Instruction>) -> i32 {
    let mut x = 1;
    let mut cycle = 1;
    let mut signal_strength = 0;
    for instruction in instructions {
        signal_strength = boost_signal(signal_strength, cycle, x);
        if let Instruction::ADDX(v) = instruction {
            cycle += 1;
            signal_strength = boost_signal(signal_strength, cycle, x);
            cycle += 1;
            x += v;
        } else {
            cycle += 1;
        }
    }
    signal_strength
}

#[aoc(day10, part2)]
fn solve_part2(instructions: &Vec<Instruction>) -> String {
    let mut cycle = 1;
    let mut sprite = 0;
    let mut image: Vec<char> = vec![];
    for instruction in instructions {
        image.push(pixel(cycle, sprite));
        if let Instruction::ADDX(v) = instruction {
            cycle += 1;
            image.push(pixel(cycle, sprite));
            cycle += 1;
            sprite += v;
        } else {
            cycle += 1;
        }
    }
    image
        .chunks(40)
        .map(|line| line.iter().collect::<String>())
        .join("\n")
}

#[cfg(feature = "debug")]
fn draw(cycle: i32, sprite: i32) {
    let n = cycle % 40;
    let position = (cycle - 1) % 40;
    let c = match lit(position, sprite) {
        true => '#',
        false => '.',
    };
    match n {
        0 => print!("{c}\n"),
        _ => print!("{c}"),
    }
}

fn pixel(cycle: i32, sprite: i32) -> char {
    let position = (cycle - 1) % 40;
    match lit(position, sprite) {
        true => '#',
        false => '.',
    }
}

fn lit(n: i32, sprite: i32) -> bool {
    ((sprite)..(sprite + 3)).contains(&n)
}

fn boost_signal(signal: i32, cycle: i32, x: i32) -> i32 {
    match cycle % 40 {
        20 => signal + cycle * x,
        _ => signal,
    }
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    NOOP,
    ADDX(i32),
}

fn parse_instructions(input: &str) -> IResult<&str, Vec<Instruction>> {
    separated_list1(newline, parse_instruction)(input)
}

fn parse_instruction(input: &str) -> IResult<&str, Instruction> {
    alt((parse_noop, parse_addx))(input)
}

fn parse_noop(input: &str) -> IResult<&str, Instruction> {
    let (input, _) = tag("noop")(input)?;
    Ok((input, Instruction::NOOP))
}

fn parse_addx(input: &str) -> IResult<&str, Instruction> {
    let (input, value) = preceded(tag("addx "), nom_i32)(input)?;
    Ok((input, Instruction::ADDX(value)))
}

#[cfg(test)]
mod test {
    use crate::day10::{
        gen, parse_addx, parse_instruction, parse_instructions, parse_noop, solve_part1,
        solve_part2, Instruction,
    };

    const SMALL_EXAMPLE: &str = r"noop
addx 3
addx -5";

    #[test]
    fn noop() {
        assert_eq!(Ok(("", Instruction::NOOP)), parse_noop("noop"));
    }

    #[test]
    fn addx() {
        assert_eq!(Ok(("", Instruction::ADDX(77))), parse_addx("addx 77"));
    }

    #[test]
    fn instruction() {
        assert_eq!(Ok(("", Instruction::NOOP)), parse_instruction("noop"));
        assert_eq!(
            Ok(("", Instruction::ADDX(77))),
            parse_instruction("addx 77")
        );
    }

    #[test]
    fn instructions() {
        let (_, instructions) = parse_instructions(SMALL_EXAMPLE).unwrap();

        let expected_instructions = vec![
            Instruction::NOOP,
            Instruction::ADDX(3),
            Instruction::ADDX(-5),
        ];

        instructions
            .iter()
            .zip(expected_instructions.iter())
            .for_each(|(instruction, expected)| assert_eq!(instruction, expected));
    }

    #[test]
    fn part1() {
        assert_eq!(0, solve_part1(&gen(SMALL_EXAMPLE)));

        assert_eq!(13140, solve_part1(&gen(LARGE_EXAMPLE)));
    }

    #[test]
    fn part2() {
        let expected = r"##..##..##..##..##..##..##..##..##..##..
###...###...###...###...###...###...###.
####....####....####....####....####....
#####.....#####.....#####.....#####.....
######......######......######......####
#######.......#######.......#######....."
            .to_string();
        assert_eq!(expected, solve_part2(&gen(LARGE_EXAMPLE)));
    }

    const LARGE_EXAMPLE: &str = r"addx 15
addx -11
addx 6
addx -3
addx 5
addx -1
addx -8
addx 13
addx 4
noop
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx -35
addx 1
addx 24
addx -19
addx 1
addx 16
addx -11
noop
noop
addx 21
addx -15
noop
noop
addx -3
addx 9
addx 1
addx -3
addx 8
addx 1
addx 5
noop
noop
noop
noop
noop
addx -36
noop
addx 1
addx 7
noop
noop
noop
addx 2
addx 6
noop
noop
noop
noop
noop
addx 1
noop
noop
addx 7
addx 1
noop
addx -13
addx 13
addx 7
noop
addx 1
addx -33
noop
noop
noop
addx 2
noop
noop
noop
addx 8
noop
addx -1
addx 2
addx 1
noop
addx 17
addx -9
addx 1
addx 1
addx -3
addx 11
noop
noop
addx 1
noop
addx 1
noop
noop
addx -13
addx -19
addx 1
addx 3
addx 26
addx -30
addx 12
addx -1
addx 3
addx 1
noop
noop
noop
addx -9
addx 18
addx 1
addx 2
noop
noop
addx 9
noop
noop
noop
addx -1
addx 2
addx -37
addx 1
addx 3
noop
addx 15
addx -21
addx 22
addx -6
addx 1
noop
addx 2
addx 1
noop
addx -10
noop
noop
addx 20
addx 1
addx 2
addx 2
addx -6
addx -11
noop
noop
noop";
}
