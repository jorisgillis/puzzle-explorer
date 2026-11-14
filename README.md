# puzzle-explorer

**81 cells. One solution. Two languages. Zero mercy.**

Classic sudoku, explored from every angle: puzzles are **generated** with a
uniqueness guarantee, **solved** by constraint propagation plus MRV backtracking,
**played** in a native GUI, and **graded** with information-theoretic complexity
metrics. Two implementations live side by side — **Racket** (`sudoku/`) and
**Rust + egui** (`rust/`) — held to identical behavior by golden parity fixtures.

## The face of the puzzle

Every puzzle is an 81-character string, row-major, `0` (or `.`) for an empty cell:

```
530070000600195000098000060800060003400803001700020006060000280000419005000080079
```

That string hides exactly one completed grid:

```
+-------+-------+-------+       +-------+-------+-------+
| 5 3 . | . 7 . | . . . |       | 5 3 4 | 6 7 8 | 9 1 2 |
| 6 . . | 1 9 5 | . . . |       | 6 7 2 | 1 9 5 | 3 4 8 |
| . 9 8 | . . . | . 6 . |  -->  | 1 9 8 | 3 4 2 | 5 6 7 |
+-------+-------+-------+       +-------+-------+-------+
| 8 . . | . 6 . | . . 3 |       | 8 5 9 | 7 6 1 | 4 2 3 |
| 4 . . | 8 . 3 | . . 1 |       | 4 2 6 | 8 5 3 | 7 9 1 |
| 7 . . | . 2 . | . . 6 |       | 7 1 3 | 9 2 4 | 8 5 6 |
+-------+-------+-------+       +-------+-------+-------+
| . 6 . | . . . | 2 8 . |       | 9 6 1 | 5 3 7 | 2 8 4 |
| . . . | 4 1 9 | . . 5 |       | 2 8 7 | 4 1 9 | 6 3 5 |
| . . . | . 8 . | . 7 9 |       | 3 4 5 | 2 8 6 | 1 7 9 |
+-------+-------+-------+       +-------+-------+-------+
   the puzzle                        the only solution
```

This repo can generate such a puzzle, prove it has exactly one solution, and
tell you how much of a fight it puts up.

## Quick start

Play in the native GUI:

```
racket sudoku/main.rkt ui
```

Or stay in the terminal — generate a fresh puzzle, solve one, or grade one
(difficulty targets: easy 40, medium 32, hard 26 givens, always unique):

```
racket sudoku/main.rkt generate hard
racket sudoku/main.rkt solve   "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
racket sudoku/main.rkt analyze "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
```

`analyze` answers "how hard is this, really?":

```
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

## Playing the GUI

- Click a cell, type `1`-`9` to enter a value, `0`/`Delete` to clear; arrow keys move the selection.
- Givens are bold blue, user entries black, conflicting entries red.
- The selected cell's row/column/box is highlighted.
- Buttons: New Easy/Medium/Hard, Solve, Hint, Check, Analyze; a Pencils toggle
  shows candidate numbers in empty cells.
- A Heat toggle overlays each empty cell with a background color encoding how many
  candidates remain for it on the current board (green = 1 … red = 9, error red = 0),
  recomputed live as you play — a mood ring per cell.
- Analyze toggles a live analysis panel with the complexity metrics below,
  recomputed from the current board (givens plus your entries) on every change
  while you play.

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

## Layout

```
sudoku/           Racket implementation
  info.rkt        package metadata (installable via raco pkg)
  grid.rkt        9x9 grid model, validation, string/pretty rendering
  solver.rkt      MRV backtracking solver, constraint propagation, solution counting
  generator.rkt   complete-grid generation + clue removal with uniqueness guarantee
  complexity.rkt  complexity metrics and 0-100 difficulty score
  ui.rkt          native racket/gui front end
  main.rkt        CLI entry point
rust/             Rust implementation (cargo workspace)
  core/           sudoku-core lib crate: grid, solver, generator, complexity
  app/            sudoku bin crate: clap CLI + egui GUI
  SPEC.md         core crate spec
  SPEC-UI.md      egui app behavior spec (ported from sudoku/ui.rkt)
tests/            rackunit test suites (raco test tests/)
```

## Requirements

- Racket (developed against v9.3). The GUI additionally needs the platform's
  GTK3/X11 libraries, which any desktop Racket install provides.
- Rust (stable toolchain) for the `rust/` workspace.

## Tests

```
raco test tests/
```

## The Rust twin (rust/)

Same capabilities as the Racket app: egui GUI with the same board interaction,
colors, buttons and live analysis panel, and the same CLI subcommands. The
core logic (MRV solver with propagation, uniqueness-checked generator,
complexity metrics) lives in the `sudoku-core` library crate and is covered by
unit tests plus golden parity fixtures measured from the Racket
implementation (`rust/core/tests/fixtures/parity.json`) — two languages,
one truth.

Build, run, test:

```
cd rust
cargo build            # builds sudoku-core + the sudoku app
cargo run -- ui        # GUI (on Wayland: env -u LD_LIBRARY_PATH cargo run -- ui)
cargo run -- generate hard
cargo run -- solve  "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
cargo run -- analyze "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
cargo test --workspace # fmt/clippy/test also run in CI (.github/workflows/rust.yml)
```

Deliberate divergences from the Racket version (documented in the parity
fixtures): the Rust solution counter is correct where Racket's `for/and`
short-circuit undercounts (so `unique` is `true` for puzzles like
"hard-17"/"escargot" where Racket reports `#f`), and CLI booleans print as
`true`/`false` instead of `#t`/`#f`.
