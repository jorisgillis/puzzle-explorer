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

## Generating

**Complete-grid generation**:
Producing a random complete grid by backtracking with shuffled candidate order.

**Clue removal**:
Removing givens from a complete grid one at a time, keeping a removal only if the
puzzle still has a unique solution, until a target givens count is reached.

**Difficulty**:
The target givens count of a generated puzzle: easy 40, medium 32, hard 26.

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
