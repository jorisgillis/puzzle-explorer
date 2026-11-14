# DLX / Algorithm X exact-cover encoding for Sudoku + solver benchmark methodology (ticket #34)

**Ticket:** #34, child of map #33 ("Map: algorithm explainer docs + multi-solver gallery"). Map #33 lists
"Benchmark fixture set: which hard 9x9 instances (17-clue collections?), where shared fixture files live,
generated seeds vs committed files" under *Not yet specified* — §6 answers that directly.
**Date:** 2026-11-14 (UTC). **Repo context:** `jorisgillis/puzzle-explorer` @ `54dcfc03223cc5193c7ee081a91ffbbfd6fc05c0` (branch `main`).
**Method:** every claim below is grounded in a primary source (Knuth's paper and literate programs, the dataset
authors' own pages, canonical benchmark repos) with URLs inline and collected in §7. Sources were fetched and
quoted on 2026-11-14.

---

## 1. Exact cover, Algorithm X, and Dancing Links — the primary source

- Donald E. Knuth, **"Dancing Links"**, arXiv:cs/0011047 (submitted 15 Nov 2000) —
  https://arxiv.org/abs/cs/0011047 (PDF https://arxiv.org/pdf/cs/0011047; text mirror
  https://www.ocf.berkeley.edu/~jchu/publicportal/sudoku/0011047.pdf).
  Abstract: the paper presents "two tricks to accelerate depth-first search algorithms for a class of
  combinatorial puzzle problems. The first trick is to implement each assumption of the search with
  **reversible local operations on doubly linked lists**; by this trick, every step of the search affects the
  data incrementally. The second trick is to add a ghost square that represents the identity of each polyomino
  … equivalent to 0-1 integer programming. … the search can naturally switch between placing a fixed polyomino
  or covering a fixed square at different stages, according to a combined heuristic."
- Algorithm X itself is defined in TAOCP vol. 4, fascicle 5, §7.2.2.1 ("exact cover"); Knuth's reference
  implementations are the CWEB programs **DANCE / DLX1 / DLX2 / DLX3** (plus **SUDOKU-DLX**, **SUDOKU-GENERAL-DLX**
  change files) listed on https://www-cs-faculty.stanford.edu/~knuth/programs.html.
- **Data structure:** the sparse 0-1 matrix is stored as circular doubly linked lists; every node has
  `L R U D` pointers and belongs to a column header that keeps a size field `S`. `cover(c)` removes header `c`
  from the header list (`L[R[c]]←L[c]`, `R[L[c]]←R[c]`), then for each row `i` in column `c`, for each `j≠c` in
  row `i`: `S[j]←S[j]−1`, `U[D[j]]←U[j]`, `D[U[j]]←D[j]`; `uncover(c)` re-links in exact reverse order.
  Each individual link surgery is **O(1)** — no scanning, no allocation; a cover/uncover pair costs time
  proportional to the number of links touched (Knuth, arXiv:cs/0011047 §2–3).
- **Machine-independent cost metric:** Knuth: "the running time … remains essentially proportional to the
  **number of updates**, which is the number of times the links do their dance. Therefore I prefer to measure
  the performance of algorithm DLX by **counting the number of updates, not … the number of elapsed seconds**"
  ("updates" = link removals/restorations; his 500 MHz Pentium III did 5.94 mega-updates/s).
- **Column-choice heuristic (MRV):** "Or if we want to minimize the branching factor, we could set s ← ∞ and
  then for each j ← R[h], R[R[h]], …, while j ≠ h, if S[j] < s set c ← j and s ← S[j]. Then **c is a column with
  the smallest number of 1s**. (The S fields are not needed unless we want to minimize branching in this way.)"
  — i.e. branch on the minimum-`S` column ("shortest column"). The paper's worked example: 28 updates with the
  S heuristic on that instance.

## 2. Canonical exact-cover encoding of 9×9 Sudoku

- **Rows = candidate placements (r, c, d): 9·9·9 = 729 rows.**
- **Columns = four constraint groups × 81 = 324 primary columns:**
  1. cell: `(r,c)` must be filled — constraint `rc(r,c)`
  2. row×digit: digit `d` appears once in row `r` — `rn(r,d)`
  3. column×digit: digit `d` appears once in column `c` — `cn(c,d)`
  4. box×digit: digit `d` appears once in box `b(r,c)` — `bn(b,d)`
- Each placement row `(r,c,d)` has **exactly four 1s**, one per group: in columns `rc(r,c)`, `rn(r,d)`, `cn(c,d)`,
  `bn(b(r,c),d)`. Total 729·4 = 2916 ones in a 729×324 sparse matrix (4 of 324 entries per row).
- A completed grid = a set of **81 rows** (one per cell) covering **each of the 324 columns exactly once** ⇒ an
  exact cover. Givens are pre-placed: insert row `(r,c,g)` and cover its four columns, which deletes all
  conflicting placements.
- Primary sources for exactly this encoding:
  - Knuth's own literate programs: https://www-cs-faculty.stanford.edu/~knuth/programs/sudoku.w ("We want an
    exact cover of those rows, columns, and boxes"; feeds a puzzle into `DANCE`; emits one line per placement
    naming its row/column/box constraints) and `sudoku-dlx.w` (SUDOKU-DLX), listed on
    https://www-cs-faculty.stanford.edu/~knuth/programs.html.
  - Ali Assaf's canonical 30-line DLX sudoku encoder, which builds exactly the four groups:
    `X = [("rc", rc) …] + [("rn", rn) …] + [("cn", cn) …] + [("bn", bn) …]` —
    https://www.cs.mcgill.ca/~aassaf9/python/algorithm_x.html and the code
    https://www.cs.mcgill.ca/~aassaf9/python/sudoku.txt (the 324/729 arithmetic is spelled out in the write-up:
    81+81+81+81 = 324 columns, 729 candidate rows).
