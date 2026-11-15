//! Solver gallery: three solvers behind one [`Solver`] trait, for comparing
//! algorithms side by side (map #33, ticket #36 decision). The production
//! solver (`crate::solver`) is untouched; the propagation+MRV entry is an
//! adapter delegating to it. `all_solvers` is the registry harnesses
//! (benchmark, docs) iterate uniformly.

pub mod dlx;
pub mod naive;
pub mod propagation_mrv;

use crate::grid::Grid;
use crate::solver::SearchStats;

/// The solver contract every gallery solver implements.
pub trait Solver {
    fn name(&self) -> &'static str;
    /// Solve without mutating the input; `None` for invalid or unsolvable.
    fn solve(&self, g: &Grid) -> Option<Grid>;
    /// Solve with search stats (nodes visited, max depth).
    fn solve_with_stats(&self, g: &Grid) -> (Option<Grid>, SearchStats);
    /// Count solutions, stopping early once `count` reaches `cap`.
    /// Invalid grids have 0 solutions.
    fn count_solutions(&self, g: &Grid, cap: u64) -> u64;
}

/// The gallery registry: every solver as (name, solver), in gallery order.
pub fn all_solvers() -> [(&'static str, &'static dyn Solver); 3] {
    [
        ("naive", &naive::Naive as &dyn Solver),
        (
            "propagation-mrv",
            &propagation_mrv::PropagationMrv as &dyn Solver,
        ),
        ("dlx", &dlx::Dlx as &dyn Solver),
    ]
}
