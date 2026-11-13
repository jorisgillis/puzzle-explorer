//! Core library of the Rust sudoku implementation.
//!
//! Spec: `../SPEC.md`. Full implementation lands with ticket #15; this
//! scaffold wires the module structure and CLI-facing stubs so the
//! workspace builds and tests green from the start.

pub mod complexity;
pub mod generator;
pub mod grid;
pub mod solver;

/// Stub for the CLI `generate` subcommand — returns `(puzzle, solution)`
/// as 81-char puzzle strings. Implemented by ticket #15.
pub fn generate_stub(_difficulty: &str) -> (String, String) {
    todo!("implement generator — ticket #15")
}

/// Stub for the CLI `solve` subcommand. Implemented by ticket #15.
pub fn solve_stub(_puzzle: &str) -> Option<String> {
    todo!("implement solver — ticket #15")
}

/// Stub for the CLI `analyze` subcommand. Implemented by ticket #15.
pub fn analyze_stub(_puzzle: &str) -> String {
    todo!("implement complexity — ticket #15")
}

#[cfg(test)]
mod tests {
    #[test]
    fn scaffold_smoke() {
        assert_eq!(2 + 2, 4);
    }
}
