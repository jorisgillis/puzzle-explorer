# Solving and measuring difficulty

CLI examples (Racket; the Rust CLI takes the same subcommands — see
[implementations.md](implementations.md)):

```
racket sudoku/main.rkt solve   "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
racket sudoku/main.rkt analyze "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
```

## How the solver thinks

`propagate` keeps filling naked singles (a cell with one candidate left) and
hidden singles (a value with one spot left in a unit) until nothing more follows.
When no single remains, `pick-cell` branches on the empty cell with the fewest
candidates (MRV) and backtracks on dead ends. `count-solutions` with a small cap
proves whether the puzzle has exactly one solution.

## Complexity metrics

For a puzzle with `g` givens and `n` solver nodes explored (MRV backtracking):

- **givens / empty cells** — raw clue count.
- **total candidates** — sum of pencil-mark counts over empty cells before any deduction.
- **candidate entropy** — `sum log2(#candidates)` over empty cells: the information
  content (in bits) of the initial search space.
- **cells filled by propagation** — how much is solvable by naked/hidden singles alone.
- **solver nodes / max depth** — search effort of the MRV backtracking solver.
- **unique?** — whether the puzzle has exactly one solution (verified with a capped
  solution count).
- **difficulty score (0-100)** — `100 * (0.6 * min(1, n/5000) + 0.4 * (1 - g/81))`,
  i.e. 60% search effort, 40% clue scarcity.

## Sample `analyze` output

```
$ racket sudoku/main.rkt analyze "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
Complexity analysis
  givens: 30
  empty cells: 51
  total candidates: 153
  candidate entropy (bits): 75.17
  cells filled by propagation: 51
  solver nodes explored: 52
  max search depth: 51
  unique solution: #t
  solvable: #t
  difficulty score (0-100): 26
```

## Generating puzzles

`generate <easy|medium|hard>` builds a random complete grid by backtracking
with shuffled candidates, then removes clues one at a time, keeping a removal
only if the puzzle still has a unique solution, until the difficulty's target
givens count is reached (easy 40, medium 32, hard 26). The best of several
removal attempts is kept.
