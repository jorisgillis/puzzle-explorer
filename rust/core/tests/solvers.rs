//! Gallery tests: every solver in `all_solvers` agrees with the golden
//! Racket parity fixtures (solutions and solution counts), plus registry,
//! invalid-grid, and cross-solver agreement checks (map #33, ticket #40).

use serde::Deserialize;
use sudoku_core::Grid;
use sudoku_core::solvers::all_solvers;
use sudoku_core::solvers::dlx::Dlx;

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
}

#[derive(Deserialize)]
struct SolveFixture {
    solution: Option<String>,
}

// AI Escargot (Inkala 2006), a hard puzzle outside the fixture file
const ESCARGOT: &str =
    "100007090030020008009600500005300900010080002600004000300000010040000007007000300";

#[test]
fn registry_lists_the_three_gallery_solvers_in_order() {
    let names: Vec<&str> = all_solvers().iter().map(|(n, _)| *n).collect();
    assert_eq!(names, ["naive", "propagation-mrv", "dlx"]);
}

#[test]
fn gallery_solvers_match_racket_parity_fixtures() {
    let json = include_str!("fixtures/parity.json");
    let fx: Fixtures = serde_json::from_str(json).unwrap();
    for p in &fx.puzzles {
        let g = Grid::from_puzzle_str(&p.input).unwrap_or_else(|e| panic!("{}: {e}", p.id));
        let unique = p.count_solutions_cap2 == 1;
        for (name, s) in all_solvers() {
            let (sol, stats) = s.solve_with_stats(&g);
            match &p.solve.solution {
                Some(expected) if unique => {
                    // unique puzzle: every solver must find the one solution
                    assert_eq!(
                        sol.as_ref().map(Grid::to_puzzle_string).as_deref(),
                        Some(expected.as_str()),
                        "{}: {} solution mismatch",
                        p.id,
                        name
                    );
                    assert!(stats.nodes > 0, "{}: {} nodes", p.id, name);
                }
                Some(_) => {
                    // multi-solution puzzle: any valid completion of the givens
                    let sol = sol.unwrap_or_else(|| panic!("{}: {name}: no solution", p.id));
                    assert!(sol.is_complete() && sol.is_valid(), "{}: {name}", p.id);
                    for i in 0..81 {
                        if g.get(i) != 0 {
                            assert_eq!(sol.get(i), g.get(i), "{}: {name} givens", p.id);
                        }
                    }
                }
                None => assert!(sol.is_none(), "{}: {} expected no solution", p.id, name),
            }
            assert_eq!(
                s.count_solutions(&g, 2),
                p.count_solutions_cap2,
                "{}: {} count cap 2",
                p.id,
                name
            );
            assert_eq!(
                s.count_solutions(&g, 3),
                p.count_solutions_cap3,
                "{}: {} count cap 3",
                p.id,
                name
            );
        }
    }
}

#[test]
fn all_solvers_agree_on_ai_escargot() {
    let g = Grid::from_puzzle_str(ESCARGOT).unwrap();
    let expected = sudoku_core::solve(&g).unwrap().to_puzzle_string();
    for (name, s) in all_solvers() {
        let sol = s.solve(&g).unwrap_or_else(|| panic!("{name}: no solution"));
        assert_eq!(sol.to_puzzle_string(), expected, "{name}");
    }
}

#[test]
fn invalid_grids_short_circuit_for_every_solver() {
    let bad = Grid::from_puzzle_str(&format!("11{}", "0".repeat(79))).unwrap();
    for (name, s) in all_solvers() {
        let (sol, stats) = s.solve_with_stats(&bad);
        assert!(sol.is_none(), "{name}");
        assert_eq!(stats.nodes, 0, "{name}");
        assert_eq!(stats.max_depth, 0, "{name}");
        assert_eq!(s.count_solutions(&bad, 2), 0, "{name}");
    }
}

#[test]
fn dlx_reports_link_updates() {
    let g = Grid::from_puzzle_str(ESCARGOT).unwrap();
    let (_, ds) = Dlx.solve_with_dlx_stats(&g);
    assert!(ds.link_updates > 0);
}

#[test]
fn solving_does_not_mutate_input() {
    let g = Grid::from_puzzle_str(ESCARGOT).unwrap();
    let before = g.to_puzzle_string();
    for (_, s) in all_solvers() {
        let _ = s.solve(&g);
    }
    assert_eq!(g.to_puzzle_string(), before);
}
