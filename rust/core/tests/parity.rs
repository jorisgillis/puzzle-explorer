//! Parity tests against the golden fixtures measured from the Racket
//! implementation (ticket #11 findings, `fixtures/parity.json`).

use serde::Deserialize;
use sudoku_core::{Grid, analyze, count_solutions, propagate, solve_with_stats};

#[derive(Deserialize)]
struct Fixtures {
    puzzles: Vec<PuzzleFixture>,
}

#[derive(Deserialize)]
struct PuzzleFixture {
    id: String,
    input: String,
    solve: SolveFixture,
    count_solutions_cap2: u64,
    count_solutions_cap3: u64,
    propagate_filled: usize,
    analysis: AnalysisFixture,
}

#[derive(Deserialize)]
struct SolveFixture {
    solution: Option<String>,
    nodes: u64,
    max_depth: u64,
}

#[derive(Deserialize)]
struct AnalysisFixture {
    givens: usize,
    empty_cells: usize,
    total_candidates: usize,
    candidate_entropy_bits: f64,
    propagated: usize,
    solver_nodes: u64,
    max_depth: u64,
    unique: bool,
    solvable: bool,
    difficulty: u32,
}

#[test]
fn parity_with_racket_fixtures() {
    let json = include_str!("fixtures/parity.json");
    let fx: Fixtures = serde_json::from_str(json).unwrap();
    assert!(fx.puzzles.len() >= 7);

    for p in &fx.puzzles {
        let g = Grid::from_puzzle_str(&p.input).unwrap_or_else(|e| panic!("{}: {e}", p.id));

        let (sol, stats) = solve_with_stats(&g);
        match &p.solve.solution {
            Some(expected) => {
                assert_eq!(
                    sol.as_ref().map(Grid::to_puzzle_string).as_deref(),
                    Some(expected.as_str()),
                    "{}: solution mismatch",
                    p.id
                );
            }
            None => assert!(sol.is_none(), "{}: expected no solution", p.id),
        }
        assert_eq!(stats.nodes, p.solve.nodes, "{}: nodes", p.id);
        assert_eq!(stats.max_depth, p.solve.max_depth, "{}: max_depth", p.id);

        assert_eq!(
            count_solutions(&g, 2),
            p.count_solutions_cap2,
            "{}: count_solutions cap 2",
            p.id
        );
        assert_eq!(
            count_solutions(&g, 3),
            p.count_solutions_cap3,
            "{}: count_solutions cap 3",
            p.id
        );

        let mut gc = g;
        assert_eq!(
            propagate(&mut gc),
            p.propagate_filled,
            "{}: propagate_filled",
            p.id
        );

        let a = analyze(&g);
        let e = &p.analysis;
        assert_eq!(a.givens, e.givens, "{}: givens", p.id);
        assert_eq!(a.empty_cells, e.empty_cells, "{}: empty_cells", p.id);
        assert_eq!(
            a.total_candidates, e.total_candidates,
            "{}: total_candidates",
            p.id
        );
        assert!(
            (a.candidate_entropy - e.candidate_entropy_bits).abs() < 1e-9,
            "{}: entropy {} vs {}",
            p.id,
            a.candidate_entropy,
            e.candidate_entropy_bits
        );
        assert_eq!(a.propagated, e.propagated, "{}: propagated", p.id);
        assert_eq!(a.solver_nodes, e.solver_nodes, "{}: solver_nodes", p.id);
        assert_eq!(a.max_depth, e.max_depth, "{}: max_depth", p.id);
        assert_eq!(a.unique, e.unique, "{}: unique", p.id);
        assert_eq!(a.solvable, e.solvable, "{}: solvable", p.id);
        assert_eq!(a.difficulty, e.difficulty, "{}: difficulty", p.id);
    }
}

#[test]
fn parse_error_cases() {
    assert!(Grid::from_puzzle_str("123").is_err());
    assert!(Grid::from_puzzle_str(&"a".repeat(81)).is_err());
    // disallowed characters are filtered before the length check (Racket quirk)
    assert!(Grid::from_puzzle_str(&format!("{}x", "1".repeat(81))).is_ok());
    let err = Grid::from_puzzle_str("123").unwrap_err();
    assert!(
        err.to_string()
            .contains("81-character string of digits 0-9")
    );
}
