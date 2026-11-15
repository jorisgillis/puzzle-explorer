//! Naive backtracking solver: scan cells 0-80 in order, try digits 1-9 in
//! order using grid candidates. No constraint propagation, no MRV.
//! Invalid input never enters the search: `(None, zero stats)` / 0.

use crate::grid::{CELL_COUNT, Grid};
use crate::solver::SearchStats;

use super::Solver;

pub struct Naive;

fn search(g: &mut Grid, depth: u64, stats: &mut SearchStats) -> Option<Grid> {
    stats.nodes += 1;
    stats.max_depth = stats.max_depth.max(depth);
    if g.is_complete() {
        return Some(*g);
    }
    let i = (0..CELL_COUNT).find(|&i| g.is_empty_cell(i)).unwrap();
    for v in g.candidates(i) {
        g.set(i, v);
        if let Some(result) = search(g, depth + 1, stats) {
            return Some(result);
        }
        g.set(i, 0); // backtrack before trying the next digit
    }
    None
}

pub fn solve_with_stats(g: &Grid) -> (Option<Grid>, SearchStats) {
    let mut stats = SearchStats {
        nodes: 0,
        max_depth: 0,
    };
    let result = if g.is_valid() {
        let mut work = *g;
        search(&mut work, 0, &mut stats)
    } else {
        None
    };
    (result, stats)
}

/// Returns true when the whole search should stop (cap reached).
fn count_search(g: &mut Grid, cap: u64, count: &mut u64) -> bool {
    if *count >= cap {
        return true;
    }
    if g.is_complete() {
        *count += 1;
        return *count >= cap;
    }
    let i = (0..CELL_COUNT).find(|&i| g.is_empty_cell(i)).unwrap();
    let cs = g.candidates(i);
    if cs.is_empty() {
        return false; // dead end ends only this branch
    }
    for v in cs {
        g.set(i, v);
        let stop = count_search(g, cap, count);
        g.set(i, 0);
        if stop {
            return true;
        }
    }
    false
}

pub fn count_solutions(g: &Grid, cap: u64) -> u64 {
    if !g.is_valid() {
        return 0;
    }
    let mut count = 0u64;
    let mut work = *g;
    count_search(&mut work, cap, &mut count);
    count
}

impl Solver for Naive {
    fn name(&self) -> &'static str {
        "naive"
    }
    fn solve(&self, g: &Grid) -> Option<Grid> {
        solve_with_stats(g).0
    }
    fn solve_with_stats(&self, g: &Grid) -> (Option<Grid>, SearchStats) {
        solve_with_stats(g)
    }
    fn count_solutions(&self, g: &Grid, cap: u64) -> u64 {
        count_solutions(g, cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const README: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    const SOLVED: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    #[test]
    fn solves_readme_puzzle() {
        let g = Grid::from_puzzle_str(README).unwrap();
        let (sol, stats) = solve_with_stats(&g);
        assert_eq!(sol.unwrap().to_puzzle_string(), SOLVED);
        assert!(stats.nodes > 0 && stats.max_depth > 0);
    }

    #[test]
    fn count_respects_cap_and_invalid() {
        let g = Grid::from_puzzle_str(README).unwrap();
        assert_eq!(count_solutions(&g, 2), 1);
        assert_eq!(count_solutions(&Grid::empty(), 2), 2);
        assert_eq!(count_solutions(&Grid::empty(), 3), 3);
        let mut bad = Grid::empty();
        bad.set(0, 1);
        bad.set(1, 1);
        assert_eq!(count_solutions(&bad, 2), 0);
        let (sol, stats) = solve_with_stats(&bad);
        assert!(sol.is_none());
        assert_eq!(stats.nodes, 0);
    }
}
