# Research: feasibility and cost of technique detection with our solver

Wayfinder ticket #27 (child of map #25). Question: using the existing repo code,
can we detect that a puzzle is NOT solvable by constraint propagation + singles
alone (i.e. would require an advanced technique), at what cost, and what does that
imply for generate-and-test feasibility of form-constrained generation?

- Branch: `research/technique-detection-cost` (throwaway; no PR).
- Scratch experiment code (clearly scratch, not part of any workspace, safe to
  delete): `research/scratch/technique-cost/` (Rust crate, path-depends on
  `rust/core`, the analyzer was implemented in **Rust** because generation
  benchmarking needs speed and the GUI/CLI app under test is `rust/app`) and
  `research/scratch/technique-cost.rkt` (Racket cross-check for the `sudoku/`
  CLI implementation).
- Machine: Intel Core i7-7700K @ 4.20 GHz, 8 cores, Linux; Rust `--release`
  build; seeds fixed (`StdRng::seed_from_u64(20261115…)`) for reproducibility.
- Repo tests still green on this branch: `raco test tests/` 29 passed;
  `cargo test --workspace` 22 passed.

## Sources (existing repo code used)

- `sudoku/solver.rkt`, `rust/core/src/solver.rs` — `propagate` = fixpoint of
  naked singles + hidden singles; `solve/stats` = MRV backtracking;
  `count-solutions` (capped).
- `sudoku/generator.rkt`, `rust/core/src/generator.rs` — `generate-puzzle` /
  `generate_puzzle`: random complete grid + symmetric-pair clue removal to a
  givens target (easy 40 / medium 32 / hard 26), 11 attempts keeping fewest
  givens, each removal guarded by a uniqueness check.
- `sudoku/complexity.rkt`, `rust/core/src/complexity.rs` — `analyze` already
  reports `propagated` (cells filled by the propagation fixpoint) and
  `empty-cells`; see finding 1.
- `rust/app/src/ui.rs:98` / `rust/app/src/main.rs:58` — the GUI calls
  `generate_random(difficulty)` on button press, so generation latency is
  directly user-facing.

## Finding 1 — "not singles-only" is detectable cheaply, with zero new code

The existing `propagate` (naked + hidden singles fixpoint) followed by a
completeness check answers exactly "solvable by constraint propagation +
singles alone?". Equivalently, the existing `analyze` hash already contains
it: `propagated == empty-cells` iff the propagation fixpoint completes the
grid. Measured detection-only cost (propagate + completeness check, release
build, n=100 puzzles per difficulty):

| target | detect-only | full MRV solve | generate (11-attempt clue removal) |
|---|---|---|---|
| easy 40 | 26 µs/puz (38k puz/s) | 0.043 ms/puz (23k puz/s) | 6.9 ms/puz (144 puz/s) |
| medium 32 | 99 µs/puz (10k puz/s) | 0.079 ms/puz (13k puz/s) | 15.6 ms/puz (64 puz/s) |
| hard 26 | 208 µs/puz (4.8k puz/s) | 0.33 ms/puz (3.0k puz/s, avg 249 nodes) | 68 ms/puz (15 puz/s) |

So detection is ~300x cheaper than generation at hard; analysis is never the
bottleneck — generation (uniqueness-checked clue removal) is.

## Finding 2 — how often are generated puzzles singles-only? (Rust, n=400 per target)

| target (avg givens) | unique | naked-singles-only fixpoint | propagate fixpoint (naked+hidden) = "singles-only" |
|---|---|---|---|
| easy 40 (39.0) | 100% | 94.5% | **99.2%** |
| medium 32 (31.0) | 100% | 37.8% | **90.5%** |
| hard 26 (25.3) | 100% | 2.2% | **55.0%** |

Racket cross-check (`research/scratch/technique-cost.rkt`, unseeded): n=30 →
easy 100%, medium 80%, hard 27%; hard-only n=100 → 29%. The Racket rate at hard
is lower than Rust's 55%; the repo documents a deliberate `count-solutions`
quirk in Racket (`rust/core/tests/fixtures/parity.json` `known_quirks`: Racket's
`for/and` aborts the candidate loop on the first dead-end branch and can return
0 for solvable unique puzzles), which affects clue removal, so the Rust numbers
are the cleaner reference. Either way the qualitative answer agrees: at easy
nearly everything is singles-only; at hard roughly half to two-thirds are.

## Finding 3 — scratch technique analyzer: tiers and throughput

Scratch analyzer in `research/scratch/technique-cost/src/main.rs` (Rust, on top
of `sudoku-core`): bitmask pencil marks; cumulative tiers 0 = naked+hidden
singles, 1 = + locked candidates (pointing/claiming), 2 = + naked pairs,
3 = + hidden pairs, 4 = + X-Wing (row and column). "Minimal tier" = smallest
cumulative tier whose fixpoint completes the grid (5 = needs backtracking
search). Cross-checked on every sample: a tier-t solve's grid must equal
`solve_with_stats` output (asserted in `minimal_tier`); README puzzle sanity =
tier 0.

Minimal-tier distribution of generated puzzles (unique-solution rate 100%
everywhere):

