use yaah::*;

type TreeRow = Vec<char>;
type TreeRows = Vec<TreeRow>;
type VisibilityPlot = Vec<Vec<bool>>;

#[aoc_generator(day8)]
fn gen(input: &'static str) -> Vec<Vec<char>> {
	input.lines().map(|s| s.chars().collect()).collect()
}

#[aoc(day8, part1)]
fn solve_part1(rows: &TreeRows) -> u32 {
	let columns: TreeRows = transpose(rows);
	let visibility_plot = visibility_plot(rows, &columns);

	visibility_plot
		.iter()
		.map(|row| row.iter().filter(|v| **v).count() as u32)
		.sum()
}

#[aoc(day8, part2)]
fn solve_part2(trees: &TreeRows) -> u32 {
	trees
		.iter()
		.enumerate()
		.flat_map(|(y, row)| {
			row.iter()
				.enumerate()
				.map(|(x, treehouse)| scenic_score(x, y, trees, *treehouse))
				.collect::<Vec<u32>>()
		})
		.max()
		.unwrap()
}

fn scenic_score(x: usize, y: usize, trees: &TreeRows, treehouse: char) -> u32 {
	[
		up(x, y, trees),
		down(x, y, trees),
		left(x, y, trees),
		right(x, y, trees),
	]
	.iter()
	.map(|sightline| visible_trees(treehouse, sightline))
	.product::<u32>()
}

fn up(x: usize, y: usize, trees: &TreeRows) -> TreeRow {
	(0..y).rev().map(|i| trees[i][x]).collect()
}

fn down(x: usize, y: usize, trees: &TreeRows) -> TreeRow {
	let size = trees.len();
	((y + 1)..size).map(|i| trees[i][x]).collect()
}

fn left(x: usize, y: usize, trees: &TreeRows) -> TreeRow {
	(0..x).rev().map(|i| trees[y][i]).collect()
}

fn right(x: usize, y: usize, trees: &TreeRows) -> TreeRow {
	let size = trees[0].len();
	((x + 1)..size).map(|i| trees[y][i]).collect()
}

fn visible_trees(treehouse: char, trees: &TreeRow) -> u32 {
	match trees
		.iter()
		.enumerate()
		.find_map(|(idx, tree)| match *tree >= treehouse {
			true => Some(&trees[..=idx]),
			false => None,
		}) {
		Some(tr) => tr.len() as u32,
		None => trees.len() as u32,
	}
}

fn transpose(rows: &TreeRows) -> TreeRows {
	rows[0]
		.iter()
		.enumerate()
		.map(|(i, _)| rows.iter().map(|row| row[i]).collect::<TreeRow>())
		.collect()
}

fn transpose_plot(rows: &VisibilityPlot) -> VisibilityPlot {
	rows[0]
		.iter()
		.enumerate()
		.map(|(i, _)| rows.iter().map(|row| row[i]).collect::<Vec<bool>>())
		.collect()
}

fn taller(tree: char, others: &[char]) -> bool {
	others.iter().all(|other| tree > *other)
}

fn directional_visibility(row: &TreeRow) -> Vec<bool> {
	row.iter()
		.enumerate()
		.map(|(i, tree)| (*tree, row.split_at(i)))
		.map(|(tree, (left, tail))| (left, tree, &tail[1..]))
		.map(|(left, tree, right)| taller(tree, left) || taller(tree, right))
		.collect()
}

fn visibility_plot(rows: &TreeRows, columns: &TreeRows) -> VisibilityPlot {
	rows.iter()
		.map(directional_visibility)
		.zip(transpose_plot(&columns.iter().map(directional_visibility).collect()).iter())
		.map(|(row_visibility, column_visibility)| {
			row_visibility
				.iter()
				.zip(column_visibility.iter())
				.map(|(row_value, column_value)| *row_value || *column_value)
				.collect::<Vec<bool>>()
		})
		.collect::<VisibilityPlot>()
}

#[cfg(test)]
mod test {
	use crate::day8::{
		gen, scenic_score, solve_part1, solve_part2, transpose, visibility_plot, visible_trees,
		TreeRows, VisibilityPlot,
	};

	const EXAMPLE: &str = r"30373
25512
65332
33549
35390";

	#[test]
	fn example_input() {
		let trees = gen(EXAMPLE);
		assert_eq!(5, trees.len());
		assert_eq!(5, trees[0].len());

		let columns: TreeRows = transpose(&trees);
		let visibility_plot = visibility_plot(&trees, &columns);
		print_visibility_plot(&visibility_plot);
	}

	#[test]
	fn example_part1() {
		assert_eq!(21, solve_part1(&gen(EXAMPLE)));
	}

	#[test]
	fn example_part2() {
		assert_eq!(8, solve_part2(&gen(EXAMPLE)));
	}

	/// In the example above, consider the middle 5 in the second row:
	///
	/// ```
	/// 30373
	/// 25512
	/// 65332
	/// 33549
	/// 35390
	/// ```
	/// * Looking up, its view is not blocked; it can see `1` tree (of height `3`).
	/// * Looking left, its view is blocked immediately; it can see only 1 tree (of height `5`, right next to it).
	/// * Looking right, its view is not blocked; it can see `2` trees.
	/// * Looking down, its view is blocked eventually; it can see `2` trees (one of height `3`, then the tree of height `5` that blocks its view).
	#[test]
	fn part2_example1() {
		let up = "3".chars().collect();
		let left = "52".chars().collect();
		let right = "12".chars().collect();
		let down = "353".chars().collect();

		assert_eq!(1, visible_trees('5', &up));
		assert_eq!(1, visible_trees('5', &left));
		assert_eq!(2, visible_trees('5', &right));
		assert_eq!(2, visible_trees('5', &down));

		assert_eq!(4, scenic_score(2, 1, &gen(EXAMPLE), '5'));
	}

	///However, you can do even better: consider the tree of height 5 in the middle of the fourth row:
	/// ```
	/// 30373
	/// 25512
	/// 65332
	/// 33549
	/// 35390
	/// ```
	/// Looking up, its view is blocked at `2` trees (by another tree with a height of `5`).
	/// Looking left, its view is not blocked; it can see `2` trees.
	/// Looking down, its view is also not blocked; it can see `1` tree.
	/// Looking right, its view is blocked at `2` trees (by a massive tree of height `9`).
	///
	/// This tree's scenic score is `8` `(2 * 2 * 1 * 2)`; this is the ideal spot for the tree house.
	///
	#[test]
	fn part2_example2() {
		let up = "353".chars().collect();
		let left = "33".chars().collect();
		let right = "49".chars().collect();
		let down = "3".chars().collect();

		assert_eq!(2, visible_trees('5', &up));
		assert_eq!(2, visible_trees('5', &left));
		assert_eq!(1, visible_trees('5', &down));
		assert_eq!(2, visible_trees('5', &right));

		assert_eq!(8, scenic_score(2, 3, &gen(EXAMPLE), '5'));

		assert_eq!(8, solve_part2(&gen(EXAMPLE)));
	}

	fn print_visibility_plot(map: &VisibilityPlot) {
		map.iter()
			.map(|column| {
				column
					.iter()
					.map(|v| match *v {
						true => '+',
						false => ' ',
					})
					.collect::<String>()
			})
			.for_each(|s| println!("{:?}", s))
	}
}
