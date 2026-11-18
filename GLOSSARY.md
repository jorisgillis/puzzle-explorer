# Sudoku

Shared vocabulary for the puzzle-explorer project: generating, solving, playing,
and measuring classic sudoku puzzles. Both implementations (Racket in `sudoku/`,
Rust in `rust/`) use these terms in code, docs, and the issue tracker.

## The puzzle

**Puzzle**:
A 9x9 grid of 81 cells, partially filled with givens, intended to have exactly one
solution. _Avoid_: riddle, board (ambiguous with the UI widget)

**Givens**:
The cells filled in the puzzle as given; immutable during play. _Avoid_: clues, fixed cells

**Cell**:
One of the 81 positions, indexed 0-80 row-major. _Avoid_: square

**Empty cell**:
A cell with no value yet (value 0; `0` or `.` in a puzzle string).

**Unit**:
A row, column, or 3x3 box — one of the 27 groups whose cells must each contain
the values 1-9 exactly once.

**Peers**:
The 20 cells that share at least one unit with a given cell.

**Puzzle string**:
The 81-character row-major serialization of a grid: digits 1-9 for filled cells,
`0` or `.` for empty cells. _Avoid_: board string

**Valid grid**:
A grid in which no unit contains a repeated non-zero value. _Avoid_: legal, consistent

**Complete grid**:
A valid grid with all 81 cells filled.

**Solution**:
A complete grid consistent with the puzzle's givens.

## Solving

**Candidates** (pencil marks):
The values 1-9 not eliminated for an empty cell by the values already present
among its peers.

**Constraint propagation**:
Repeatedly filling every cell whose candidate set has been reduced to exactly one
value, until no further cell can be filled this way.

**Naked single**:
An empty cell with exactly one candidate.

**Hidden single**:
A value that fits in exactly one remaining cell of a unit.

**MRV** (minimum remaining values):
The search heuristic that branches on the empty cell with the fewest candidates.

**Backtracking search**:
Depth-first search that assigns a candidate to a cell, recurses, and undoes the
assignment when a dead end is reached.

**Solver nodes / max depth**:
The number of search nodes explored and the deepest recursion level reached —
measures of search effort.

**Unique solution**:
The property of a puzzle having exactly one solution, verified by counting
solutions up to a small cap.

**Solver contract**:
The shared set of operations every solver in the solver gallery implements:
solve, solve with search stats, and count solutions up to a cap.

**Solver gallery**:
The side-by-side collection of solver implementations behind one solver
contract (`sudoku/solvers/` in Racket, `rust/core/src/solvers/` in Rust), kept
for comparing algorithms; the production solver (`solver.rkt`, `solver.rs`)
is untouched, and one gallery entry delegates to it.

**Naive backtracking solver**:
Backtracking search that scans cells 0-80 in order and tries digits 1-9 in
order using grid candidates — no constraint propagation, no MRV. Distinct
from "backtracking search" itself, which is the general technique.

**Propagation + MRV solver**:
Constraint propagation (naked/hidden singles) followed by MRV backtracking
search — the production solver, exposed in the gallery through an adapter.

**DLX solver**:
Algorithm X with Dancing Links over the exact-cover matrix of 729 candidate
placements by 324 constraint columns (cell, row-digit, column-digit, and
box-digit groups); branches on the minimum-size column with O(1) cover/uncover.

**Link updates**:
The number of link removals/restorations a DLX search performs — Knuth's
machine-independent cost metric for exact-cover search.

## Techniques

**Technique**:
A named, sound human deduction rule — a placement or candidate elimination
that preserves all solutions of a puzzle. The project uses the locked
technique depth scale below. _Avoid_: strategy (broader; a strategy may combine
several techniques)