- All 324 columns are **primary** (mandatory) for Sudoku, so Knuth's secondary/optional "ghost" columns are not
  needed here (they exist in DLX for optional constraints; see arXiv:cs/0011047).

## 3. Standard optimizations (as used in practice)

1. **MRV column choice** via the `S` size fields — branch on the min-`S` column (quote in §1). For Sudoku this
   is MRV over constraints: an empty cell with the fewest remaining candidates is the smallest column.
2. **O(1) cover/uncover** per link update via the circular `L/R/U/D` lists (§1); DLX3 ("dancing cells", Nov 2020,
   on the programs page) replaces links with packed arrays — same complexity, better constants.
3. **Measure updates, not seconds** — report link-update counts alongside wall time (Knuth's recommendation, §1).
4. Optional: secondary columns / color-controlled exact cover (TAOCP 7.2.2.1) — not needed for plain Sudoku.

## 4. DLX doubles as a solution counter → uniqueness checks

- Algorithm X **enumerates all exact covers** if you don't stop at the first solution; every search-tree leaf is
  one solution. The DLX paper cites programs that "enumerate all solutions" (arXiv:cs/0011047).
- Knuth's `sudoku.w` exists precisely for this: Mike Spivey's Feb-2005 programming contest asked for sudoku
  puzzle generators, and Knuth's program feeds each puzzle into `DANCE` **"in order to deduce all solutions"**
  (https://www-cs-faculty.stanford.edu/~knuth/programs/sudoku.w).
- **Uniqueness check = count solutions with early exit at 2**: 0 ⇒ unsolvable, 1 ⇒ unique, ≥2 ⇒ not unique.
  This is exactly what this repo's generator already relies on (capped `count-solutions` with cap 2; see
  `research/parity-fixtures.md` on branch `research/parity-fixtures`).
- Canonical enumeration results that such counters reproduce at scale:
  - Felgenhauer & Jarvis, "Enumerating possible Sudoku grids": **6,670,903,752,021,072,936,960 ≈ 6.671×10²¹**
    completed grids — http://www.afjarvis.org.uk/sudoku/bertram.html, paper
    http://www.afjarvis.org.uk/sudoku/sudoku.pdf.
  - McGuire, Tugemann, Civario, "There is no 16-Clue Sudoku" (minimum clue count is 17; the exhaustive proof
    hinges on every 16-clue candidate having 0 or ≥2 solutions — multi-solution detection) —
    https://arxiv.org/abs/1201.0749.

## 5. Benchmark methodology used by credible Sudoku-solver comparisons

### 5.1 Fixture sets
- **Gordon Royle's 17-clue collection** — his page: "Currently I have a collection of **49151 distinct Sudoku
  configurations with 17 entries**. Download the file containing all of them." —
  http://mapleta.maths.uwa.edu.au/~gordon/sudokumin.php (legacy URL
  http://staffhome.ecm.uwa.edu.au/~00013890/sudokumin.php). This collection is now used as *the* hard-instance
  benchmark: "the **Royle 17-clue Sudoku benchmark**" in arXiv:2606.06518 (DiBS), https://arxiv.org/abs/2606.06518.
- **Norvig's sets** — 50 easy + **95 hard** (`top95.txt`, http://norvig.com/top95.txt) + `hardest.txt` + a
  1,000,000-random-puzzle file; his essay reports e.g. "Solved 50 of 50 easy puzzles (avg 0.01 secs (71 Hz), max
  0.02 secs). Solved **95 of 95 hard puzzles** (avg 0.04 secs (24 Hz)…)" and a timing table over the million —
  https://www.norvig.com/sudoku.html. It also features Arto Inkala's 2006 puzzle (widely known as **AI
  Escargot**), described there as "the most difficult sudoku-puzzle known so far".
