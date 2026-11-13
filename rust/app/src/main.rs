//! CLI + GUI entry point of the Rust sudoku app.
//!
//! CLI surface mirrors `sudoku/main.rkt`: no args or `ui` launches the GUI,
//! `generate <easy|medium|hard>`, `solve <81-char>`, `analyze <81-char>`.
//! Core logic lands with ticket #15; the egui app with ticket #17.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sudoku", version, about = "Puzzle Explorer - Sudoku (Rust port)")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Launch the egui GUI (ticket #17)
    Ui,
    /// Generate a puzzle of the given difficulty: easy | medium | hard
    Generate { difficulty: String },
    /// Solve an 81-char puzzle string (0 or . for empty cells)
    Solve { puzzle: String },
    /// Print the complexity analysis of an 81-char puzzle string
    Analyze { puzzle: String },
}

fn main() {
    match Cli::parse().command {
        None | Some(Command::Ui) => run_ui(),
        Some(Command::Generate { difficulty }) => {
            let (puzzle, _solution) = sudoku_core::generate_stub(&difficulty);
            println!("{puzzle}");
        }
        Some(Command::Solve { puzzle }) => match sudoku_core::solve_stub(&puzzle) {
            Some(solution) => println!("{solution}"),
            None => println!("No solution."),
        },
        Some(Command::Analyze { puzzle }) => {
            println!("{}", sudoku_core::analyze_stub(&puzzle));
        }
    }
}

fn run_ui() {
    todo!("egui app — ticket #17")
}
