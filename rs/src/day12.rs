use pathfinding::matrix::Matrix;
use petgraph::algo::astar;
use petgraph::prelude::DiGraphMap;
use std::fmt;
use yaah::*;

#[aoc_generator(day12)]
fn gen(input: &'static str) -> Matrix<Elevation> {
	Matrix::from_rows(
		input
			.lines()
			.enumerate()
			.map(|(row, line)| {
				line.chars()
					.enumerate()
					.map(|(column, height)| Elevation {
						row,
						column,
						height,
					})
					.collect()
			})
			.collect::<Vec<Vec<Elevation>>>(),
	)
	.unwrap()
}

#[aoc(day12, part1)]
fn solve_part1(map: &Matrix<Elevation>) -> u32 {
	hike(map, 'S', 'E')
}

#[aoc(day12, part2)]
fn solve_part2(map: &Matrix<Elevation>) -> u32 {
	hike(map, 'E', 'a')
}

fn hike(map: &Matrix<Elevation>, from: char, to: char) -> u32 {
	let mut graph: DiGraphMap<Elevation, u32> = DiGraphMap::new();

	let edges = map
		.items()
		.map(|(_, &elevation)| {
			successors(map, elevation.clone())
				.iter()
				.map(|&other| (elevation, other, 1u32))
				.collect::<Vec<(Elevation, Elevation, u32)>>()
		})
		.flatten()
		.collect::<Vec<(Elevation, Elevation, u32)>>();

	for (left, right, weight) in edges {
		match from {
			'S' => graph.add_edge(left, right, weight),
			_ => graph.add_edge(right, left, weight),
		};
	}

	let start = graph.nodes().find(|n| n.height == from).unwrap();

	let (steps, _) = astar(&graph, start, |e| e.height == to, |_| 1, |_| 0).unwrap();

	steps as u32
}

fn successors(map: &Matrix<Elevation>, elevation: Elevation) -> Vec<Elevation> {
	let horses = map
		.neighbours((elevation.row, elevation.column), false)
		.map(|rc| *map.get(rc).unwrap())
		.filter(|neighbor| is_passable(&elevation, neighbor))
		.collect::<Vec<Elevation>>();
	match horses.iter().find(|&e| e.height == 'E') {
		Some(&end) => vec![end],
		None => horses,
	}
}

/// To avoid needing to get out your climbing gear,
/// the elevation of the destination square can be at most one higher than the elevation of your current square;
/// that is, if your current elevation is m, you could step to elevation n, but not to elevation o.
/// (This also means that the elevation of the destination square can be much lower than the elevation of your current square.)
fn is_passable(e1: &Elevation, e2: &Elevation) -> bool {
	e2.height_value() <= e1.height_value() + 1
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Ord, PartialOrd)]
pub struct Elevation {
	row: usize,
	column: usize,
	height: char,
}

impl fmt::Display for Elevation {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}({},{})", self.height, self.row + 1, self.column + 1)
	}
}

impl Elevation {
	fn height_value(&self) -> u32 {
		match self.height {
			'E' => 'z' as u32,
			'S' => 'a' as u32,
			c => c as u32,
		}
	}
}

#[cfg(test)]
mod test {
	use crate::day12::{gen, solve_part1, solve_part2};

	const EXAMPLE: &str = r"Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi";

	#[test]
	fn test_input() {
		let map = gen(EXAMPLE);
		assert_eq!(8 * 5, map.len())
	}

	#[test]
	fn part1() {
		assert_eq!(31, solve_part1(&gen(EXAMPLE)));
	}

	#[test]
	fn part2() {
		assert_eq!(29, solve_part2(&gen(EXAMPLE)));
	}
}