- **Kaggle 1M dataset** (bryanpark, puzzles + solutions) —
  https://www.kaggle.com/datasets/bryanpark/sudoku-sudoku-puzzles-1-million — used as a large *representative*
  sample in benchmark suites (e.g. tdoku).
- **tdoku's benchmark suite dataset zoo** — https://github.com/t-dillon/tdoku/tree/master/benchmarks — Kaggle 1M,
  EnjoySudoku forum hardest sets (`forum_hardest_1905_…`), `gen_puzzles.zip`, and generated sets; the README
  explicitly warns that some sources "often contain puzzles **selected for special properties**" and that
  generated puzzles are "extremely easy … [and] may favor one solver over another".
- **Comparative benchmark repos:** `th-tsai/sudoku-solver` (five solvers — classic backtracking, MRV, constraint
  propagation, **Dancing Links**, ILP — over **400 puzzles = 100 per difficulty tier** drawn from a
  Kaggle-rated dataset; **each solver runs 5 times per puzzle, median solve time is recorded; every solution is
  validated against the known ground-truth answer; results exported to CSV**) —
  https://github.com/th-tsai/sudoku-solver. `michaelkrisper/sudoku` (six algorithms — backtracking, MRV, Norvig,
  **DLX**, rule-based, multithreaded — in Python/JS/Go/Rust/C++/x86-64 asm behind one benchmark harness) —
  https://github.com/michaelkrisper/sudoku.

