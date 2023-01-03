use std::{cmp, fmt, iter};
use std::borrow::BorrowMut;
use std::fmt::format;
use std::iter::Cycle;
use std::path::Iter;
use std::str::Chars;
use itertools::Itertools;
use yaah::*;


#[aoc(day17, part1)]
fn solve_part1(jet_pattern: &'static str) -> usize {
    let mut chamber = Chamber::default();

    let mut shapes = vec![
        Shape::MINUS,
        Shape::PLUS,
        Shape::L,
        Shape::I,
        Shape::BOX,
    ].into_iter().cycle();
    let mut jet_cycle = jet_pattern.trim().chars().cycle();
    // let mut jet_stream = jet_cycle.borrow_mut();

    for _ in 0..2022 {
        chamber.drop_shape(jet_cycle.borrow_mut(), &shapes.next().unwrap());
        // println!("Chamber:\n{chamber}");
    }


    chamber.height()
}

type Position = (i32, i32);

#[derive(Debug, Eq, PartialEq, Default)]
struct Chamber {
    grid: Vec<Vec<char>>,
}

impl fmt::Display for Chamber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let lines = self.grid.iter()
            .map(|row| row.iter().collect::<String>())
            .map(|row| format!("|{row}|"))
            .rev()
            .chain(iter::once("+-------+".to_string()))
            .join("\n");
        write!(f, "{lines}")
    }
}

impl Chamber {
    fn height(&self) -> usize {
        self.grid.iter()
            .enumerate()
            .rev()
            .find(|(_, row)| row.contains(&'#'))
            .map(|(row, _)| row + 1)
            .unwrap_or(0)
    }

    fn extend_height(&mut self, extension: usize) {
        let height = self.height() + extension;
        // println!("Height: {height} = {} + {extension}", self.height());
        while self.grid.len() <= height {
            self.grid.push(".......".chars().collect());
        }
    }

    fn drop_shape(&mut self, jets: &mut Cycle<Chars>, shape: &Shape) {
        self.extend_height(3 + shape.height());
        let mut position = self.initial_position(shape);
        let sprite = shape.sprite();

        loop {
            // self.draw_progress(shape, position);
            position = self.jet(jets.next().unwrap(), &sprite, position);
            // self.draw_progress(shape, position);
            if self.at_rest(&sprite, position) {
                break;
            }
            position = self.fall(&sprite, position);
        }
        self.draw(shape, position);
    }

    fn initial_position(&self, shape: &Shape) -> (usize, usize) {
        match self.height() {
            0 => (shape.height() + 2, 2usize),
            _ => (self.height() + 2 + shape.height(), 2usize)
        }


    }
    fn at_rest(&self, sprite: &Vec<Vec<char>>, position: (usize, usize)) -> bool {
        let (row, column) = position;
        if row <= (sprite.len() - 1) {
            true
        } else {
            !self.available(sprite, (row - 1, column))
        }
    }

    fn available(&self, sprite: &Vec<Vec<char>>, position: (usize, usize)) -> bool {
        let (grid_row, grid_column) = position;
        grid_row >= (sprite.len() - 1) && !sprite.iter()
            .enumerate()
            .any(|(row, line)|
                line.iter()
                    .enumerate()
                    .filter(|(_, &ch)| ch == '#')
                    .map(|(column, &ch)| (ch, self.grid[grid_row - row][grid_column + column]))
                    .any(|(left, right)| left == right)
            )
    }

    fn fall(&self, sprite: &Vec<Vec<char>>, position: (usize, usize)) -> (usize, usize) {
        if position.0 == 0 {
            position
        } else {
            let new_position = (position.0 - 1, position.1);
            match self.available(sprite, new_position) {
                true => new_position,
                false => position
            }
        }
    }

