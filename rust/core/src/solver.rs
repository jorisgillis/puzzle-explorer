//! MRV backtracking solver with constraint propagation (naked + hidden
//! singles), search stats, and capped solution counting.
//! Spec: `../SPEC.md` §3.

use crate::grid::{Grid, UNITS};

/// Fill every forced cell until a fixpoint, mutating `g`; returns how many
/// cells were filled. Naked singles first, then hidden singles, per round.
pub fn propagate(g: &mut Grid) -> usize {
    let mut filled = 0;
    loop {
        let mut progressed = 0;
        for i in 0..81 {
            if g.is_empty_cell(i) {
                let cs = g.candidates(i);
                if cs.len() == 1 {
                    g.set(i, cs[0]);
                    progressed += 1;
                }
            }
        }
        for unit in UNITS.iter() {
            for v in 1..=9u8 {
                let spots: Vec<usize> = unit
                    .iter()
                    .copied()
                    .filter(|&i| g.is_empty_cell(i) && g.candidates(i).contains(&v))
                    .collect();
                if spots.len() == 1 {
                    g.set(spots[0], v);
                    progressed += 1;
                }
            }
        }
        if progressed == 0 {
            return filled;
        }
        filled += progressed;
    }
}

/// MRV: the empty cell with the fewest candidates (lowest index on ties).
/// `None` when some empty cell has no candidates left (dead end).
pub fn pick_cell(g: &Grid) -> Option<(usize, Vec<u8>)> {
    let mut best: Option<(usize, Vec<u8>)> = None;
    for i in 0..81 {
        if g.is_empty_cell(i) {
            let cs = g.candidates(i);
            if cs.is_empty() {
                return None;
            }
            if best.as_ref().is_none_or(|(_, bcs)| cs.len() < bcs.len()) {
                best = Some((i, cs));
            }
        }
    }
    best
}

/// Search effort: nodes visited and deepest recursion level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchStats {
    pub nodes: u64,
    pub max_depth: u64,
}

/// Solve with MRV backtracking on a copy of `g` (input not mutated).
/// Invalid input never enters the search: stats stay at zero.
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

fn search(g: &mut Grid, depth: u64, stats: &mut SearchStats) -> Option<Grid> {
    stats.nodes += 1;
    stats.max_depth = stats.max_depth.max(depth);
    if g.is_complete() {
        return Some(*g);
    }
    let (i, cs) = pick_cell(g)?;
    for v in cs {
        g.set(i, v);
        if let Some(result) = search(g, depth + 1, stats) {
            return Some(result);
        }
        g.set(i, 0); // backtrack before trying the next candidate
    }
    None
}

/// Solve, discarding the search stats.
pub fn solve(g: &Grid) -> Option<Grid> {
    solve_with_stats(g).0
}

/// Count solutions, stopping early once `count` reaches `cap`.
/// Invalid grids have 0 solutions. A dead-end branch only ends that branch
/// (the intended semantics — Racket's `for/and` short-circuit quirk is
/// deliberately NOT replicated; see `tests/fixtures/parity.json`
/// `known_quirks`).
pub fn count_solutions(g: &Grid, cap: u64) -> u64 {
    if !g.is_valid() {
        return 0;
    }
    let mut count = 0u64;
    let mut work = *g;
    count_search(&mut work, cap, &mut count);
    count
}

/// Returns true when the cap has been reached and the search should stop.
fn count_search(g: &mut Grid, cap: u64, count: &mut u64) -> bool {
    if *count >= cap {
        return true;
    }
    if g.is_complete() {
        *count += 1;
        return *count >= cap;
    }
    let (i, cs) = match pick_cell(g) {
        Some(x) => x,
        None => return false,
    };
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

#[cfg(test)]
mod tests {
    use super::*;

    const README: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

    #[test]
    fn candidates_of_cell_2() {
        let g = Grid::from_puzzle_str(README).unwrap();
        assert_eq!(g.candidates(2), vec![1, 2, 4]);
    }

    #[test]
    fn propagate_fills_readme_completely() {
        let g = Grid::from_puzzle_str(README).unwrap();
        let mut gc = g;
        assert_eq!(propagate(&mut gc), 51);
        assert!(gc.is_complete());
    }

    #[test]
    fn count_solutions_caps() {
        let g = Grid::from_puzzle_str(README).unwrap();
        assert_eq!(count_solutions(&g, 2), 1);
        assert_eq!(count_solutions(&g, 3), 1);
        let empty = Grid::empty();
        assert_eq!(count_solutions(&empty, 2), 2);
        assert_eq!(count_solutions(&empty, 3), 3);
    }

    #[test]
    fn invalid_grid_never_searches() {
        let mut g = Grid::empty();
        g.set(0, 1);
        g.set(1, 1); // duplicate in row 0
        let (sol, stats) = solve_with_stats(&g);
        assert!(sol.is_none());
        assert_eq!(stats.nodes, 0);
        assert_eq!(stats.max_depth, 0);
        assert_eq!(count_solutions(&g, 2), 0);
    }
}
