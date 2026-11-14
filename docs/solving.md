# Solving and measuring difficulty

CLI examples (Racket; the Rust CLI takes the same subcommands — see
[implementations.md](implementations.md)):

```
racket sudoku/main.rkt solve   "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
racket sudoku/main.rkt analyze "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
```

`solve` prints the completed grid (or reports the puzzle as unsolvable);
`analyze` prints the complexity report below.

## How the solver thinks

`propagate` keeps filling **naked singles** (a cell with one candidate left)
and **hidden singles** (a value with one spot left in a unit) until nothing
more follows. When no single remains, `pick-cell` branches on the empty cell
with the fewest candidates (**MRV**) and backtracks on dead ends.
`count-solutions` with a small cap proves whether the puzzle has exactly one
solution.

Naked single — every value except 7 already appears among the cell's peers:

```
  peers of this cell contain 1,2,3,4,5,6,8,9
  +---+
  | 7 |   candidates = {7}  ->  fill 7, no guessing
  +---+
```

Hidden single — in this row, 4 can only go in the marked cell (the other
empty cells already see a 4 in their column or box):

```
  row:   1 . . | 4 . . | . 9 .        value 4: only one spot left in the row
             ^
  that cell's candidates are {4, 6}, but 6 also fits elsewhere in the row
  ->  hidden single: place 4 there
```

When propagation gets stuck, the search branches (MRV first) and undoes dead
ends:

```
              [cell = 2]                     <- branch on MRV cell
              /        \
        [cell = 7]    [cell = 8]             <- second branch
        /     \           \
     ...      #f (dead)    #f (dead)         <- backtrack: undo, try next value
```

## Complexity metrics

`analyze` reports, for a puzzle with `g` givens and `n` solver nodes explored:

- **givens / empty cells** — raw clue count: `g` and `81 - g`.
- **total candidates** — sum of pencil-mark counts over empty cells before any deduction.
- **candidate entropy** — `sum log2(#candidates)` over empty cells: the information
  content (in bits) of the initial search space.
- **cells filled by propagation** — how much is solvable by naked/hidden singles alone.
- **solver nodes / max depth** — search effort of the MRV backtracking solver.
- **unique?** — whether the puzzle has exactly one solution (verified with a capped
  solution count).
- **difficulty score (0-100)** — `100 * (0.6 * min(1, n/5000) + 0.4 * (1 - g/81))`,
  i.e. 60% search effort, 40% clue scarcity.

### Illustrating the metrics

**Candidate entropy.** Each empty cell contributes `log2` of its candidate count:

```
  cell with candidates {2, 5, 9}:   log2(3) ≈ 1.58 bits
  cell with candidates {4}:         log2(1) = 0 bits   (already forced)
  cell with candidates {1..9}:      log2(9) ≈ 3.17 bits (total ignorance)
  puzzle entropy = sum over all 51 empty cells -> 75.17 bits for the example below
```

**Propagation.** How many of the 51 empty cells are naked/hidden singles already:
the example puzzle fills all 51 by propagation alone; the hard puzzle below
fills only 8 — the other 48 need real search.

**Search effort.** Nodes = decisions visited in the tree above; depth = deepest
recursion. The example needs 52 nodes / depth 51 (a straight propagation
cascade), the hard puzzle 218 nodes / depth 58.

**Difficulty score, worked example** (classic puzzle: `n = 52`, `g = 30`):

```
  100 * (0.6 * min(1, 52/5000) + 0.4 * (1 - 30/81))
  = 100 * (0.6 * 0.0104        + 0.4 * 0.6296)
  = 100 * (0.00624             + 0.25185)  = 25.8  ->  score 26
```

## Example gallery

All numbers below are real `analyze` output:

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

| puzzle | givens | candidates | entropy | propagated | nodes | depth | unique | score |
|---|---|---|---|---|---|---|---|---|
| classic example above | 30 | 153 | 75.17 bits | 51 | 52 | 51 | yes | 26 |
| `generate hard` (940300010…) | 25 | 214 | 105.27 bits | 8 | 57 | 56 | yes | 28 |
| published hard puzzle (23 givens, "escargot"-style) | 23 | 223 | 109.82 bits | 1 | 218 | 58 | `#f` in Racket* | 31 |

\* known divergence: Racket's `count-solutions` undercounts because of a
`for/and` short-circuit; the Rust implementation reports `true` here. See
[implementations.md](implementations.md) for details.

The contrast is the point: same 81 cells, but the escargot-style puzzle has
more entropy (109.82 vs 75.17 bits), almost nothing solvable by singles
(1 vs 51), and 4x the search effort (218 vs 52 nodes) — hence a higher
difficulty score despite having *fewer* givens.

## Generating puzzles

`generate <easy|medium|hard>` builds a random complete grid by backtracking
with shuffled candidates, then removes clues one at a time, keeping a removal
only if the puzzle still has a unique solution, until the difficulty's target
givens count is reached (easy 40, medium 32, hard 26). The best of several
removal attempts is kept.
