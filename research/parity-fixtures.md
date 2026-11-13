# Parity fixtures extracted from the Racket test suite and CLI (ticket #11)

**Source of truth:** `jorisgillis/puzzle-explorer` @ `449a8cfb9c5740266e40cec0a4f3a26eeb99af9d` (branch `main`),
Racket v8.10 [cs] (developed against v9.3 per README). All values below were produced by actually running
`raco test tests/` (27 tests passed) and `racket sudoku/main.rkt …` plus small probe scripts that call
`analyze`, `solve/stats`, `count-solutions`, `propagate`, `candidates-of` directly on the same inputs.

**Determinism note (important for Rust parity tests):** `solve`, `analyze`, `count-solutions`, `propagate`
are fully deterministic (MRV picks the first minimal-candidate cell, candidates tried in ascending order).
`generate` is **randomized** (`shuffle` in `generator.rkt`), so it yields *invariants*, not golden outputs.

---

## 1. Proposed JSON schema (for e.g. `rust/core/tests/fixtures/parity.json`)

```json
{
  "version": 1,
  "source": {
    "repo": "jorisgillis/puzzle-explorer",
    "commit": "449a8cfb9c5740266e40cec0a4f3a26eeb99af9d",
    "racket": "v8.10 [cs]",
    "captured_by": "raco test tests/ + racket sudoku/main.rkt + direct analyze/solve-stats probes"
  },
  "grid_facts": {
    "cell_count": 81,
    "peers_of_cell_0": [1,2,3,4,5,6,7,8,9,10,11,18,19,20,27,36,45,54,63,72],
    "row_of": {"0": 0},
    "col_of": {"80": 8},
    "box_of": {"0": 0, "40": 4, "80": 8},
    "pretty_string_line_count": 13,
    "accepted_chars": "0-9 and . (dot normalizes to 0)"
  },
  "puzzles": [
    {
      "id": "readme",
      "input": "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
      "aliases": ["53.070000… (dots variant, identical results)"],
      "valid": true,
      "solve": {
        "solution": "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        "nodes": 52,
        "max_depth": 51,
        "mutates_input": false
      },
      "count_solutions": {"cap2": 1, "cap3": 1},
      "propagate_filled": 51,
      "candidates_of_cell_2": [1, 2, 4],
      "analysis": {
        "givens": 30, "empty_cells": 51, "total_candidates": 153,
        "candidate_entropy_bits": 75.17318120275553,
        "propagated": 51, "solver_nodes": 52, "max_depth": 51,
        "unique": true, "solvable": true, "difficulty": 26
      }
    }
  ],
  "error_cases": [
    {
      "id": "wrong-length",
      "cli": ["solve", "123"],
      "exit_code": 1,
      "stderr_contains": "string->grid: contract violation",
      "stderr_contains_expected": "81-character string of digits 0-9",
      "stderr_contains_given": "\"123\""
    }
  ],
  "cli_contract": {
    "usage_stdout": "Usage:\n  racket main.rkt ui                        launch the GUI\n  racket main.rkt generate <easy|medium|hard>\n  racket main.rkt solve <81-char-string>\n  racket main.rkt analyze <81-char-string>",
    "no_args": "launches the GUI (racket/gui; fails headless without a display)",
    "unknown_or_missing_args": "prints usage to stdout, exit code 0"
  },
  "generator": {
    "difficulty_givens_targets": {"easy": 40, "medium": 32, "hard": 26},
    "randomized": true,
    "invariants": [
      "puzzle has exactly one solution (count-solutions cap 2 == 1)",
      "solve(puzzle) == solution grid",
      "givens(puzzle) <= difficulty-givens(difficulty)",
      "givens(hard) < givens(easy)",
      "puzzle is a subset of its solution grid",
      "generate-puzzle 'impossible raises exn:fail?"
    ]
  },
  "known_quirks": [
    "count-solutions uses for/and, which aborts the whole value loop on the first dead-end branch: it can return 0 for solvable unique puzzles (see 'hard-17' and 'escargot' below). analyze's unique? inherits this (reports false for unique puzzles whose first MRV branch dead-ends). Decide explicitly whether the Rust port replicates or fixes this."
  ]
}
```

