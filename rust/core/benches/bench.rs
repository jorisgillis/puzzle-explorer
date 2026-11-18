//! Criterion benchmark for the solver gallery (map #33, tickets #37/#41).
//! Same shared fixtures as the Racket harness (`sudoku/solvers/benchmark.rkt`):
//! `tests/fixtures/bench-*.txt`. Per-puzzle benchmarks for the extremes and
//! top95 sets; a whole-slice pass over a labeled prefix of the Royle 17-clue
//! set (slice sizes per solver keep naive's cost sane: the full 49,151 file
//! is covered by the Racket run).

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use sudoku_core::Grid;
use sudoku_core::solvers::Solver;
use sudoku_core::solvers::all_solvers;

fn load_fixtures(file: &str, limit: Option<usize>) -> Vec<Grid> {
    let path = format!("{}/../../tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), file);
    let content =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        out.push(Grid::from_puzzle_str(line).unwrap());
        if limit.is_some_and(|l| out.len() >= l) {
            break;
        }
    }
    out
}

fn bench(c: &mut Criterion) {
    let extremes = load_fixtures("bench-extremes.txt", None);
    let top95 = load_fixtures("bench-top95.txt", None);
    let royle = load_fixtures("bench-royle17.txt", Some(1000));

    for (name, solver) in all_solvers() {
        let top95_slice: &[Grid] = if name == "naive" { &top95[..10] } else { &top95 };
        for (set, puzzles) in [("extremes", &extremes[..]), ("top95", top95_slice)] {
            let mut group = c.benchmark_group(format!("{set}/{name}"));
            group.sample_size(10);
            for (i, p) in puzzles.iter().enumerate() {
                group.bench_with_input(BenchmarkId::from_parameter(i), p, |b, g| {
                    b.iter(|| solver.solve(g))
                });
            }
            group.finish();
        }
        let slice: &[Grid] = match name {
            "naive" => &royle[..5],
            "propagation-mrv" => &royle[..100],
            _ => &royle,
        };
        let mut group = c.benchmark_group(format!("royle17-prefix/{name}"));
        group.sample_size(10);
        group.bench_function("solve-slice", |b| {
            b.iter(|| {
                for p in slice {
                    std::hint::black_box(solver.solve(p));
                }
            })
        });
        group.finish();
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
