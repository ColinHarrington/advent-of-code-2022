use std::collections::HashMap;
use std::fmt;
use itertools::Itertools;

use pathfinding::matrix::Matrix;
use pathfinding::prelude::{astar, dfs, dijkstra};
// use petgraph::algo::{astar, dijkstra};
// use petgraph::Directed;
// use petgraph::dot::{Config, Dot};
// use petgraph::graphmap::DiGraphMap;
// use petgraph::prelude::GraphMap;
use yaah::*;

/*
  Parse the grid
  * Find the starting point
  * Find the ending point
  * Constuct a chain.
  * identify nodes that only have one forward path

  If a node has no options, it's stranded
  If a node has only one option, then it's fixed.
  If a node has

  Map paths
  Point => vec[Point]
  */

// #[aoc_generator(day12)]
// fn gen(input: &'static str) -> Heightmap {
//     input.lines()
//         .enumerate()
//         .map(|(y, line)| line.chars().enumerate()
//             .map(|(x, height)| Elevation(x, y, height))
//             .collect()
//         )
//         .collect()
// }

#[aoc_generator(day12)]
fn gen(input: &'static str) -> Matrix<Elevation> {
    Matrix::from_rows(
        input.lines()
            .enumerate()
            .map(|(row, line)| line.chars()
                .enumerate()
                .map(|(column, height)| Elevation { row, column, height })
                .collect()
            )
            .collect::<Vec<Vec<Elevation>>>()
    ).unwrap()
}

#[aoc(day12, part1)]
fn solve_part1(map: &Matrix<Elevation>) -> u32 {
    let start = map.items()
        .map(|(_, &elevation)| elevation)
        .find(|&elevation| elevation.height == 'S').unwrap();
    let end = map.items()
        .map(|(_, &elevation)| elevation)
        .find(|&elevation| elevation.height == 'E').unwrap();

    0
}

fn successors(map: &Matrix<Elevation>, elevation: Elevation) -> Vec<Elevation> {
    let horses = map.neighbours((elevation.row, elevation.column), false)
        .map(|rc| *map.get(rc).unwrap())
        .filter(|neighbor| is_passable(&elevation, neighbor))
        .collect::<Vec<Elevation>>();

    // let display = horses.iter().map(|elevation)|format!("{elevation}")).join(", ");
    // println!("{elevation} => {display}");
    // dbg!(&elevation, &horses);
    match horses.iter().find(|&e|e.height == 'E') {
        Some(&end) => vec![end],
        None => horses
    }
}
//
// fn passable_distance(e1: Elevation, e2: Elevation) -> Option<usize> {
//     match (e1.height, e2.height) {
//         ('E', _) => None,
//         ('S', 'a') => Some(1),
//         ('S', _) => None,
//         ('z', 'E') => Some(1),
//         (a, b) => match (b as i32) - (a as i32) {
//             0 => Some(0),
//             1 => Some(1),
//             _ => None
//         }
//     }
// }

// #[aoc(day12, part1)]
// fn solve_part1(map: &Matrix<Elevation>) -> u32 {
//     // let m:DiGraphMap<Elevation, u32> = DiGraphMap::from_elements(map.items());
//     let edges: Vec<(Elevation, Elevation)> = map.items()
//         .map(|((row, column), &elevation)| map
//             .neighbours((row, column), false)
//             .inspect(|n| println!("[{:?}] -> {:?}", elevation, n))
//             .map(|rc| *map.get(rc).unwrap())
//             .filter(|&neighbor|is_passable(&elevation, &neighbor))
//             .map(|neighbor|(elevation, neighbor))
//             .inspect(|&p| println!("Edge:{:?}", p))
//             .collect::<Vec<(Elevation, Elevation)>>()
//         )
//         .flatten()
//         .collect();
//     // let x = map.items().map(|(r,c),c| c).collect();
//     // dbg!(&edges);
//     let mut graph: DiGraphMap<Elevation, ()> = DiGraphMap::from_edges(edges);
//
//
//     // for (rc, &elevation) in map.items() {
//     //     match graph.contains_node(elevation) {
//     //         true => (),
//     //         false => graph.add_node(elevation)
//     //     };
//     // }
//
//     dbg!(graph.nodes().len(), map.items().count());
//     let start = map.items()
//         .map(|(_, &elevation)| elevation)
//         .find(|&elevation| elevation.height == 'S').unwrap();
//     let end = map.items()
//         .map(|(_, &elevation)| elevation)
//         .find(|&elevation| elevation.height == 'E').unwrap();
//     dbg!(graph.nodes().len(), map.items().count(), &start, &end);
//     // let res = dijkstra(&graph, start, Some(end), |_| 1u32);
//     let res = astar(&graph, start, |n| n.height == 'E', |(e1, e2, _)| 1,|e| 0u32);
//     dbg!(&res);
//
//     // *res.get(&end).unwrap() as u32
//     0
// }

