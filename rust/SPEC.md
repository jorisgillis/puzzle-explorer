# Core crate spec — `sudoku-core`

Spec for the core library crate of the Rust reimplementation. Behavior is
spec'd for parity with the Racket reference (`sudoku/*.rkt`, `tests/*.rkt`),
using the vocabulary of the root [GLOSSARY.md](../GLOSSARY.md). Stack and
dependency decisions come from the closed research tickets
[#9](https://github.com/jorisgillis/puzzle-explorer/issues/9) (eframe/egui
0.37.0) and
[#10](https://github.com/jorisgillis/puzzle-explorer/issues/10) (rand 0.10,
clap 4, serde/serde_json dev-only); parity fixtures come from
[#11](https://github.com/jorisgillis/puzzle-explorer/issues/11).

## 1. Workspace placement and crate names

```
rust/
  Cargo.toml          # workspace: members = ["core", "app"]
  SPEC.md             # this file
  core/               # lib crate `sudoku-core`
    Cargo.toml        # deps: rand = "0.10"; dev-deps: serde, serde_json
    src/lib.rs
    src/grid.rs
    src/solver.rs
    src/generator.rs
    src/complexity.rs
    tests/fixtures/parity.json   # from ticket #11 findings
  app/                # bin crate `sudoku` (scaffolded by ticket #14)
```

`core` is a pure library: no GUI, no CLI parsing, no I/O beyond tests.

## 2. Grid representation — `grid.rs`

- `pub struct Grid([u8; 81])` — row-major cells 0-80, `0` = empty cell.
  Internally an array (not `Vec<u8>`): serde derives only cover arrays up to
  length 32, so serde is implemented manually (see below).
- Precomputed consts: `UNITS: [[usize; 9]; 27]` (9 rows, 9 cols, 9 boxes) and
  `PEERS: [[usize; 20]; 81]` — same tables as `grid.rkt`.
- API:
  - `Grid::empty() -> Grid`
  - `Grid::from_puzzle_str(s: &str) -> Result<Grid, ParseGridError>` — exactly
    81 chars, each `1`-`9`, `0`, or `.` (`0`/`.` → empty); otherwise an error
    whose message matches the Racket contract: `expected: 81-character string
    of digits 0-9`. (Only chars in `0123456789.` are accepted, like
    `string->grid`; other characters are rejected, not skipped.)
  - `grid.to_puzzle_string() -> String` (81 chars, `0` for empty)
  - `grid.get(i) -> u8`, `grid.set(i, v)`
  - `grid.is_empty_cell(i) -> bool`, `grid.is_valid() -> bool`,
    `grid.is_complete() -> bool` — same definitions as `grid.rkt`.
  - `grid.peers(i) -> &[usize; 20]`, `grid.candidates(i) -> Vec<u8>` — values
    1-9 not present among peers (`candidates-of`).
- Serde: `Grid` serializes as its 81-char puzzle string (custom
  `Serialize`/`Deserialize` impls), so parity fixtures stay human-readable.

## 3. Solver — `solver.rs`

- `propagate(g: &mut Grid) -> usize` — fill naked singles (empty cell with
  exactly one candidate) and hidden singles (a value fitting exactly one empty
  cell of a unit), looping until no progress; returns cells filled. Same
  fixpoint semantics as `propagate` in `solver.rkt` (no validity check inside).
- `pick_cell(g: &Grid) -> Option<(usize, Vec<u8>)>` — MRV: first empty cell
  (lowest index on ties) with the fewest candidates; `None` if any empty cell
  has zero candidates (dead end).
- `solve_with_stats(g: &Grid) -> (Option<Grid>, SearchStats)` where
  `SearchStats { nodes: u64, max_depth: u64 }` — MRV backtracking on a copy;
  input not mutated; invalid input → `(None, stats with nodes = 0)` (Racket
  skips the search entirely when `valid?` fails). `nodes` counts every
  `search` invocation including the root; `max_depth` tracks recursion depth.
- `solve(g: &Grid) -> Option<Grid>` — `solve_with_stats` discarding stats.
- `count_solutions(g: &Grid, cap: u64) -> u64` — **correct** capped counting:
  continue with the next candidate on a dead end; stop as soon as
  `count >= cap`; invalid grid → 0.
  - **Deliberate divergence from Racket** (documented in fixtures
    `known_quirks`): Racket's `count-solutions` uses `for/and`, which aborts
    the whole search on the first dead-end branch and can therefore return 0
    for solvable unique puzzles (e.g. AI Escargot), making Racket's
    `unique?` wrong there. Rust implements the intended semantics; parity
    fixtures record Racket's actual (buggy) outputs alongside.

## 4. Generator — `generator.rs`

- `pub enum Difficulty { Easy, Medium, Hard }` with target givens
  easy 40 / medium 32 / hard 26 (`difficulty-givens`).
- `generate_complete_grid(rng: &mut impl Rng) -> Grid` — backtracking with
  shuffled candidates (`rand` 0.10: `rng.random_range`, `SliceRandom::shuffle`).
- `remove_clues(solution: &Grid, target: usize, order: &[(usize, usize)], rng)
  -> Grid` — iterate symmetric cell pairs, remove each clue if
  `count_solutions(puzzle, 2) == 1` (restore otherwise); stop when givens ≤
  target.
- `generate_puzzle(difficulty, rng) -> (Grid, Grid)` — 10 attempts with
  shuffled pair orders, keep the candidate with the fewest givens; returns
  `(puzzle, solution)`. Randomized: tests assert **invariants** (givens count
  target or best-of-10, unique solution, puzzle ⊆ solution), never golden
  grids.

## 5. Complexity — `complexity.rs`

- `pub struct Analysis` with fields: `givens`, `empty_cells`,
  `total_candidates`, `candidate_entropy: f64`, `propagated`,
  `solver_nodes`, `max_depth`, `unique: bool`, `solvable: bool`,
  `difficulty: u32`.
- `analyze(g: &Grid) -> Analysis` — same definitions as `complexity.rkt`:
  candidates/entropy over empty cells of the *input* grid (entropy =
  Σ log2(candidate count), full `f64` precision); `propagated` from
  `propagate` on a copy; `solver_nodes`/`max_depth` from `solve_with_stats`;
  `unique` = `count_solutions(g, 2) == 1`; `solvable` = `solve(g).is_some()`;
  `difficulty` = `round(100 * (0.6 * min(1, nodes/5000) + 0.4 * (1 - givens/81)))`
  as an integer 0-100.
- `format_analysis(a: &Analysis) -> String` — the exact report text the Racket
  CLI prints (header + the ten `  label: value` lines, entropy with 2
  decimals), so CLI output is byte-identical for the same input.

## 6. Error and CLI-contract parity (consumed by the app crate)

From the fixtures (#11): malformed puzzle string → parse error (message above)
and exit code 1 at the CLI; unsolvable `solve` → prints `No solution.`, exit 0;
unknown/missing subcommand → usage text, exit 0; `generate` with an invalid
difficulty → error, exit 1. The core crate exposes typed errors; exit codes
and messages are the app crate's job.

## 7. Testing

- Unit tests per module mirroring `tests/*.rkt` (grid facts, candidates, peers,
  propagate, solve/stats, count-solutions, generator invariants, complexity).
- `core/tests/parity.rs` loads `tests/fixtures/parity.json` (dev-deps serde +
  serde_json) and asserts parity on fixed puzzles: solution equality, stats
  (nodes/max depth), analysis fields, error cases. Where Racket's behavior is
  buggy (the `count-solutions` quirk), the fixture's `known_quirks` section
  records both Racket's actual output and the corrected Rust expectation.

## 8. Out of scope for the core crate

GUI, CLI parsing, eframe/egui, clap — all belong to the `app` crate
(tickets #14, #17). No thiserror/anyhow/logging for the core scope (#10).