| target | tier0 singles | tier1 +locked | tier2 +naked-pair | tier3 +hidden-pair | tier4 +X-Wing | tier5 needs-search |
|---|---|---|---|---|---|---|
| easy 40 (n=400) | 99.2% | 0% | 0% | 0% | 0% | 0.8% |
| medium 32 (n=400) | 90.5% | 1.8% | 0.8% | 0% | 0% | 7.0% |
| hard 26 (n=400) | 55.0% | 11.2% | 2.5% | 1.8% | **0%** | 29.5% |
| custom 24 (n=150) | 50.0% | 18.0% | 2.7% | 1.3% | 0% | 28.0% |
| custom 22 (n=150) | 40.7% | 16.7% | 6.7% | 0.7% | 0% | 35.3% |
| custom ~20 (n=150, avg 22.7) | 48.7% | 13.3% | 6.7% | 3.3% | 0% | 28.0% |

(Custom targets use the same generator with a custom givens target via
`remove_clues`; note the symmetric-pair removal usually cannot go below
~22-23 givens — the 20 target yielded avg 22.7.)

Technique *usage* (how often each technique fires at all during a tier≤4 solve,
n=200 per difficulty) — X-Wing fires surprisingly often but is never the
minimal requirement:

| difficulty | locked | naked pair | hidden pair | X-Wing | solved without search |
|---|---|---|---|---|---|
| easy | 23.0% | 0.5% | 3.0% | 0.5% | 99.5% |
| medium | 94.5% | 55.0% | 49.0% | 21.5% | 94.5% |
| hard | 100% | 76.5% | 79.5% | 48.5% | 73.5% |

Throughput, generate + full technique analysis (release build, n=100 per
difficulty):

| difficulty | generate | generate+analyze | analyze-only |
|---|---|---|---|
| easy | 144 puz/s (6.9 ms) | 141 puz/s (7.1 ms) | 8,036 puz/s (0.12 ms) |
| medium | 64 puz/s (15.6 ms) | 63 puz/s (15.9 ms) | 3,893 puz/s (0.26 ms) |
| hard | 15 puz/s (68 ms) | 14 puz/s (70 ms) | 801 puz/s (1.25 ms) |

Racket CLI (`racket research/scratch/technique-cost.rkt`, generate + propagate
+ uniqueness + solve/stats, n=30/difficulty): ~0.8-1.9 s/puz easy,
~1.1 s/puz medium, ~1.9-2.5 s/puz hard (n=100) — i.e. roughly **1 puzzle/s**, 15-70x
slower than the Rust app at generation.

## Finding 4 — expected generate-and-test attempts for "requires technique T"

Expected attempts = 1 / frequency of unique puzzles whose minimal tier is
exactly T (hard target 26, Rust; attempts run n=300, generation 82.9 ms/puz):

| form | frequency (hard) | expected attempts | expected wall time (Rust GUI) |
|---|---|---|---|
| singles-only (tier 0) | 53-55% | ~2 | ~0.2 s |
| requires search / advanced (tier 5) | 30-32% | ~3 | ~0.3 s |
| requires hidden pair (tier 3) | 1.7-1.8% | ~55-60 | **~5 s** |
| requires X-Wing (tier 4) | 0/850 combined (400 hard + 450 at 20-24 givens; 95% upper bound ≈ 0.4%) | > ~250 | **> ~25 s, realistically minutes+** |

## Verdict

1. **Detection: yes, and essentially free.** "Not solvable by propagation +
   singles alone" = propagation fixpoint does not complete the grid. It is
   already computable from the existing `analyze` output (`propagated ==
   empty-cells`) or by running `propagate` + completeness check, at 26-208 µs
   per puzzle (release) — ~300x cheaper than generating the puzzle, and
   cheaper than a full MRV solve. In the Racket CLI the same check is a
   `propagate` + `complete?` on a grid copy.
2. **Analysis cost is not the bottleneck.** Generate+analyze ≈ generate alone
   (analysis is 1-2% of generation cost at hard: 68 ms vs 1.25 ms). The cost of
   generate-and-test is dominated by the 11-attempt uniqueness-checked clue
   removal: 144/64/15 puz/s (easy/medium/hard) in Rust, ~1 puz/s in the Racket
   CLI.
3. **Generate-and-test is feasible at interactive CLI/GUI latency for common
   forms**: singles-only (~2 attempts, <0.2 s in the GUI), "requires advanced
   technique / search" (~3 attempts), and even "requires hidden pair" at the
   hard target (~55-60 attempts ≈ 5 s in the Rust GUI; ~2-3 min in the Racket
   CLI — acceptable for a one-off but not for rapid iteration).
4. **Not feasible for rare specific techniques at the existing givens
   targets**: "requires X-Wing" was never observed as the minimal tier in 850
   generated puzzles (including down to ~22-23 givens, which is about as low as
   the current symmetric-pair generator gets), so generate-and-test would need
   > ~250 attempts (>25 s in the GUI, realistically minutes) — and that is a
   lower bound on an event with p < 0.4%. X-Wing *fires* in ~48% of hard puzzles
   but is never the *minimal* requirement: whenever it applies, lower tiers
   already solve or search is needed anyway. For form constraints like
   "requires X-Wing", constructive generation (build the puzzle around the
   technique) or a much deeper clue-removal target would be needed; that is a
   design input for map #25, not something generate-and-test can deliver at
   interactive latency with the current generator.

## Reproduce

```
cd research/scratch/technique-cost
cargo run --release -- dist 400       # singles-only rates + tier distribution per target
cargo run --release -- custom 300     # custom givens targets 24/22/20
cargo run --release -- usage 200      # per-technique firing rates
cargo run --release -- throughput     # puzzles/second: generate / analyze / detect / solve
cargo run --release -- attempts 300   # expected attempts + wall time per form (hard)
cd ../.. && racket research/scratch/technique-cost.rkt            # Racket cross-check (n=30)
racket research/scratch/technique-cost.rkt hard 100               # Racket hard-only (n=100)
```