//
// #[aoc(day12, part1)]
// fn solve_part1(map: &Heightmap) -> u32 {
//     let mut graph:DiGraphMap<Elevation, ()> = DiGraphMap::new();
//
//     let paths = build_paths(map);
//     for row in map {
//         for &elevation in row {
//             // let node_id = graph.add_node(elevation.clone());
//             let Elevation(x, y, height) = elevation;
//             paths.get(&(elevation.0, elevation.1)).unwrap().iter()
//                 .for_each(|&point| {
//                     let e2 = map[point.1][point.0];
//                     graph.add_edge(elevation, e2, ());
//                 })
//         }
//     }
//
//     println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
//
//     let start = *map.iter().flatten().find(|&e|e.2 == 'S').unwrap();
//     let end = *map.iter().flatten().find(|&e|e.2 == 'E').unwrap();
//     dbg!(&start, &end);
//     // let end: Option<Elevation>= map.iter()
//     //     .flatten()
//     //     .find_map(|&e| match e.2 {
//     //         'E' => Some(e),
//     //         _ => None
//     //     });
//
//     let res = dijkstra(&graph, start,Some(end), |_| 1);
//     // let (steps, nodes) = dijkstra(&graph, *start,
//     //                            end,
//     //                            |_| 1);
//     dbg!(&res, &res.len(), res.get(&end));
//     *res.get(&end).unwrap() as u32
// }


// fn at_end(elevation: &Elevation) -> bool {
//     return elevation.2 == 'E';
// }

// pub struct ElevationMap {
//     map: Matrix<Elevation>,
// }
//
// impl ElevationMap {
//     pub fn successors(&self, point: &Point) -> Vec<(Point, usize)> {
//         let elevation = self.map.get(*point).unwrap();
//
//         // let ok = self.map.neighbours(*point, false).for_each(|(r,c)| println!("{r},{c}"));
//         // dbg!(&ok);
//         let neighbors = self.map.neighbours(*point, false)
//             // .inspect(|(r, c)| println!("{r},{c}"))
//             .filter_map(|i| self.map.get(i))
//             // .inspect(|el| println!("{:?} @ {:?}", el.height, el.point))
//             .filter(|other| is_passable(elevation, other))
//             .map(|e| (e.point, 1 as usize))
//             // .inspect(|s| println!("{:?} successor {:?}", point, s))
//             .collect::<Vec<(Point, usize)>>();
//         println!("{:?} => {:?} => {:?}", point, elevation.height, neighbors);
//         neighbors
//     }
//
//
//     pub fn start(&self) -> &Elevation {
//         self.map.values().find(|e| e.height == 'S').unwrap()
//     }
//
//     pub fn end(&self) -> &Elevation {
//         self.map.values().find(|e| e.height == 'E').unwrap()
//     }
//
//     pub fn is_end(&self, point: Point) -> bool {
//         match self.map.get(point) {
//             Some(elevation) => elevation.height == 'E',
//             None => false
//         }
//     }
// }

// S => only 'a'
// Same
// z => 'E'
fn char_cmp(h1: char, h2: char) -> bool {
    println!("{:?} <=> {:?}", h1, h2);
    match h1 {
        'S' => h2 == 'a',
        'z' => h2 == 'z' || h2 == 'E',
        _ => match (h2 as i32) - (h1 as i32) {
            0 | 1 => true,
            _ => false
        }
    }
}

