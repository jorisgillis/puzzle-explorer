# Playing

Both implementations ship a native GUI with the same board interaction —
**Racket** (`racket sudoku/main.rkt ui`, racket/gui) and **Rust**
(`cd rust && cargo run -- ui`, egui). The behavior spec for the egui app is
[rust/SPEC-UI.md](../rust/SPEC-UI.md), ported from `sudoku/ui.rkt`.

## The puzzle string

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

## GUI controls and features

- Click a cell, type `1`-`9` to enter a value, `0`/`Delete` to clear; arrow keys move the selection.
- Givens are bold blue, user entries black, conflicting entries red.
- The selected cell's row/column/box is highlighted.
- Buttons: New Easy/Medium/Hard, Solve, Hint, Check, Analyze; a Pencils toggle
  shows candidate numbers in empty cells.
- A Heat toggle overlays each empty cell with a background color encoding how many
  candidates remain for it on the current board (green = 1 … red = 9, error red = 0),
  recomputed live as you play — a mood ring per cell.
- Analyze toggles a live analysis panel with the complexity metrics
  (see [solving.md](solving.md)), recomputed from the current board
  (givens plus your entries) on every change while you play.