### 5.2 Metrics
- **Wall time** per puzzle plus avg/max (Norvig's "avg 0.04 secs (24 Hz), max …" format, §5.1).
- **Machine-independent work counters**: for DLX, link **updates** (Knuth's preferred metric, §1); for
  backtracking solvers, **search nodes / backtracks / max depth**. The DiBS paper evaluates on the Royle
  17-clue benchmark and reports **node counts, backtracks, and long-tail percentiles** (avg nodes −42.0%,
  p95 time −60.5% vs MRV+FC+LCV+Degree) — https://arxiv.org/abs/2606.06518. This repo's solver already exposes
  `nodes` + `max_depth` via `solve/stats` (see `research/parity-fixtures.md`, branch `research/parity-fixtures`),
  so the planned gallery can report **wall time + nodes + max depth** uniformly across Racket and Rust.
- **Correctness gate**: validate every produced solution against the known answer before trusting timings
  (th-tsai does this).

### 5.3 Repetitions and warmup
- Run each puzzle **multiple times and take the median** (th-tsai: 5 runs/puzzle, median) — robust to outliers.
- **Warm up before measuring.** Criterion.rs (the standard Rust benchmarking harness — directly relevant to the
  planned Rust gallery) runs a warm-up phase first because "the OS, CPU and JIT [need] time to adapt to the
  benchmark", then adaptively increases sample-iteration counts and reports confidence intervals and IQR-based
  outlier warnings — https://bheisler.github.io/criterion.rs/book/analysis.html.
- Pair every wall-time number with a work counter (updates/nodes), because wall time is hardware-dependent
  (Knuth's mega-updates point, §1).

### 5.4 Known pitfalls
- **Puzzle-set bias.** Hardest-lists (Norvig's top95, EnjoySudoku forum hardest) are selected *because* they are
  hard for some solver class — fine as stress tests, misleading for "typical performance" claims. Samplers can
  **over-sample low-clue (minimal) puzzles** relative to the natural distribution; DiBS therefore samples "with
  uniform probability (conditional on the number of clues) from the set of all minimal Sudoku puzzles"
  (https://arxiv.org/html/2606.06518), and tdoku flags property-selected/generated sets as solver-favoring
  (benchmarks README, §5.1). → Use **≥2 complementary sets**: a hard/minimal-clue set (Royle 17-clue) *and* a
  representative random set (Kaggle 1M / Norvig's million), plus named extremes (AI Escargot; Norvig's
  hardest-for-brute-force grid2).
- **Cache / JIT / allocator warm-up.** First runs are systematically slower (Criterion's warm-up rationale,
  §5.3): do untimed warmup iterations and discard them.
- **Timing noise.** Median of N≥5 runs; report max/long-tail percentiles (DiBS uses p95/p99) because a few
  puzzles dominate the tail; use outlier detection (Criterion's IQR method).
- **Hardware dependence of wall time.** Always co-report nodes/updates/max depth so results transfer across
  machines (Knuth, §1).
- **Correctness before speed.** Validate solutions against known answers (th-tsai) so a fast-but-wrong solver
  never wins.

## 6. Recommendation for map #33 (the "Benchmark fixture set" item under *Not yet specified*)

- **Committed fixture files** (deterministic, byte-identical for Racket + Rust — *not* generated seeds), one
  81-char puzzle per line, with a source-URL comment header:
  - `tests/fixtures/bench-top95.txt` — Norvig's 95 hard puzzles (http://norvig.com/top95.txt);
  - `tests/fixtures/bench-royle17.txt` — a fixed sample of Royle's 49,151 17-clue puzzles (e.g. first N lines of
    the downloadable file, or the whole file committed), sourced from
    http://mapleta.maths.uwa.edu.au/~gordon/sudokumin.php;
  - `tests/fixtures/bench-extremes.txt` — AI Escargot (Inkala 2006), Norvig's grid2 "hardest for brute force",
    and the README puzzle;
  - optionally a representative random sample from the Kaggle 1M dataset.
- **Harness per language:** for each fixture puzzle: 1 untimed warmup solve, then N=5 timed runs; report
  **median wall time + search nodes + max depth** (already available from `solve/stats`); the DLX solver
  additionally reports **link updates**. Validate each solution. Results table lands in `docs/solvers.md`.

## 7. Sources (all URLs)

- Knuth, "Dancing Links", arXiv:cs/0011047 — https://arxiv.org/abs/cs/0011047 (PDF https://arxiv.org/pdf/cs/0011047; text mirror https://www.ocf.berkeley.edu/~jchu/publicportal/sudoku/0011047.pdf)
- Knuth programs page (DANCE, DLX1/2/3, SUDOKU-DLX, sudoku.w) — https://www-cs-faculty.stanford.edu/~knuth/programs.html ; sudoku.w — https://www-cs-faculty.stanford.edu/~knuth/programs/sudoku.w ; sudoku-dlx.w — https://www-cs-faculty.stanford.edu/~knuth/programs/sudoku-dlx.w
- Ali Assaf, "Algorithm X in 30 lines!" + sudoku encoder — https://www.cs.mcgill.ca/~aassaf9/python/algorithm_x.html , https://www.cs.mcgill.ca/~aassaf9/python/sudoku.txt
- Gordon Royle, 17-clue collection (49,151 puzzles) — http://mapleta.maths.uwa.edu.au/~gordon/sudokumin.php (legacy http://staffhome.ecm.uwa.edu.au/~00013890/sudokumin.php)
- McGuire, Tugemann, Civario, "There is no 16-Clue Sudoku" — https://arxiv.org/abs/1201.0749
- Felgenhauer & Jarvis, "Enumerating possible Sudoku grids" (6,670,903,752,021,072,936,960 grids) — http://www.afjarvis.org.uk/sudoku/bertram.html , http://www.afjarvis.org.uk/sudoku/sudoku.pdf
- Peter Norvig, "Solving Every Sudoku Puzzle" (easy50/top95/hardest/1M fixtures + timing methodology) — https://www.norvig.com/sudoku.html , http://norvig.com/top95.txt
- Kaggle 1M sudoku dataset (bryanpark) — https://www.kaggle.com/datasets/bryanpark/sudoku-sudoku-puzzles-1-million
- tdoku benchmark suite (dataset zoo + bias warnings) — https://github.com/t-dillon/tdoku/tree/master/benchmarks
- th-tsai/sudoku-solver (5 solvers, 400 puzzles, 5 runs median, validation, CSV) — https://github.com/th-tsai/sudoku-solver
- michaelkrisper/sudoku (6 algorithms × 7 languages benchmark) — https://github.com/michaelkrisper/sudoku
- DiBS, arXiv:2606.06518 (Royle 17-clue benchmark; nodes/backtracks/p95-p99; uniform sampling note) — https://arxiv.org/abs/2606.06518 , https://arxiv.org/html/2606.06518
- Criterion.rs analysis docs (warm-up phase, adaptive sampling, confidence intervals, IQR outliers) — https://bheisler.github.io/criterion.rs/book/analysis.html
- Repo-internal: `research/parity-fixtures.md` on branch `research/parity-fixtures` (nodes/max_depth/`count-solutions` cap-2 invariants of the current solver)
