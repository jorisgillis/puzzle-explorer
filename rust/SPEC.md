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

## 9. Form-constrained generation, technique analysis, and strategy measurement (wayfinder map #25)

Semantics locked by map #25's closed decision tickets
[#28](https://github.com/jorisgillis/puzzle-explorer/issues/28) (technique
set and depth scale),
[#29](https://github.com/jorisgillis/puzzle-explorer/issues/29) (generator
forms),
[#30](https://github.com/jorisgillis/puzzle-explorer/issues/30) (measurement
model),
[#31](https://github.com/jorisgillis/puzzle-explorer/issues/31) (difficulty
bands), with evidence from research tickets
[#26](https://github.com/jorisgillis/puzzle-explorer/issues/26) and
[#27](https://github.com/jorisgillis/puzzle-explorer/issues/27). The Racket
side (`sudoku/`) implements the same semantics; parity is pinned by tests
and fixtures. Vocabulary: root [GLOSSARY.md](../GLOSSARY.md), "Techniques"
section.

### 9.1 Technique analyzer (core: new `techniques` module; Racket: `solver.rkt` extension)

- Locked technique set, totally ordered in human difficulty order: L1 naked
  single, L2 hidden single, L3 locked candidates pointing, L4 locked
  candidates claiming, L5 naked pair, L6 hidden pair, L7 naked triple,
  L8 hidden triple, L9 X-Wing. Level 10 = "requires search".
- L1–L2 detection reuses the existing `propagate` fixpoint (naked + hidden
  singles); no separate L1/L2 detectors. L3–L9 are new deterministic scans
  over `UNITS`/`PEERS`.
- `solution_path(g) -> Vec<TechniqueStep>` — greedy easiest-step-first: at
  each state apply the easiest applicable technique (L1 first), record the
  step, repeat until solved or stuck. For a totally ordered set of sound
  rules the greedy max level equals the path-existence minimum (research
  #26), so depth = max level applied; stuck before solved → depth 10.
- `technique_depth(g) -> Option<u8>` and per-technique application counts
  (tally of the path) — computed only for uniquely solvable grids; otherwise
  `None`, reported as "n/a (not unique)".

### 9.2 Analysis fields (`complexity.rs` / `complexity.rkt`)

`Analysis` gains: `technique_depth: Option<u8>` (formatted with its tier
label), `required_technique: Option<Technique>`, `technique_counts` (per
L1–L9 along the solution path), and `difficulty_band` (§9.5).
`format_analysis` prints the new lines after the existing ones. The same
text is the single source for CLI `analyze`, CLI `generate` output, and both
GUI live analysis panels (`sudoku/ui.rkt`, `rust/app/src/ui.rs`) — full
metrics everywhere, no verbosity split.

### 9.3 Generation forms (`generator.rs` / `generator.rkt`, app CLIs)

- CLI, identical in `sudoku/main.rkt` and `rust/app/src/main.rs`:
  `generate <easy|medium|hard> [--max-depth N] [--requires TECH] [--band NAME]`
  - `TECH` ∈ `naked-single | hidden-single | pointing | claiming | naked-pair
    | hidden-pair | naked-triple | hidden-triple | x-wing`
  - `N` ∈ 1..=10 (solvable up to level N)
  - `NAME` ∈ `gentle | moderate | tough | diabolical | extreme`
- Combination: AND. `--max-depth` / `--requires` / `--band` are hard
  candidate filters; the givens target stays soft (best-of attempts, fewest
  givens wins), exactly as the no-flag path.
- Algorithm for flagged calls: constructive plant-and-carve — when
  `--requires TECH` is given, plant TECH's pattern in the grid, complete it,
  carve clues keeping a unique solution; `--max-depth` / `--band` / givens
  filter candidates within the attempt cap. Every emitted puzzle is verified
  by the technique analyzer (§9.1). No-flag `generate <diff>` keeps the
  legacy path (§4) unchanged.
- Failure: after 100 attempts without a matching candidate, print a clear
  error and exit non-zero, in both CLIs.
- `generate` output additionally prints the puzzle's measured technique
  depth and required technique next to the analysis block.
- GUI controls for the form flags: follow-on ticket
  [#44](https://github.com/jorisgillis/puzzle-explorer/issues/44), both GUIs.

### 9.4 Aggregate measurement (app CLIs)

- `measure <easy|medium|hard> [--max-depth N] [--requires TECH] [--band NAME] --count N`
  in both CLIs, byte-identical output format: aligned text tables of puzzles
  per depth level, puzzles per required technique, mean per-technique
  application counts, and a per-band breakdown.

### 9.5 Difficulty bands

| Band | Score range |
|---|---|
| gentle | 0–19 |
| moderate | 20–39 |
| tough | 40–59 |
| diabolical | 60–79 |
| extreme | 80–100 |

- Bands are orthogonal to the givens targets easy/medium/hard (40/32/26).
  `--band NAME` is a hard form filter (§9.3); `difficulty_band` is reported
  per puzzle (§9.2) and aggregated by `measure` (§9.4).
- The band table is identical in Racket and Rust and pinned by parity tests.

### 9.6 Testing

- Unit tests per technique on known patterns (each of L1–L9 fires and is
  counted correctly); depth/counts/band fields added to the parity fixtures
  for the fixed puzzle set (`core/tests/parity.rs` +
  `tests/fixtures/parity.json`); CLI contract tests for the new flags,
  `--requires`/`--band` validation, and the 100-attempt error path, in both
  implementations.
