//! Complexity metrics (`Analysis`) and the report formatter.
//! Spec: `../SPEC.md` §5.

use crate::grid::Grid;
use crate::solver::{self, SearchStats};

/// Complexity metrics for a grid (a puzzle, or a board in progress).
#[derive(Debug, Clone, PartialEq)]
pub struct Analysis {
    pub givens: usize,
    pub empty_cells: usize,
    pub total_candidates: usize,
    pub candidate_entropy: f64,
    pub propagated: usize,
    pub solver_nodes: u64,
    pub max_depth: u64,
    pub unique: bool,
    pub solvable: bool,
    pub difficulty: u32,
}

/// 0-100 score: 60% search effort, 40% clue scarcity.
/// `100 * (0.6 * min(1, nodes/5000) + 0.4 * (1 - givens/81))`, rounded.
pub fn difficulty_score(givens: usize, nodes: u64) -> u32 {
    let givens_part = 1.0 - (givens as f64 / 81.0);
    let nodes_part = (nodes as f64 / 5000.0).min(1.0);
    (100.0 * (0.6 * nodes_part + 0.4 * givens_part)).round() as u32
}

pub fn analyze(g: &Grid) -> Analysis {
    let givens = g.givens();
    let mut total_candidates = 0usize;
    let mut entropy = 0.0f64;
    for i in 0..81 {
        if g.is_empty_cell(i) {
            let n = g.candidates(i).len();
            total_candidates += n;
            if n > 0 {
                entropy += (n as f64).log2();
            }
        }
    }
    let mut g_copy = *g;
    let propagated = solver::propagate(&mut g_copy);
    let (solution, SearchStats { nodes, max_depth }) = solver::solve_with_stats(g);
    let unique = solver::count_solutions(g, 2) == 1;
    Analysis {
        givens,
        empty_cells: 81 - givens,
        total_candidates,
        candidate_entropy: entropy,
        propagated,
        solver_nodes: nodes,
        max_depth,
        unique,
        solvable: solution.is_some(),
        difficulty: difficulty_score(givens, nodes),
    }
}

/// The report text (same structure as the Racket CLI; booleans print as
/// `true`/`false`, entropy with 2 decimals).
pub fn format_analysis(a: &Analysis) -> String {
    format!(
        "Complexity analysis\n  givens: {}\n  empty cells: {}\n  total candidates: {}\n  candidate entropy (bits): {:.2}\n  cells filled by propagation: {}\n  solver nodes explored: {}\n  max search depth: {}\n  unique solution: {}\n  solvable: {}\n  difficulty score (0-100): {}",
        a.givens,
        a.empty_cells,
        a.total_candidates,
        a.candidate_entropy,
        a.propagated,
        a.solver_nodes,
        a.max_depth,
        a.unique,
        a.solvable,
        a.difficulty
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulty_score_bounds() {
        assert_eq!(difficulty_score(81, 0), 0);
        let s = difficulty_score(17, 1_000_000);
        assert!((90..=100).contains(&s));
        assert_eq!(s, 92);
    }

    #[test]
    fn analyze_readme_puzzle() {
        let g = Grid::from_puzzle_str(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        )
        .unwrap();
        let a = analyze(&g);
        assert_eq!(a.givens, 30);
        assert_eq!(a.empty_cells, 51);
        assert_eq!(a.total_candidates, 153);
        assert!((a.candidate_entropy - 75.17318120275553).abs() < 1e-9);
        assert_eq!(a.propagated, 51);
        assert_eq!(a.solver_nodes, 52);
        assert_eq!(a.max_depth, 51);
        assert!(a.unique);
        assert!(a.solvable);
        assert_eq!(a.difficulty, 26);
    }

    #[test]
    fn format_contains_key_lines() {
        let g = Grid::from_puzzle_str(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        )
        .unwrap();
        let report = format_analysis(&analyze(&g));
        assert!(report.contains("difficulty score"));
        assert!(report.contains("solver nodes"));
        assert!(report.contains("candidate entropy (bits): 75.17"));
    }
}