    fn jet(&self, direction: char, sprite: &Vec<Vec<char>>, position: (usize, usize)) -> (usize, usize) {
        let column = match direction {
            '>' => position.1 as i32 + 1,
            '<' => position.1 as i32 - 1,
            _ => panic!()
        };

        let bounded = cmp::max(0, cmp::min(column, 7i32 - (sprite[0].len() as i32)));
        let new_position = (position.0, bounded as usize);
        if self.available(sprite, new_position) {
            new_position
        } else {
            position
        }
    }

    fn draw(&mut self, shape: &Shape, position: (usize, usize)) {
        let (row, column) = position;

        for (r, line) in shape.sprite().iter().enumerate() {
            line.iter().enumerate()
                .filter(|(_, &ch)| ch == '#')
                .for_each(|(c, ch)| self.grid[row - r][column + c] = *ch)
        }
    }

    fn draw_progress(&mut self, shape: &Shape, position: (usize, usize)) {
        let (row, column) = position;

        for (r, line) in shape.sprite().iter().enumerate() {
            line.iter().enumerate()
                .filter(|(_, &ch)| ch == '#')
                .for_each(|(c, _)| self.grid[row - r][column + c] = '@')
        }
        println!("{self}");

        for (r, line) in shape.sprite().iter().enumerate() {
            line.iter().enumerate()
                .filter(|(_, &ch)| ch == '#')
                .for_each(|(c, _)| self.grid[row - r][column + c] = '.')
        }
    }
}


#[derive(Clone)]
enum Shape {
    MINUS,
    PLUS,
    L,
    I,
    BOX,
}

impl Shape {
    fn height(&self) -> usize {
        self.sprite().len()
    }

    fn width(&self) -> usize {
        self.sprite().first().unwrap().len()
    }

    fn sprite(&self) -> Vec<Vec<char>> {
        match (self) {
            Shape::MINUS => vec!["####"],
            Shape::PLUS => vec![
                ".#.",
                "###",
                ".#.",
            ],
            Shape::L => vec![
                "..#",
                "..#",
                "###",
            ],
            Shape::I => vec![
                "#",
                "#",
                "#",
                "#",
            ],
            Shape::BOX => vec![
                "##",
                "##",
            ],
        }.into_iter()
            .map(|s| s.chars().collect())
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::day17::{Chamber, solve_part1};

    const EXAMPLE: &str = r">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>";

    #[test]
    fn part1() {
        assert_eq!(3068, solve_part1(EXAMPLE));
    }

    #[test]
    fn chamber_height() {
        let empty = Chamber { grid: vec![] };
        assert_eq!(0, empty.height());

        let small_grid: Vec<Vec<char>> = vec!["..####.".chars().collect()];
        let small = Chamber { grid: small_grid };
        assert_eq!(1, small.height());

        let grid: Vec<Vec<char>> = vec![
            "..####.",
            "...#...",
            "..###..",
            "####...",
            "..#....",
            "..#....",
        ].iter()
            .map(|s| s.chars().collect())
            .collect();

        let chamber = Chamber { grid };
        assert_eq!(6, chamber.height());
    }

    #[test]
    fn chamber_display_empty() {
        let chamber = Chamber { grid: vec![] };
        assert_eq!("+-------+", format!("{chamber}"));
    }

    #[test]
    fn chamber_display_simple() {
        let grid: Vec<Vec<char>> = vec!["..####.".chars().collect()];
        let chamber = Chamber { grid };
        let expected = r"|..####.|
+-------+";
        assert_eq!(expected, format!("{chamber}"));
    }

    #[test]
    fn chamber_display_example() {
        let grid: Vec<Vec<char>> = vec![
            "..####.",
            "...#...",
            "..###..",
            "####...",
            "..#....",
            "..#....",
        ].iter()
            .map(|s| s.chars().collect())
            .collect();

        let chamber = Chamber { grid };
        let expected = r"|..#....|
|..#....|
|####...|
|..###..|
|...#...|
|..####.|
+-------+";
        println!("{chamber}");
        assert_eq!(expected, format!("{chamber}"));
    }
}