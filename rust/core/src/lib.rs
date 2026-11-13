//! Core library of the Rust sudoku implementation: grid model, MRV solver
//! with constraint propagation, uniqueness-checked generator, and complexity
//! metrics. Spec: `../SPEC.md`. GUI and CLI live in the `app` crate.

pub mod complexity;
pub mod generator;
pub mod grid;
pub mod solver;

pub use complexity::{Analysis, analyze, difficulty_score, format_analysis};
pub use generator::{
    Difficulty, generate_complete_grid, generate_puzzle, generate_random, remove_clues,
};
pub use grid::{CELL_COUNT, Grid, PEERS, ParseGridError, UNITS};
pub use solver::{SearchStats, count_solutions, propagate, solve, solve_with_stats};
