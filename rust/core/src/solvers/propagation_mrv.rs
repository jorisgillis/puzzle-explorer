//! Adapter exposing the production propagation + MRV solver
//! (`crate::solver`) as a gallery entry. Delegates without duplicating or
//! modifying the production module.

use crate::grid::Grid;
use crate::solver::SearchStats;

use super::Solver;

pub struct PropagationMrv;

impl Solver for PropagationMrv {
    fn name(&self) -> &'static str {
        "propagation-mrv"
    }
    fn solve(&self, g: &Grid) -> Option<Grid> {
        crate::solver::solve(g)
    }
    fn solve_with_stats(&self, g: &Grid) -> (Option<Grid>, SearchStats) {
        crate::solver::solve_with_stats(g)
    }
    fn count_solutions(&self, g: &Grid, cap: u64) -> u64 {
        crate::solver::count_solutions(g, cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const README: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

    #[test]
    fn delegates_to_production_solver() {
        let g = Grid::from_puzzle_str(README).unwrap();
        let (sol, stats) = PropagationMrv.solve_with_stats(&g);
        let (prod_sol, prod_stats) = crate::solver::solve_with_stats(&g);
        assert_eq!(
            sol.map(|x| x.to_puzzle_string()),
            prod_sol.map(|x| x.to_puzzle_string())
        );
        assert_eq!(stats, prod_stats);
        assert_eq!(
            PropagationMrv.count_solutions(&g, 2),
            crate::solver::count_solutions(&g, 2)
        );
    }
}