Notes on field types for the Rust side: `candidate_entropy_bits` is an `f64` (Racket `exact->inexact` of `sum log2(n)`);
the CLI prints it with 2 decimals (`~r #:precision 2`), so tests should compare with an epsilon (e.g. 1e-9)
against the full-precision value, or compare the rounded CLI string. All other metrics are integers/bools.

---

## 2. Concrete golden entries (measured, deterministic)

### 2.1 `readme` — README example puzzle (also the fixture in `tests/solver-test.rkt` and `tests/complexity-test.rkt`)

- input: `530070000600195000098000060800060003400803001700020006060000280000419005000080079`
- dots variant `53.070000…079` (cell 2 as `.`) produces **byte-identical** analysis/solve results (dots normalize to 0).
- solve → `534678912672195348198342567859761423426853791713924856961537284287419635345286179`, nodes = **52**, max-depth = **51**, input not mutated.
- count-solutions: cap 2 → **1**, cap 3 → **1**.
- propagate on a copy fills **51** cells (all of them — solvable by singles alone).
- candidates-of cell 2 = **(1 2 4)** (asserted in solver-test.rkt).
- analyze: givens **30**, empty-cells **51**, total-candidates **153**, candidate-entropy **75.17318120275553** bits (CLI prints `75.17`), propagated **51**, solver-nodes **52**, max-depth **51**, unique? **#t**, solvable? **#t**, difficulty **26**.
- CLI `solve` prints the 13-line pretty grid (exact text captured below in §4); CLI `analyze` prints the 11-line `Complexity analysis` report (§3).

### 2.2 `solved` — the complete grid from `tests/grid-test.rkt`

- input: `534678912672195348198342567859761423426853791713924856961537284287419635345286179`
- valid? #t, complete? #t; grid->string round-trips to the same string.
- analyze: givens **81**, empty **0**, total-candidates **0**, entropy **0.0**, propagated **0**, nodes **1**, max-depth **0**, unique? **#t**, solvable? **#t**, difficulty **0**.
- pretty-string has 13 lines (asserted in grid-test.rkt).

### 2.3 `empty` — 81 zeros

- analyze: givens **0**, empty **81**, total-candidates **729**, entropy **256.7639251168277** (CLI `256.76`), propagated **0**, nodes **82**, max-depth **81**, unique? **#f**, solvable? **#t**, difficulty **41**.
- solve → `123456789456789123789123456231674895875912364694538217317265948542897631968341572` (deterministic first solution), nodes 82, depth 81.
- count-solutions: cap 2 → **2**, cap 3 → **3** (asserted in solver-test.rkt).
- candidates-of cell 2 = (1..9).

### 2.4 `unsolvable-11` — `"11" + 79 zeros` (from solver-test.rkt)

- input: `110000000000000000000000000000000000000000000000000000000000000000000000000000000`
- valid? **#f** (two 1s in row 0). CLI `solve` prints `No solution.` on **stdout, exit code 0**.
- solve/stats → solution **#f**, nodes **0**, max-depth **0** (search never runs because `valid?` fails).
- count-solutions cap 2 → **0**, cap 3 → **0**.
- analyze: givens **2**, empty **79**, total-candidates **686**, entropy **246.17595007788515** (CLI `246.18`), propagated **0**, nodes **0**, depth **0**, unique? **#f**, solvable? **#f**, difficulty **39**.

### 2.5 `row1-filled` — `123456789` + 72 zeros (multi-solution but solvable)

- analyze: givens **9**, empty **72**, total-candidates **540**, entropy **208.5293250129808**, propagated **0**, nodes **73**, max-depth **72**, unique? **#f**, solvable? **#t**, difficulty **36**.
- solve → `123456789456789123789123456231674895875912364694538217317265948542897631968341572`, count-solutions cap 2 → **2**.

