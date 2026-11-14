# Research: primary citations for Sudoku complexity and generation results

Wayfinder ticket #35 (child of map #33). Question: exact, citable primary
sources (title, authors, venue, year, URL/DOI, one-line statement of each
result) for the five claims used by the public explainer's "Further reading"
section (`docs/complexity.md`, `docs/generation.md`).

- Branch: `research/complexity-citations` (throwaway; no PR).
- Verified 2026-11-14 against publisher/index pages (IEICE GLOBALS, dblp,
  ScienceDirect/ASU Pure, Taylor & Francis, arXiv, Cambridge Core, arXiv).
- Two corrections to the ticket's shorthand: Colbourn 1984 is in **Discrete
  Applied Mathematics** 8(1):25–30 (DOI `10.1016/0166-218X(84)90075-1`), not
  SIAM J. Alg. Discrete Methods; McGuire–Tugemann–Civario's journal version is
  **Experimental Mathematics** 23(2):190–217 (2014), DOI
  `10.1080/10586458.2013.870056` (arXiv:1201.0749 is the preprint).

## 1. Generalized n²×n² Sudoku is NP-complete (decision problem)

- **Takayuki Yato and Takahiro Seta**, "Complexity and Completeness of Finding
  Another Solution and Its Application to Puzzles", *IEICE Transactions on
  Fundamentals of Electronics, Communications and Computer Sciences*, vol.
  E86-A, no. 5, pp. 1052–1060, May 2003.
  DOI: `10.1587/e86-a_5_1052` —
  https://globals.ieice.org/en_transactions/fundamentals/10.1587/e86-a_5_1052/_p
  (dblp: https://dblp.org/pid/27/2801.html)
- One-line result: SAT is shown ASP-complete (via Cook's reduction) and the
  "Another Solution Problem" of **Number Place (= generalized n²×n² Sudoku)**
  is proved ASP-complete, which implies the decision problem "does this
  generalized Sudoku instance have a solution?" is **NP-complete** for the
  family of n²×n² grids with n free to grow (n = 3 is the standard 9×9 board).

### Relation to Colbourn 1984 (partial Latin squares)

- **Charles J. Colbourn**, "The complexity of completing partial Latin
  squares", *Discrete Applied Mathematics*, vol. 8, no. 1, pp. 25–30, April
  1984. DOI: `10.1016/0166-218X(84)90075-1` —
  https://www.sciencedirect.com/science/article/pii/0166218X84900751
- One-line result: completing a partial Latin square (a Sudoku grid is a
  completed Latin square with the extra block constraints, and a partial
  Sudoku is a partial Latin square plus block clues) is **NP-complete**, via a
  reduction from partitioning tripartite graphs into triangles — the
  historical precursor to the Sudoku-specific NP-completeness proof.

## 2. The "another solution" / uniqueness problem is ASP-complete

- Same primary source as #1: **Yato & Seta 2003** (IEICE E86-A(5):1052–1060,
  DOI `10.1587/e86-a_5_1052`, URL above). The ASP (Another Solution Problem)
  of a problem asks, given an instance and one solution, for a *different*
  solution; the paper proves ASP of Number Place (Sudoku), Slither Link, and
  Cross Sum **ASP-complete** under parsimonious reductions.
- One-line result: given a Sudoku puzzle and one of its solutions, finding
  another solution is **ASP-complete** (so its decision version is NP-complete
  — ASP-completeness implies NP-completeness), and asking whether the puzzle
  has a *unique* solution is the complementary question, the basis for the
  uniqueness re-check used in puzzle generation.

## 3. Fixed 9×9 Sudoku is finite: exactly 6,670,903,752,021,072,936,960 grids

- **Bertram Felgenhauer and Frazer Jarvis**, "Mathematics of Sudoku I",
  *The Mathematical Gazette*, vol. 89, no. 516, pp. 499–510, July 2005.
  DOI: `10.1017/S0025556305003767` —
  https://www.cambridge.org/core/journals/mathematical-gazette (journal page)
  Preprint (where the count was first announced, 20 June 2005): "Enumerating
  possible Sudoku grids", http://www.afjarvis.org.uk/sudoku/ (PDF mirror:
  https://www.apprendre-en-ligne.net/blog/docu/sudoku.pdf;
  "Mathematics of Sudoku I" spec:
  http://www.afjarvis.org.uk/sudoku/felgenhauer_jarvis_spec1.pdf)
- One-line result: by a symmetry-reduced brute-force enumeration, the number of
  valid completed 9×9 Sudoku grids is exactly
  **N₀ = 6,670,903,752,021,072,936,960 ≈ 6.67 × 10²¹**.
- Consequence used by the explainer: with the board size fixed at 9×9 the
  instance space is finite, so the decision problem is solvable in constant
  time (look up / precompute); NP-hardness lives only in the unbounded n²×n²
  family (#1).

## 4. 17 is the minimum number of clues for a unique 9×9 puzzle

- **Gary McGuire, Bastian Tugemann, and Gilles Civario**, "There is no
  16-Clue Sudoku: Solving the Sudoku Minimum Number of Clues Problem via
  Hitting Set Enumeration", *Experimental Mathematics*, vol. 23, no. 2,
  pp. 190–217, 2014. DOI: `10.1080/10586458.2013.870056` —
  https://www.tandfonline.com/doi/abs/10.1080/10586458.2013.870056
  Preprint: arXiv:1201.0749 [cs.DS], 1 Jan 2012 — https://arxiv.org/abs/1201.0749
- One-line result: an exhaustive computer search (hitting-set enumeration)
  found **no 16-clue Sudoku puzzle with a unique solution**, proving the
  minimum number of clues is exactly **17** (17-clue puzzles are known).

## 5. Standard generation pipeline (random complete grid + uniqueness-checked clue removal)

Primary citable algorithm source for the exact-cover/backtracking core:

- **Donald E. Knuth**, "Dancing Links", arXiv:cs/0011047 [cs.DS], 15 Nov 2000
  — https://arxiv.org/abs/cs/0011047 (also published in *Millennial
  Perspectives in Computer Science*, Palgrave, 2000, pp. 187–214, DOI
  `10.1145/352220.352233`). One-line result: Algorithm X (depth-first exact
  cover) with the dancing-links trick gives the standard fast backtracking
  engine; Sudoku is an exact cover problem (324 constraints × 729 candidate
  placements), and randomizing the column choice yields a uniformly-random-ish
  complete grid.

In-repo primary implementation of the full pipeline (what our docs describe):

- `sudoku/generator.rkt` and `rust/core/src/generator.rs` —
  `generate-puzzle` / `generate_puzzle`: build a random complete grid, then
  remove clues in shuffled (symmetric-pair) order down to a givens target,
  running **11 attempts and keeping the fewest-givens candidate**, with every
  removal guarded by a uniqueness re-check (`(count-solutions puzzle 2)` must
  stay 1, i.e. stop at the second solution — see `sudoku/generator.rkt:41`).

## Ready-to-paste "Further reading" block (for `docs/complexity.md` / `docs/generation.md`)

> - Yato, T. & Seta, T. (2003). "Complexity and Completeness of Finding Another Solution and Its Application to Puzzles." *IEICE Transactions on Fundamentals*, E86-A(5), 1052–1060. https://doi.org/10.1587/e86-a_5_1052 — generalized n²×n² Sudoku is NP-complete (SAT is reduced to it), and the "another solution" problem of Sudoku is ASP-complete, so uniqueness checking is NP-hard in general.
> - Colbourn, C. J. (1984). "The complexity of completing partial Latin squares." *Discrete Applied Mathematics*, 8(1), 25–30. https://doi.org/10.1016/0166-218X(84)90075-1 — the earlier NP-completeness proof for partial Latin squares, of which Sudoku is a constrained special case.
> - Felgenhauer, B. & Jarvis, F. (2005). "Mathematics of Sudoku I." *The Mathematical Gazette*, 89(516), 499–510. https://doi.org/10.1017/S0025556305003767 — there are exactly 6,670,903,752,021,072,936,960 valid 9×9 grids (≈ 6.67 × 10²¹), so fixed-size 9×9 Sudoku is a finite problem, solvable in constant time.
> - McGuire, G., Tugemann, B. & Civario, G. (2014). "There is no 16-Clue Sudoku: Solving the Sudoku Minimum Number of Clues Problem via Hitting Set Enumeration." *Experimental Mathematics*, 23(2), 190–217. https://doi.org/10.1080/10586458.2013.870056 (arXiv:1201.0749) — exhaustive search proves no 16-clue puzzle has a unique solution; the minimum is 17 clues.
> - Knuth, D. E. (2000). "Dancing Links." arXiv:cs/0011047 — https://arxiv.org/abs/cs/0011047 — Algorithm X / DLX exact-cover backtracking, the engine behind random complete-grid generation; clue removal with a uniqueness re-check after each removal (as in `sudoku/generator.rkt` / `rust/core/src/generator.rs`) then turns a complete grid into a proper puzzle.
