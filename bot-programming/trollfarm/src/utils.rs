use crate::grid::Grid;
use crate::position::{CARDINALS, Position};

use std::collections::{HashSet, HashMap, VecDeque};

/// Computes a BFS distance map from a starting position, returning the shortest
/// distance and predecessor for every reachable cell.
///
/// Each entry in the returned map is `(distance, predecessor)`, where `predecessor`
/// is the previous position on the shortest path back to `from`. The source position
/// maps to `(0, from)` (it is its own predecessor).
///
/// Only cells whose grid value is one of `.`, `A`, `B`, `P`, or `L` are considered
/// passable, and any position in `blocked` is treated as impassable.
///
/// # Examples
///
/// ```
/// use std::collections::HashSet;
/// use trollfarm::{grid::*,position::*,utils::*};
///
/// let grid = Grid::from("ABPL.\n0.+.1");
/// let mut blocked = HashSet::new();
///
/// let start = grid.search(b'0').unwrap();
/// let dist_map = bfs_distance_map(start, &grid, &blocked);
///
/// let obstructed_position = grid.search(b'+').unwrap();
/// assert!(!dist_map.contains_key(&obstructed_position));
/// assert!(dist_map.values().len() == 8);
/// ```
pub fn bfs_distance_map(
    from: Position,
    grid: &Grid<u8>,
    blocked: &HashSet<Position>,
) -> HashMap<Position, (i32, Position)> {
    let mut map: HashMap<Position, (i32, Position)> = HashMap::new();
    let mut queue: VecDeque<Position> = VecDeque::new();
    map.insert(from, (0, from));
    queue.push_back(from);

    while let Some(cur) = queue.pop_front() {
        let cur_dist = map[&cur].0;

        for &c in CARDINALS.iter() {
            let next = cur + c;
            if map.contains_key(&next) || !grid.contains(next) {
                continue;
            }

            if !b".ABPL".contains(&grid[next]) {
                continue;
            }

            if blocked.contains(&next) {
                continue;
            }

            map.insert(next, (cur_dist + 1, cur));
            queue.push_back(next);
        }
    }

    map
}

/// Reconstructs the shortest path from `from` to `to` using a precomputed BFS
/// distance map (as returned by [`bfs_distance_map`]).
///
/// Returns `Some(path)` where `path` is the sequence of positions to visit
/// **after** `from` (i.e. `from` itself is not included). If `from == to`, an
/// empty vector is returned. If `to` is unreachable, returns `None`.
///
/// # Examples
///
/// ```
/// use std::collections::HashSet;
/// use trollfarm::{grid::*,position::*,utils::*};
///
/// let grid = Grid::from("ABPL.\n0.+.1");
/// let mut blocked = HashSet::new();
///
/// let start = grid.search(b'0').unwrap();
/// let dist_map = bfs_distance_map(start, &grid, &blocked);
///
/// let end = grid.search(b'B').unwrap();
/// let path = reconstruct_path(start, end, &dist_map);
/// assert!(path.unwrap().len() == 2);
///
/// let end = grid.search(b'L').unwrap();
/// let path = reconstruct_path(start, end, &dist_map);
/// assert!(path.unwrap().len() == 4);
/// ```
pub fn reconstruct_path(
    from: Position,
    to: Position,
    dist_map: &HashMap<Position, (i32, Position)>,
) -> Option<Vec<Position>> {
    if from == to {
        return Some(Vec::new());
    }

    if !dist_map.contains_key(&to) {
        return None;
    }

    let mut path = Vec::new();
    let mut cur = to;

    while cur != from {
        path.push(cur);
        cur = dist_map[&cur].1;
    }

    path.reverse();
    Some(path)
}
