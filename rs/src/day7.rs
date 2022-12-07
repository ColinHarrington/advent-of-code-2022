use std::borrow::Cow;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{alphanumeric1, line_ending, multispace1, newline, not_line_ending};
use nom::IResult;
use nom::multi::separated_list1;
use nom::sequence::{preceded, tuple};
use yaah::*;

// #[aoc_generator(day7)]
// fn gen(input: &'static str) -> Vec<Command<'static>> {
//     let (tail, commands) = parse_terminal(input).unwrap();
//     commands
// }
#[aoc(day7, part1)]
fn solve_part1(input: &'static str) -> u32 {
    let (tail, commands) = parse_terminal(input).unwrap();
    dbg!(&commands);
    0
}

#[derive(Debug, PartialEq)]
enum Command<'a> {
    ChangeDirectory { target: Cow<'a, str> },
    List { entries: Vec<ListEntry<'a>> },
}

#[derive(Debug, PartialEq)]
enum ListEntry<'b> {
    File { size: u32, name: Cow<'b, str> },
    Dir { name: Cow<'b, str> },
}

fn parse_terminal(input: &str) -> IResult<&str, Vec<Command>> {
    Ok(separated_list1(line_ending, command)(input)?)
}

fn command(input: &str) -> IResult<&str, Command> {
    Ok(preceded(tag("$ "), alt((cd_command, ls_command)))(input)?)
}

fn cd_command(input: &str) -> IResult<&str, Command> {
    // let (input, name) = tag("cd")(input)?;
    let (input, target) = preceded(
        tuple((tag("cd"), multispace1)),
        alt((tag("/"), tag(".."), alphanumeric1)),
    )(input)?;

    Ok((input, Command::ChangeDirectory { target: Cow::Borrowed(target) }))
}

fn ls_command(input: &str) -> IResult<&str, Command> {
    // let (input, _) = tag("$ ")(input)?;
    let (input, name) = tag("ls")(input)?;
    let (input, _) = newline(input)?;
    let (input, entries) = separated_list1(line_ending, alt((dir_entry, file_entry)))(input)?;

    Ok((input, Command::List { entries }))
}

fn dir_entry(input: &str) -> IResult<&str, ListEntry> {
    let (input, _) = tag("dir ")(input)?;
    let (input, name) = alphanumeric1(input)?;

    Ok((input, ListEntry::Dir { name: Cow::Borrowed(name) }))
}

fn file_entry(input: &str) -> IResult<&str, ListEntry> {
    let (input, size) = complete::u32(input)?;
    let (input, name) = preceded(multispace1, not_line_ending)(input)?;

    Ok((input, ListEntry::File { size, name: Cow::Borrowed(name) }))
}


#[cfg(test)]
mod test {
    use std::borrow::Cow;
    use crate::day7::{Command, command, ListEntry, parse_terminal, solve_part1};
    use crate::day7::Command::ChangeDirectory;

    const EXAMPLE: &str = r"$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k";

    #[test]
    fn example() {
        let (tail, commands) = parse_terminal(EXAMPLE).unwrap();
        dbg!(tail, &commands);
        assert_eq!(10, commands.len());
    }

    #[test]
    fn test_cd() {
        let expected = vec![
            ("$ cd /", "/"),
            ("$ cd ..", ".."),
            ("$ cd x", "x"),
        ];
        for (cmd, target) in expected {
            let (s, cmd) = command(cmd).unwrap();
            assert!(s.is_empty());
            assert_eq!(cmd, ChangeDirectory { target: Cow::Borrowed(target) });
        }
    }

    #[test]
    fn test_ls() {
        let cmd = r"$ ls
dir e
29116 f
2557 g
62596 h.lst";
        let entries: Vec<ListEntry> = vec![
            ListEntry::Dir { name: Cow::Borrowed("e") },
            ListEntry::File { size: 29116, name: Cow::Borrowed("f") },
            ListEntry::File { size: 2557, name: Cow::Borrowed("g") },
            ListEntry::File { size: 62596, name: Cow::Borrowed("h.lst") },
        ];

        let (s, cmd) = command(cmd).unwrap();
        assert!(s.is_empty());
        assert_eq!(cmd, Command::List { entries });
    }

    #[test]
    fn part1() {
        let a = solve_part1(&EXAMPLE);
        assert_eq!(99, a)
    }

}