### 2.6 `hard-17` — 21-given search-heavy puzzle (quirk demonstrator)

- input: `800000000003600000070090200050007000000045700000100030001000068008500010090000400`
- valid? #t; solve → `812753649943682175675491283154237896369845721287169534521974368438526917796318452`, nodes **10102**, max-depth **60**.
- analyze: givens **21**, empty **60**, total-candidates **254**, entropy **122.85042932917997**, propagated **0**, nodes **10102**, depth **60**, unique? **#f** (quirk — see below), solvable? **#t**, difficulty **90**.
- **Quirk:** `count-solutions` returns **0** here even though the puzzle is solvable and in fact unique (a corrected counter gives 1). Root cause: `count-solutions` uses `for/and`, so the first dead-end branch aborts the whole candidate loop instead of just that branch.

### 2.7 `escargot` — classic "AI Escargot" (known-unique 23-given puzzle; quirk demonstrator)

- input: `100007090030020008009600500005300900010080002600004000300000010040000007007000300`
- solve → `162857493534129678789643521475312986913586742628794135356478219241935867897261354`, nodes **218**, max-depth **58**.
- analyze: givens **23**, empty **58**, total-candidates **223**, entropy **109.8206233904025**, propagated **1**, nodes **218**, depth **58**, unique? **#f** (quirk: buggy count-solutions returns 0, corrected returns 1), solvable? **#t**, difficulty **31**.

### 2.8 Grid facts from `tests/grid-test.rkt`

- peers-of cell 0 (sorted): `[1,2,3,4,5,6,7,8,9,10,11,18,19,20,27,36,45,54,63,72]`; every cell has exactly 20 distinct peers, none is itself.
- row-of 0 = 0; col-of 80 = 8; box-of 0 = 0, box-of 40 = 4, box-of 80 = 8.
- `string->grid` accepts `.` as empty and round-trips; rejects wrong length with `exn:fail?`.
- valid?/complete?: solved grid → both #t; empty grid → valid #t, complete #f; setting cell 0 of the solved grid to 3 → valid? #f.
- pretty-string = 13 lines when split on `\n`.

### 2.9 Complexity facts from `tests/complexity-test.rkt`

- `difficulty-score 81 0` = **0**; `difficulty-score 17 1000000` = **92** (test asserts 90 ≤ x ≤ 100). Formula: `round(100 * (0.6*min(1, n/5000) + 0.4*(1 - g/81)))`, integer.
- `format-analysis` output contains the substrings `"difficulty score"` and `"solver nodes"`.
- A generated `hard` puzzle has strictly more solver nodes than the readme puzzle (218 < generated-hard in practice; readme = 52).

### 2.10 Generator facts from `tests/generator-test.rkt`

- `difficulty-givens`: easy = **40**, medium = **32**, hard = **26**; any other symbol raises `exn:fail?`
  (CLI: `generate impossible` → stderr `difficulty-givens: contract violation … expected: 'easy, 'medium or 'hard, given: 'impossible`, exit 1).
- generate-complete-grid → complete valid grid; two calls give different grids (randomized).
- generate-puzzle invariants: unique solution, `solve(puzzle) == solution`, `givens(puzzle) <= target`, `givens(hard) < givens(easy)`, puzzle ⊆ solution.
- CLI `generate <d>` prints: 13-line pretty grid, then the 81-char string, then the 11-line analysis — **not golden** (random), use the invariants above.
- Sample observed: easy → 39 givens / difficulty 21; medium and hard samples had ≤ 32 / ≤ 26 givens.

---

## 3. Exact CLI outputs

`racket sudoku/main.rkt analyze 530070000600195000098000060800060003400803001700020006060000280000419005000080079` (stdout, exit 0):

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

`racket sudoku/main.rkt solve 5300…079` (stdout, exit 0) — pretty grid of the solution:

