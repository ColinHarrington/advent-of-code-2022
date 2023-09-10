use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete;
use nom::character::complete::{alphanumeric1, line_ending, multispace1, newline, not_line_ending};
use nom::multi::separated_list1;
use nom::sequence::{preceded, tuple};
use nom::IResult;
use std::borrow::Cow;
use std::collections::HashMap;
use yaah::*;

type FileSystem = HashMap<String, u32>;

#[aoc(day7, part1)]
fn solve_part1(input: &'static str) -> u32 {
	let (_, commands) = parse_terminal(input).unwrap();

	let fs: FileSystem = build_fs(commands);
	let predicate = |size: &u32| *size <= 100000;
	let undersize: Vec<u32> = filter_dir_size(fs, &predicate);

	undersize.into_iter().sum()
}

#[aoc(day7, part2)]
fn solve_part2(input: &'static str) -> u32 {
	let (_, commands) = parse_terminal(input).unwrap();
	let fs: FileSystem = build_fs(commands);

	let total_disk_space: u32 = 70000000;
	let required_free_space: u32 = 30000000;
	let total_used: u32 = fs.keys().filter_map(|k| fs.get(k)).sum();
	let unused_space = total_disk_space - total_used;
	let min_deleted = required_free_space - unused_space;

	let predicate = |size: &u32| *size >= min_deleted;
	let undersize: Vec<u32> = filter_dir_size(fs, &predicate);

	undersize.into_iter().min().unwrap()
}

fn build_fs(commands: Vec<Command>) -> FileSystem {
	let mut file_system: FileSystem = FileSystem::new();
	let mut cwd = vec!["/".to_string()];
	for command in commands {
		match command {
			Command::ChangeDirectory { target } => match String::from(target).as_str() {
				"/" => cwd.clear(),
				".." => {
					cwd.pop();
				}
				name => cwd.push(name.to_string()),
			},
			Command::List { entries } => {
				let path = build_path(&cwd);
				let size: u32 = entries
					.into_iter()
					.map(|e| match e {
						ListEntry::Dir { name: _ } => 0,
						ListEntry::File { size, name: _ } => size,
					})
					.sum();
				file_system.insert(path, size);
			}
		}
	}

	file_system
}

fn filter_dir_size(fs: FileSystem, size_predicate: &dyn Fn(&u32) -> bool) -> Vec<u32> {
	fs.keys()
		.filter_map(|key| {
			fs.keys()
				.filter(|k2| k2.starts_with(key))
				.map(|k| fs.get(k))
				.sum()
		})
		.filter(size_predicate)
		.collect()
}

fn build_path(parts: &Vec<String>) -> String {
	match parts.len() {
		0 => "/".to_string(),
		_ => format!("/{}/", parts.join("/")),
	}
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
	separated_list1(line_ending, command)(input)
}

fn command(input: &str) -> IResult<&str, Command> {
	preceded(tag("$ "), alt((cd_command, ls_command)))(input)
}

fn cd_command(input: &str) -> IResult<&str, Command> {
	// let (input, name) = tag("cd")(input)?;
	let (input, target) = preceded(
		tuple((tag("cd"), multispace1)),
		alt((tag("/"), tag(".."), alphanumeric1)),
	)(input)?;

	Ok((
		input,
		Command::ChangeDirectory {
			target: Cow::Borrowed(target),
		},
	))
}

fn ls_command(input: &str) -> IResult<&str, Command> {
	let (input, _) = tag("ls")(input)?;
	let (input, _) = newline(input)?;
	let (input, entries) = separated_list1(line_ending, alt((dir_entry, file_entry)))(input)?;

	Ok((input, Command::List { entries }))
}

fn dir_entry(input: &str) -> IResult<&str, ListEntry> {
	let (input, _) = tag("dir ")(input)?;
	let (input, name) = alphanumeric1(input)?;

	Ok((
		input,
		ListEntry::Dir {
			name: Cow::Borrowed(name),
		},
	))
}

fn file_entry(input: &str) -> IResult<&str, ListEntry> {
	let (input, size) = complete::u32(input)?;
	let (input, name) = preceded(multispace1, not_line_ending)(input)?;

	Ok((
		input,
		ListEntry::File {
			size,
			name: Cow::Borrowed(name),
		},
	))
}

#[cfg(test)]
mod test {
	use crate::day7::Command::ChangeDirectory;
	use crate::day7::{command, parse_terminal, solve_part1, solve_part2, Command, ListEntry};
	use std::borrow::Cow;

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
		assert!(tail.is_empty());
		assert_eq!(10, commands.len());
	}

	#[test]
	fn test_cd() {
		let expected = vec![("$ cd /", "/"), ("$ cd ..", ".."), ("$ cd x", "x")];
		for (cmd, target) in expected {
			let (s, cmd) = command(cmd).unwrap();
			assert!(s.is_empty());
			assert_eq!(
				cmd,
				ChangeDirectory {
					target: Cow::Borrowed(target)
				}
			);
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
			ListEntry::Dir {
				name: Cow::Borrowed("e"),
			},
			ListEntry::File {
				size: 29116,
				name: Cow::Borrowed("f"),
			},
			ListEntry::File {
				size: 2557,
				name: Cow::Borrowed("g"),
			},
			ListEntry::File {
				size: 62596,
				name: Cow::Borrowed("h.lst"),
			},
		];

		let (s, cmd) = command(cmd).unwrap();
		assert!(s.is_empty());
		assert_eq!(cmd, Command::List { entries });
	}

	/// To begin, find all of the directories with a total size of at most `100000`,
	/// then calculate the sum of their total sizes. In the example above,
	/// these directories are `a` and `e`; the sum of their total sizes is `95437` (94853 + 584).
	/// (As in this example, this process can count files more than once!)
	#[test]
	fn part1() {
		assert_eq!(95437, solve_part1(EXAMPLE))
	}

	#[test]
	fn part2() {
		assert_eq!(24933642, solve_part2(EXAMPLE))
	}
}
