//! CLI + GUI entry point of the Rust sudoku app.
//!
//! CLI surface mirrors `sudoku/main.rkt`: no args or `ui` launches the GUI,
//! `generate <easy|medium|hard>`, `solve <81-char>`, `analyze <81-char>`.
//! The egui app itself lands with ticket #17.

use clap::{Parser, Subcommand};
use sudoku_core::{Difficulty, Grid, analyze, format_analysis, generate_random, solve};

#[derive(Parser)]
#[command(
    name = "sudoku",
    version,
    about = "Puzzle Explorer - Sudoku (Rust port)"
)]
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
    std::process::exit(run());
}

fn run() -> i32 {
    // Parity with the Racket CLI: unknown/missing arguments print usage to
    // stdout and exit 0 (clap errors are rendered and printed as-is).
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            print!("{e}");
            return 0;
        }
    };
    match cli.command {
        None | Some(Command::Ui) => run_ui(),
        Some(Command::Generate { difficulty }) => {
            let difficulty = match difficulty.parse::<Difficulty>() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("{e}");
                    return 1;
                }
            };
            let (puzzle, _solution) = generate_random(difficulty);
            println!("{}", puzzle.to_pretty_string());
            println!("{}", puzzle.to_puzzle_string());
            println!("{}", format_analysis(&analyze(&puzzle)));
            0
        }
        Some(Command::Solve { puzzle }) => match Grid::from_puzzle_str(&puzzle) {
            Ok(g) => match solve(&g) {
                Some(solution) => {
                    println!("{}", solution.to_pretty_string());
                    0
                }
                None => {
                    println!("No solution.");
                    0
                }
            },
            Err(e) => {
                eprintln!("{e}");
                1
            }
        },
        Some(Command::Analyze { puzzle }) => match Grid::from_puzzle_str(&puzzle) {
            Ok(g) => {
                println!("{}", format_analysis(&analyze(&g)));
                0
            }
            Err(e) => {
                eprintln!("{e}");
                1
            }
        },
    }
}

fn run_ui() -> i32 {
    todo!("egui app — ticket #17")
}