**Technique depth scale**:
The totally ordered technique set used for form-constrained generation and
strategy measurement, in human difficulty order (locked in map #25, ticket
#28): L1 naked single, L2 hidden single, L3 locked candidates (pointing),
L4 locked candidates (claiming), L5 naked pair, L6 hidden pair, L7 naked
triple, L8 hidden triple, L9 X-Wing, and level 10 "requires search".
Reporting tiers: singles (L1–L2), locked candidates (L3–L4), subsets
(L5–L8), fish (L9).

**Solvable up to level k**:
Solvable using only the first k techniques of the technique depth scale.

**Technique depth**:
The minimum k (1–10) such that a puzzle is solvable up to level k. Level 10
means the puzzle requires backtracking search.

**Requires technique T**:
The puzzle is not solvable using only techniques strictly below T on the
technique depth scale.

**Required technique**:
The technique whose level equals the puzzle's technique depth.

**Solution path**:
The sequence of technique applications produced by greedy
easiest-step-first solving: at each state apply the easiest applicable
technique, until the puzzle is solved or no technique applies.

**Technique application counts**:
How many times each technique is applied along a puzzle's solution path.

## Generating

**Complete-grid generation**:
Producing a random complete grid by backtracking with shuffled candidate order.

**Clue removal**:
Removing givens from a complete grid one at a time, keeping a removal only if the
puzzle still has a unique solution, until a target givens count is reached.

**Difficulty**:
The target givens count of a generated puzzle: easy 40, medium 32, hard 26.

**Generation form**:
The set of constraints a generated puzzle must satisfy: the givens target
(easy/medium/hard), an optional max technique depth, an optional required
technique, and an optional difficulty band — combined with AND. Form
constraints are hard filters; the givens target is soft (best-of attempts,
fewest givens wins).

## Measuring complexity

**Total candidates**:
The sum of candidate counts over all empty cells before any deduction.

**Candidate entropy**:
The sum of log2(candidate count) over all empty cells — the information content,
in bits, of the initial search space.

**Cells filled by propagation**:
How many empty cells constraint propagation alone can fill from the givens.

**Difficulty score**:
A 0-100 score: 100 * (0.6 * min(1, solver_nodes/5000) + 0.4 * (1 - givens/81)) —
60% search effort, 40% clue scarcity.

**Difficulty band**:
A label for an equal-width range of the 0-100 difficulty score: gentle 0–19,
moderate 20–39, tough 40–59, diabolical 60–79, extreme 80–100. Bands are
orthogonal to the givens targets easy/medium/hard (locked in map #25, ticket
#31).

**Technique distribution**:
The aggregate over a generated set of puzzles of a given form: how many
puzzles fall at each technique depth and required technique, plus the mean
technique application counts — the comparison that shows where a strategy
works better or worse across forms.

## Playing

**User entry**:
A value the player typed into a non-given cell, displayed distinctly from givens.

**Conflict**:
A user entry whose value already appears among a peer's values.

**Selection / highlight**:
The currently selected cell, with its row, column, and box tinted.

**Live analysis**:
The complexity report recomputed from the current board (givens plus user entries)
on every change.

**Candidate heat overlay**:
An optional per-cell background color encoding how many candidates remain for an
empty cell on the current board: green for 1 candidate up to red for 9 (one discrete
color per count), error red for 0 (a dead cell). Recomputed live on every change;
filled cells are never colored.

## Documentation

**Explainer page**:
A page under `docs/` following the reusable template in `docs/README.md`
(Question / TL;DR / Deep dive / Diagram / Worked example / Goofy image /
Complexity callout / See the code / Further reading), written for a public
audience: playful tone, every factual claim cited or code-linked.

**Benchmark harness**:
The per-language runner that solves a shared fixture set with every solver in
the solver gallery and records wall time, solver nodes, max depth (plus link
updates for the DLX solver): `sudoku/solvers/benchmark.rkt` in Racket,
`rust/core/benches/bench.rs` (criterion) in Rust.

**Benchmark fixture set**:
A committed plain-text file of 81-character puzzles shared by both benchmark
harnesses (`tests/fixtures/bench-*.txt`): Norvig's top95 hard set, Royle's
49,151 17-clue collection, and named extreme puzzles.