```
+-------+-------+-------+
| 5 3 4 | 6 7 8 | 9 1 2 |
| 6 7 2 | 1 9 5 | 3 4 8 |
| 1 9 8 | 3 4 2 | 5 6 7 |
+-------+-------+-------+
| 8 5 9 | 7 6 1 | 4 2 3 |
| 4 2 6 | 8 5 3 | 7 9 1 |
| 7 1 3 | 9 2 4 | 8 5 6 |
+-------+-------+-------+
| 9 6 1 | 5 3 7 | 2 8 4 |
| 2 8 7 | 4 1 9 | 6 3 5 |
| 3 4 5 | 2 8 6 | 1 7 9 |
+-------+-------+-------+
```

(Pretty-print format: `+-------+-------+-------+` separators, `| a b c | d e f | g h i |` rows, `.` for empty; the *puzzle* pretty-prints with dots, the *solution* with digits.)

`racket sudoku/main.rkt generate easy` prints pretty grid + 81-char string + analysis block (random content; format fixed).

## 4. Invalid-input / error behavior (golden)

| Case | Command | Exit | stdout | stderr (contains) |
|---|---|---|---|---|
| Wrong length | `solve "123"` / `analyze "12"` | 1 | — | `string->grid: contract violation` / `expected: 81-character string of digits 0-9` / `given: "123"` |
| Bad chars (81 letters) | `solve "aaaa…a"` | 1 | — | same contract violation, `given: "aaaa…a"` (chars outside `0-9.` are filtered out first, leaving ≠ 81 → error) |
| Unsolvable (but valid parse) | `solve "11000…0"` | 0 | `No solution.` | — |
| Unknown subcommand / wrong arity | `bogus`, `solve` (no arg), `solve <p> extra` | 0 | usage text (see schema `cli_contract.usage_stdout`) | — |
| Bad difficulty | `generate impossible` | 1 | — | `difficulty-givens: contract violation` / `expected: 'easy, 'medium or 'hard` / `given: 'impossible` |
| No args / `ui` | `racket sudoku/main.rkt` | — | — | launches racket/gui (headless: display error); CLI commands never load the GUI (lazy `dynamic-require`) |

Note the quirk in `string->grid`: disallowed characters are *silently dropped* before the length check, so e.g. an 82-char string with one letter passes. Rust port should replicate: filter to `[0-9.]`, then require length 81, map `.`→0.

---

## 5. How each fixture was produced (provenance)

- `tests/grid-test.rkt`, `tests/solver-test.rkt`, `tests/generator-test.rkt`, `tests/complexity-test.rkt` — read directly; `raco test tests/` → **27 tests passed** on Racket v8.10.
- Golden solve/analysis values — `racket sudoku/main.rkt solve|analyze <puzzle>` plus a probe script calling `analyze`, `solve/stats`, `count-solutions`, `propagate`, `candidates-of` from `sudoku/*.rkt` on the same inputs (full-precision entropy from the probe; 2-decimal values from the CLI).
- Error behavior — actual CLI runs capturing stdout/stderr/exit codes (table in §4).
- Quirk in §2.6/§2.7 — verified by running `count-solutions` (returns 0) against a corrected capped counter (returns 1) on the same grids.

## 6. Recommendations for the Rust side

1. Put the JSON above at `rust/core/tests/fixtures/parity.json`; deserialize with `serde_json` in `tests/parity.rs`.
2. Assert exact equality for all integer/bool fields and the solution strings; compare `candidate_entropy_bits` with `f64` epsilon (1e-9) and optionally assert the CLI-style 2-decimal rounding.
3. Add `pretty` golden strings (13 lines) for the readme puzzle and its solution as separate `.txt` fixtures if the Rust port renders the same UI format.
4. Treat `generate` as property-based (invariants in §2.10), never golden — it is `shuffle`-randomized.
5. Decide explicitly on the `count-solutions`/`unique?` quirk (§2.6, §2.7): either pin the buggy values (`unique: false` for hard-17/escargot) as `known_quirks` entries, or fix it in Rust and record the divergence.