/// To avoid needing to get out your climbing gear,
/// the elevation of the destination square can be at most one higher than the elevation of your current square;
/// that is, if your current elevation is m, you could step to elevation n, but not to elevation o.
/// (This also means that the elevation of the destination square can be much lower than the elevation of your current square.)
fn is_passable(e1: &Elevation, e2: &Elevation) -> bool {
    match (e1.height, e2.height) {
        ('E', _) => false,
        ('S', 'a') => true,
        ('S', _) => false,
        (_, 'S') => false,
        (c, 'E') => c == 'z',
        (a,b) if a == b => true,
        (a,b) if a > b => false,
        (a,b) => (b as usize) - (a as usize) == 1,
    }
}

type Point = (usize, usize);
type Heightmap = Vec<Vec<Elevation>>;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Ord, PartialOrd)]
pub struct Elevation {
    row: usize,
    column: usize,
    height: char,
}

impl fmt::Display for Elevation {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{}({},{})", self.height, self.column, self.row)
    }
}


// fn build_paths(map: &Heightmap) -> HashMap<Point, Vec<Point>> {
//     map.into_iter()
//         .map(|row| row.into_iter()
//             .map(|e| ((e.0, e.1), paths_forward(e, map)))
//             .collect::<Vec<(Point, Vec<Point>)>>()
//         )
//         .flatten()
//         .collect::<HashMap<Point, Vec<Point>>>()
// }
//
// fn paths_forward(elevation: &Elevation, map: &Heightmap) -> Vec<Point> {
//     let Elevation(x, y, _) = *elevation;
//     let xrange = 0..(map[0].len());
//     let yrange = 0..(map.len());
//     // Up, Down, Left, Right
//     let neighbors: Vec<(i32, i32)> = vec![
//         (-1, 0),
//         (1, 0),
//         (0, 1),
//         (0, -1),
//     ];
//     neighbors.iter()
//         .map(|(horizontal, vertical)| (horizontal + x as i32, vertical + y as i32))
//         .filter_map(|(x, y)| match yrange.contains(&(y as usize)) && xrange.contains(&(x as usize)) {
//             true => Some((x as usize, y as usize)),
//             false => None
//         })
//         .map(|(x1, y1)| map[y1][x1].clone())
//         .filter(|other| is_passable(elevation,other))
//         .map(|e| (e.0, e.1))
//         .collect::<Vec<Point>>()
// }
//
// fn locate_start(map: &Heightmap) -> Point {
//     map.into_iter()
//         .enumerate()
//         .find_map(|(y, line)| line.into_iter()
//             .enumerate()
//             .find_map(|(x, elevation)| match elevation.height {
//                 'S' => Some((x, y)),
//                 _ => None
//             })
//         ).unwrap()
// }
//
// fn locate_end(map: &Heightmap) -> Point {
//     map.into_iter()
//         .enumerate()
//         .find_map(|(y, line)| line.into_iter()
//             .enumerate()
//             .find_map(|(x, elevation)| match elevation.height {
//                 'E' => Some((x, y)),
//                 _ => None
//             })
//         ).unwrap()
// }

#[cfg(test)]
mod test {
    use crate::day12::{char_cmp, gen, solve_part1};

    const EXAMPLE: &str = r"Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi";

    #[test]
    fn test_input() {
        let map = gen(EXAMPLE);
    }

    #[test]
    fn part1() {
        assert_eq!(31, solve_part1(&gen(EXAMPLE)));
    }

    #[test]
    fn test_compare() {
        assert_eq!(true, char_cmp('S', 'a'));
        assert_eq!(true, char_cmp('a', 'a'));
        assert_eq!(true, char_cmp('a', 'b'));
        assert_eq!(false, char_cmp('a', 'c'));

        assert_eq!(true, char_cmp('z', 'z'));
        assert_eq!(true, char_cmp('z', 'E'));
    }
}

