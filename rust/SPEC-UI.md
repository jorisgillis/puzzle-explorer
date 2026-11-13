# UI behavior spec — egui app (`sudoku` bin crate)

Source of truth: `sudoku/ui.rkt` (Racket GUI). This spec pins the exact
behavior the egui app in `rust/app` must replicate. Ticket: #16.
Rendering is a custom-painted board widget (decision #7) — no egui
`Grid`/table widgets; we paint cells ourselves inside one `CentralPanel`
side area, as egui in-window panels only (popup viewports hang on Wayland,
see #9).

## Window and layout

- Window title: `Puzzle Explorer - Sudoku`.
- Layout top-to-bottom inside the main window (all in-window, no popups):
  1. **Board** (centered), square of `board-size` px.
  2. **Controls row** (centered, horizontal): buttons
     `New Easy`, `New Medium`, `New Hard`, `Solve`, `Hint`, `Check`,
     `Analyze`, then a `Pencils` checkbox.
  3. **Status line**: single-line label, initial text
     `Generate a puzzle to begin`.
  4. **Analysis panel**: read-only multi-line text, initially **hidden**;
     toggled by the `Analyze` button. Min size ≈ 460x132 px.
- Board metrics (from `ui.rkt`): `cell-size = 48`, `margin = 12`,
  `board-size = 9*48 + 2*12 = 456` px. Racket frame: width `board-size+40`,
  height `board-size+220` — egui: pick a comfortable initial window
  (e.g. 520x760) with the board centered; exact window size is not part of
  parity.
- **Startup**: immediately generate and show a **medium** puzzle
  (Racket calls `new-puzzle 'medium` before showing the frame).

## Colors and fonts

| Role | RGB | Notes |
|---|---|---|
| given digit | (20, 40, 160) | big font, ~22px bold, centered in cell |
| user entry | (30, 30, 30) | big font, centered |
| conflict entry | (200, 30, 30) | user entry whose value duplicates a peer in `current` |
| peer highlight | (255, 244, 180) | fill of the 20 peers of the selected cell |
| selected cell | (255, 224, 102) | drawn on top of peer highlight |
| pencil mark | (120, 120, 120) | small font ~10px, 3x3 mini-grid inside the cell |
| thick grid line | black, 2px | every 3rd line (box boundaries) + outer border |
| thin grid line | (160, 160, 160), 1px | all other lines |
| background | white | |

## Board painting (order matters)

1. Clear to white.
2. **Highlights** (under the lines): if a cell is selected, fill all 20 of
   its peers with peer-highlight, then fill the selected cell itself with
   selected color.
3. **Grid lines**: 10 vertical + 10 horizontal lines at
   `pos = margin + k*cell-size` for k in 0..9; thick/black when `k % 3 == 0`,
   else thin/gray. Then a 2px black rectangle around the whole board
   (`margin, margin` to `margin + 9*cell-size`).
4. **Cells**, for each index `i` in 0..80 (`x = margin + col*48`,
   `y = margin + row*48`):
   - `puzzle[i] > 0` (given): big bold digit, given color, centered.
   - else `current[i] > 0` (user entry): conflict = any peer `p != i` with
     `current[p] == current[i] > 0`; draw digit in error color if conflict,
     else user color; centered.
   - else if pencils shown: draw `candidates(current, i)` (ascending 1..9)
     as pencil marks — small gray font, 3x3 mini-grid inside the cell
     (`mini = cell-size/3 = 16` px); the k-th candidate of the list goes at
     row-major mini-position k (so a full candidate set reads 1 top-left …
     9 bottom-right, a subset compacts left), centered in its mini-cell.

Digit centering: center the text extent within the cell (both axes).

## Interaction

State: `puzzle` (givens, immutable during play), `current` (givens + user
entries), `selected: Option<usize>`, `show-pencils: bool`, `solution:
Option<Grid>`, status string.

- **Click**: on mouse press, `c = (x - margin) / 48`, `r = (y - margin) /
  48`; if `0 <= r,c <= 8` then `selected = r*9 + c`. Clicks in the margin or
  outside the grid do nothing.
- **Digit keys 1-9**: only when a cell is selected **and** that cell is not
  a given (`puzzle[selected] == 0`): set `current[selected] = v`; fire
  on-change; redraw.
- **0 / Backspace / Delete**: same guard (not a given): clear
  `current[selected] = 0`; on-change; redraw.
- **Arrow keys**: move selection one cell with clamping at edges
  (`up`: r-1 min 0, `down`: r+1 max 8, etc.). If nothing is selected yet,
  arrows start from cell 0. Arrows do not fire on-change.
- Givens can never be edited or cleared.

## Buttons and checkbox

- **New Easy / Medium / Hard**: `generate_puzzle(difficulty)` → store
  `solution`, reset `current = puzzle` copy, `selected = None`; status:
  `New <difficulty> puzzle (<n> givens)` (lowercase difficulty word, n =
  count of non-zero cells in puzzle).
- **Solve**: if `solution` exists: `current = solution` copy; status
  `Solved.`
- **Hint**: if `solution` exists: target = first cell where
  `current[i] == 0`, else first cell where `current[i] != solution[i]`; if
  target exists and is not a given, set `current[target] =
  solution[target]`; status: `Hint: cell <target+1> = <value>` (1-based cell
  number).
- **Check**: inspect `current`: if `!is_valid(current)` → `There are
  conflicting entries.`; else if `solution` exists and `current ==
  solution` → `Solved correctly!`; else → `No conflicts so far.`
- **Analyze**: toggle visibility of the analysis panel; when becoming
  visible, recompute it (see below).
- **Pencils** checkbox: toggle `show-pencils`; redraw.

## Live analysis panel

Recomputed **on every board change** (any mutation of `current`, including
Solve/Hint) whenever the panel is visible. Content:

```
Live analysis of the current board (<state>)
<format_analysis(analyze(current))>
```

where `<state>` is:

- `conflicting entries present` — `!is_valid(current)`
- `solved correctly` — `solution` exists and `current == solution`
- `no conflicts so far` — otherwise

`format_analysis` is the same report the CLI `analyze` prints (11 lines,
see `rust/core/src/complexity.rs`); booleans render as `true`/`false`
(deliberate divergence from Racket's `#t`/`#f`, decided in #15).

## egui-specific constraints (from research #9)

- eframe 0.37, in-window panels only — no popup dialogs/windows (they hang
  on this Wayland session).
- Custom paint the board with `egui::Painter` on a `Frame`/`Sense::click`
  area of `board-size` x `board-size`; keyboard input via
  `ctx.input(|i| i.key_pressed(...))` / `i.events` for text chars.
- Redraw via `ctx.request_repaint()` after any state mutation (or rely on
  egui's continuous mode only while animating — not needed here).
