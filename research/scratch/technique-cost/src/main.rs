//! SCRATCH experiment code for wayfinder research ticket #27
//! ("feasibility and cost of technique detection with our solver").
//! NOT part of the shipped workspace (lives under research/scratch/, depends
//! on rust/core by path). Delete freely after the findings are captured.
//!
//! Experiments:
//!   1. singles-only solvability rate of generated puzzles per givens target
//!      (easy 40 / medium 32 / hard 26, plus custom targets 24/22/20), using
//!      (a) naked-singles-only fixpoint and (b) the repo `propagate`
//!      (naked + hidden singles fixpoint)
//!   2. scratch technique analyzer (hidden singles, locked candidates,
//!      naked pairs, hidden pairs, X-Wing) -> minimal technique tier needed
//!      to solve without search, per givens target
//!   3. throughput: puzzles/sec for generate, analyze, generate+analyze,
//!      and per-puzzle detection cost (propagate fixpoint vs full solve)
//!   4. expected generate-and-test attempts (= 1 / frequency) for a unique
//!      puzzle whose minimal tier is a given technique (hidden pair, X-Wing)

use std::io::Write;
use std::time::Instant;

fn flush() {
    let _ = std::io::stdout().flush();
}

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};

use sudoku_core::generator::{Difficulty, generate_complete_grid, generate_puzzle, remove_clues};
use sudoku_core::grid::{Grid, UNITS};
use sudoku_core::solver::{count_solutions, propagate, solve_with_stats};

// ---------------------------------------------------------------- masks ----

fn masks_of(g: &Grid) -> [u16; 81] {
    let mut m = [0u16; 81];
    for i in 0..81 {
        if g.get(i) == 0 {
            let mut mask = 0u16;
            for v in g.candidates(i) {
                mask |= 1 << v;
            }
            m[i] = mask;
        } else {
            m[i] = 1 << g.get(i);
        }
    }
    m
}

// ------------------------------------------------- scratch technique solver
// Tiers (cumulative): 0 = naked+hidden singles, 1 = + locked candidates,
// 2 = + naked pairs, 3 = + hidden pairs, 4 = + X-Wing. Returns
// (solved_without_search, techniques_used).

fn solve_techniques(g: &Grid, max_tier: u8) -> (bool, Vec<&'static str>, Grid) {
    let mut w = *g;
    let mut m = masks_of(g);
    let mut used: Vec<&'static str> = Vec::new();
    let mut mark = |used: &mut Vec<&'static str>, t: &'static str| {
        if !used.contains(&t) {
            used.push(t);
        }
    };

    macro_rules! assign {
        ($i:expr, $v:expr) => {{
            let i = $i;
            let v = $v;
            if w.get(i) == 0 {
                w.set(i, v);
                m[i] = 1 << v;
                for p in sudoku_core::PEERS[i] {
                    m[p] &= !(1 << v);
                }
                true
            } else {
                w.get(i) == v
            }
        }};
    }
    loop {
        let mut changed = false;

        // tier 0a: naked singles
        for i in 0..81 {
            if w.get(i) == 0 && m[i].count_ones() == 1 {
                let v = m[i].trailing_zeros() as u8;
                changed |= assign!(i, v);
                mark(&mut used, "naked-single");
            }
        }
        // tier 0b: hidden singles
        for unit in UNITS.iter() {
            for v in 1..=9u8 {
                let spots: Vec<usize> = unit
                    .iter()
                    .copied()
                    .filter(|&i| w.get(i) == 0 && m[i] & (1 << v) != 0)
                    .collect();
                if spots.len() == 1 {
                    changed |= assign!(spots[0], v);
                    mark(&mut used, "hidden-single");
                }
            }
        }

        if max_tier >= 1 {
            // locked candidates, box -> row/col (pointing)
            for b in 0..9 {
                let bx = &UNITS[18 + b];
                for v in 1..=9u8 {
                    let cells: Vec<usize> =
                        bx.iter().copied().filter(|&i| m[i] & (1 << v) != 0).collect();
                    if cells.is_empty() || cells.iter().any(|&i| w.get(i) == v) {
                        continue;
                    }
                    let rows: std::collections::HashSet<usize> =
                        cells.iter().map(|&i| i / 9).collect();
                    let cols: std::collections::HashSet<usize> =
                        cells.iter().map(|&i| i % 9).collect();
                    if rows.len() == 1 {
                        // pointing: v confined to row r within box b -> eliminate
                        // v from the rest of row r (cells outside the box)
                        let r = *rows.iter().next().unwrap();
                        for c in 0..9 {
                            let i = r * 9 + c;
                            if !bx.contains(&i) {
                                let bit = 1u16 << v;
                                if w.get(i) == 0 && m[i] & bit != 0 {
                                    m[i] &= !bit;
                                    changed = true;
                                    mark(&mut used, "locked-candidates");
                                }
                            }
                        }
                    }
                    if cols.len() == 1 {
                        let c = *cols.iter().next().unwrap();
                        for r in 0..9 {
                            let i = r * 9 + c;
                            if !bx.contains(&i) {
                                let bit = 1u16 << v;
                                if w.get(i) == 0 && m[i] & bit != 0 {
                                    m[i] &= !bit;
                                    changed = true;
                                    mark(&mut used, "locked-candidates");
                                }
                            }
                        }
                    }
                }
            }
            // locked candidates, row/col -> box (claiming)
            for u in 0..18 {
                let unit = &UNITS[u];
                for v in 1..=9u8 {
                    let cells: Vec<usize> =
                        unit.iter().copied().filter(|&i| m[i] & (1 << v) != 0).collect();
                    if cells.is_empty() || cells.iter().any(|&i| w.get(i) == v) {
                        continue;
                    }
                    let boxes: std::collections::HashSet<usize> =
                        cells.iter().map(|&i| (i / 27) * 3 + (i % 9) / 3).collect();
                    if boxes.len() == 1 {
                        let b = *boxes.iter().next().unwrap();
                        let bx = &UNITS[18 + b];
                        for &i in bx {
                            if !unit.contains(&i) {
                                let bit = 1u16 << v;
                                if w.get(i) == 0 && m[i] & bit != 0 {
                                    m[i] &= !bit;
                                    changed = true;
                                    mark(&mut used, "locked-candidates");
                                }
                            }
                        }
                    }
                }
            }
        }

        if max_tier >= 2 {
            // naked pairs: two cells in a unit with identical 2-bit masks
            for unit in UNITS.iter() {
                let empties: Vec<usize> =
                    unit.iter().copied().filter(|&i| w.get(i) == 0).collect();
                for a in 0..empties.len() {
                    for b in (a + 1)..empties.len() {
                        let (ca, cb) = (empties[a], empties[b]);
                        if m[ca].count_ones() == 2 && m[ca] == m[cb] {
                            let pair = m[ca];
                            let mut ch = false;
                            for &i in unit.iter() {
                                if i != ca && i != cb && w.get(i) == 0 && m[i] & pair != 0 {
                                    m[i] &= !pair;
                                    ch = true;
                                }
                            }
                            if ch {
                                changed = true;
                                mark(&mut used, "naked-pair");
                            }
                        }
                    }
                }
            }
        }

        if max_tier >= 3 {
            // hidden pairs: values v1,v2 confined to the same two cells of a unit
            for unit in UNITS.iter() {
                let empties: Vec<usize> =
                    unit.iter().copied().filter(|&i| w.get(i) == 0).collect();
                for v1 in 1..=9u8 {
                    for v2 in (v1 + 1)..=9u8 {
                        let s1: Vec<usize> = empties
                            .iter()
                            .copied()
                            .filter(|&i| m[i] & (1 << v1) != 0)
                            .collect();
                        let s2: Vec<usize> = empties
                            .iter()
                            .copied()
                            .filter(|&i| m[i] & (1 << v2) != 0)
                            .collect();
                        if s1.len() == 2 && s1 == s2 {
                            let keep = (1u16 << v1) | (1u16 << v2);
                            let mut ch = false;
                            for i in s1.iter().copied() {
                                if w.get(i) == 0 && m[i] & !keep != 0 {
                                    m[i] &= keep;
                                    ch = true;
                                }
                            }
                            if ch {
                                changed = true;
                                mark(&mut used, "hidden-pair");
                            }
                        }
                    }
                }
            }
        }

        if max_tier >= 4 {
            // X-Wing on rows (and columns): value v in exactly 2 cells of each
            // of 2 rows, aligned on the same 2 columns
            for v in 1..=9u8 {
                let row_cells: Vec<(usize, Vec<usize>)> = (0..9)
                    .map(|r| {
                        (
                            r,
                            (0..9)
                                .filter(|c| w.get(r * 9 + c) == 0 && m[r * 9 + c] & (1 << v) != 0)
                                .map(|c| r * 9 + c)
                                .collect::<Vec<_>>(),
                        )
                    })
                    .filter(|(_, cs)| cs.len() == 2)
                    .collect();
                for a in 0..row_cells.len() {
                    for b in (a + 1)..row_cells.len() {
                        let cols_a: Vec<usize> =
                            row_cells[a].1.iter().map(|i| i % 9).collect();
                        let cols_b: Vec<usize> =
                            row_cells[b].1.iter().map(|i| i % 9).collect();
                        if cols_a == cols_b {
                            let mut ch = false;
                            for &c in &cols_a {
                                for r in 0..9 {
                                    let i = r * 9 + c;
                                    if !row_cells[a].1.contains(&i)
                                        && !row_cells[b].1.contains(&i)
                                        && w.get(i) == 0
                                        && m[i] & (1 << v) != 0
                                    {
                                        m[i] &= !(1 << v);
                                        ch = true;
                                    }
                                }
                            }
                            if ch {
                                changed = true;
                                mark(&mut used, "x-wing");
                            }
                        }
                    }
                }
                // column version
                let col_cells: Vec<(usize, Vec<usize>)> = (0..9)
                    .map(|c| {
                        (
                            c,
                            (0..9)
                                .filter(|r| w.get(r * 9 + c) == 0 && m[r * 9 + c] & (1 << v) != 0)
                                .map(|r| r * 9 + c)
                                .collect::<Vec<_>>(),
                        )
                    })
                    .filter(|(_, cs)| cs.len() == 2)
                    .collect();
                for a in 0..col_cells.len() {
                    for b in (a + 1)..col_cells.len() {
                        let rows_a: Vec<usize> =
                            col_cells[a].1.iter().map(|i| i / 9).collect();
                        let rows_b: Vec<usize> =
                            col_cells[b].1.iter().map(|i| i / 9).collect();
                        if rows_a == rows_b {
                            let mut ch = false;
                            for &r in &rows_a {
                                for c in 0..9 {
                                    let i = r * 9 + c;
                                    if !col_cells[a].1.contains(&i)
                                        && !col_cells[b].1.contains(&i)
                                        && w.get(i) == 0
                                        && m[i] & (1 << v) != 0
                                    {
                                        m[i] &= !(1 << v);
                                        ch = true;
                                    }
                                }
                            }
                            if ch {
                                changed = true;
                                mark(&mut used, "x-wing");
                            }
                        }
                    }
                }
            }
        }

        // dead cell => contradiction, stop
        if (0..81).any(|i| w.get(i) == 0 && m[i] == 0) {
            return (false, used, w);
        }
        if !changed {
            break;
        }
    }
    (w.is_complete(), used, w)
}

/// Minimal tier (0..=4) whose cumulative technique set solves `g` without
/// search; 5 = needs backtracking search.
fn minimal_tier(g: &Grid) -> u8 {
    for t in 0..=4u8 {
        let (ok, _used, solved) = solve_techniques(g, t);
        if ok {
            // cross-check: technique result must match the backtracking solver
            let (sol, _) = solve_with_stats(g);
            assert_eq!(Some(solved), sol, "technique solver disagrees with search on {}", g.to_puzzle_string());
            return t;
        }
    }
    5
}

/// Naked-singles-only fixpoint (strict "singles" reading, no hidden singles).
fn naked_only_complete(g: &Grid) -> bool {
    let mut w = *g;
    loop {
        let mut progressed = false;
        for i in 0..81 {
            if w.get(i) == 0 {
                let cs = w.candidates(i);
                if cs.len() == 1 {
                    w.set(i, cs[0]);
                    progressed = true;
                }
            }
        }
        if !progressed {
            break;
        }
    }
    w.is_complete()
}

// ------------------------------------------------------------ generation ---

/// Mimic `generate_puzzle` but with a custom givens target (scratch helper).
fn generate_with_target(rng: &mut StdRng, target: usize) -> Grid {
    let solution = generate_complete_grid(rng);
    let base: Vec<(usize, usize)> = (0..41).map(|i| (i, 80 - i)).collect();
    let mut best = {
        let mut pairs = base.clone();
        pairs.shuffle(rng);
        remove_clues(&solution, target, &pairs)
    };
    for _ in 0..10 {
        let mut pairs = base.clone();
        pairs.shuffle(rng);
        let cand = remove_clues(&solution, target, &pairs);
        if cand.givens() < best.givens() {
            best = cand;
        }
    }
    best
}

struct Sample {
    givens: usize,
    unique: bool,
    naked_only: bool,
    propagate_only: bool,
    tier: u8,
}

fn sample(rng: &mut StdRng, target: Option<usize>, difficulty: Option<Difficulty>, n: usize) -> Vec<Sample> {
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let puzzle = match (target, difficulty) {
            (Some(t), _) => generate_with_target(rng, t),
            (None, Some(d)) => generate_puzzle(d, rng).0,
            _ => unreachable!(),
        };
        let mut w = puzzle;
        let _ = propagate(&mut w);
        out.push(Sample {
            givens: puzzle.givens(),
            unique: count_solutions(&puzzle, 2) == 1,
            naked_only: naked_only_complete(&puzzle),
            propagate_only: w.is_complete(),
            tier: minimal_tier(&puzzle),
        });
    }
    out
}

fn report(label: &str, samples: &[Sample]) {
    let n = samples.len() as f64;
    let uniq = samples.iter().filter(|s| s.unique).count() as f64 / n;
    let naked = samples.iter().filter(|s| s.naked_only).count() as f64 / n;
    let prop = samples.iter().filter(|s| s.propagate_only).count() as f64 / n;
    let givens_avg = samples.iter().map(|s| s.givens as f64).sum::<f64>() / n;
    println!("\n## {label}  (n={}, avg givens={:.1})", samples.len(), givens_avg);
    println!("  unique-solution rate:            {:.1}%", 100.0 * uniq);
    println!("  naked-singles-only solvable:     {:.1}%", 100.0 * naked);
    println!("  propagate fixpoint solvable:     {:.1}%  (naked+hidden singles)", 100.0 * prop);
    let names = ["tier0 singles", "tier1 +locked", "tier2 +naked-pair", "tier3 +hidden-pair", "tier4 +x-wing", "tier5 needs-search"];
    for t in 0..=5u8 {
        let c = samples.iter().filter(|s| s.tier == t).count();
        println!("  {:24}: {:3}  ({:.1}%)   expected attempts for exactly this tier: {:.0}",
                 names[t as usize], c, 100.0 * c as f64 / n, if c > 0 { n / c as f64 } else { f64::INFINITY });
    }
    flush();
}

fn main() {
    // args: [mode] [n]   mode = dist | custom | throughput | attempts | all
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("all");
    let n: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(150);
    let seed = 20261115u64;

    // sanity: the README puzzle is singles-only
    let readme = Grid::from_puzzle_str(
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
    )
    .unwrap();
    assert_eq!(minimal_tier(&readme), 0, "README puzzle should be tier 0");
    println!("sanity: README puzzle minimal tier = 0 OK");
    flush();

    // ---- experiment 1+2: distributions per givens target -------------------
    let mut rng = StdRng::seed_from_u64(seed);
    if mode == "dist" || mode == "all" {
        for (name, diff) in [("easy (target 40)", Difficulty::Easy),
                             ("medium (target 32)", Difficulty::Medium),
                             ("hard (target 26)", Difficulty::Hard)] {
            let s = sample(&mut rng, None, Some(diff), n);
            report(&format!("generated puzzles, {name}"), &s);
        }
    }
    // harder custom targets to see where advanced techniques become common
    if mode == "custom" || mode == "all" {
        for t in [24usize, 22, 20] {
            let s = sample(&mut rng, Some(t), None, n / 2);
            report(&format!("generated puzzles, custom target {t} givens"), &s);
        }
    }

    // ---- usage: how often each technique fires at all (tier-4 solver) ------
    if mode == "usage" {
        let mut rng = StdRng::seed_from_u64(seed + 4);
        let mut counts = std::collections::HashMap::new();
        let mut solved = 0usize;
        for (name, diff) in [("easy", Difficulty::Easy), ("medium", Difficulty::Medium), ("hard", Difficulty::Hard)] {
            let mut rng = StdRng::seed_from_u64(seed + 4);
            let mut c = std::collections::HashMap::new();
            let mut solved = 0usize;
            for _ in 0..n {
                let p = generate_puzzle(diff, &mut rng).0;
                let (ok, used, _) = solve_techniques(&p, 4);
                if ok { solved += 1; }
                for t in used { *c.entry(t).or_insert(0usize) += 1; }
            }
            println!("\n## technique usage at tier<=4 solve, {name} (n={n}, solved-without-search {solved})", );
            for t in ["naked-single", "hidden-single", "locked-candidates", "naked-pair", "hidden-pair", "x-wing"] {
                let cnt = c.get(t).copied().unwrap_or(0);
                println!("  {:18}: {:4} ({:.1}%)", t, cnt, 100.0 * cnt as f64 / n as f64);
            }
            counts.extend(c);
            solved += solved;
            let _ = solved;
        }
        flush();
    }

    // ---- experiment 3: throughput ------------------------------------------
    if mode == "throughput" || mode == "all" {
    println!("\n## throughput (release build)");
    for (name, diff) in [("easy", Difficulty::Easy), ("medium", Difficulty::Medium), ("hard", Difficulty::Hard)] {
        let mut rng2 = StdRng::seed_from_u64(seed + 1);
        let m = 100;
        // generate only
        let t0 = Instant::now();
        let mut puzzles = Vec::with_capacity(m);
        for _ in 0..m {
            puzzles.push(generate_puzzle(diff, &mut rng2).0);
        }
        let gen_dt = t0.elapsed();
        // analyze only (full technique tier classification + uniqueness re-check)
        let t0 = Instant::now();
        for p in &puzzles {
            let _ = minimal_tier(p);
            let _ = count_solutions(p, 2);
        }
        let an_dt = t0.elapsed();
        // detection only: propagate fixpoint + completeness check
        let t0 = Instant::now();
        let mut det = 0usize;
        for p in &puzzles {
            let mut w = *p;
            propagate(&mut w);
            if w.is_complete() {
                det += 1;
            }
        }
        let det_dt = t0.elapsed();
        // full solver for comparison
        let t0 = Instant::now();
        let mut nodes = 0u64;
        for p in &puzzles {
            let (_, st) = solve_with_stats(p);
            nodes += st.nodes;
        }
        let solve_dt = t0.elapsed();
        println!("  {name}: generate {:.0} puz/s ({:.2} ms/puz) | generate+analyze {:.0} puz/s ({:.2} ms/puz) | analyze-only {:.0} puz/s ({:.3} ms/puz) | detect-only {:.0} puz/s ({:.1} us/puz, singles-only {}%) | full solve {:.0} puz/s ({:.3} ms/puz, avg nodes {:.0})",
            m as f64 / gen_dt.as_secs_f64(), gen_dt.as_secs_f64() * 1000.0 / m as f64,
            m as f64 / (gen_dt + an_dt).as_secs_f64(), (gen_dt + an_dt).as_secs_f64() * 1000.0 / m as f64,
            m as f64 / an_dt.as_secs_f64(), an_dt.as_secs_f64() * 1e6 / m as f64,
            m as f64 / det_dt.as_secs_f64(), det_dt.as_secs_f64() * 1e6 / m as f64, 100.0 * det as f64 / m as f64,
            m as f64 / solve_dt.as_secs_f64(), solve_dt.as_secs_f64() * 1000.0 / m as f64, nodes as f64 / m as f64);
    }
    flush();
    }

    // ---- experiment 4: expected wall time for form-constrained generation --
    if mode == "attempts" || mode == "all" {
    println!("\n## expected generate-and-test wall time (hard target 26, from tier distribution above)");
    let mut rng3 = StdRng::seed_from_u64(seed + 2);
    let s = sample(&mut rng3, None, Some(Difficulty::Hard), n);
    let m = 50usize;
    let mut rng4 = StdRng::seed_from_u64(seed + 3);
    let t0 = Instant::now();
    for _ in 0..m {
        let _ = generate_puzzle(Difficulty::Hard, &mut rng4);
    }
    let ms_per_gen = t0.elapsed().as_secs_f64() * 1000.0 / m as f64;
    let n = s.len() as f64;
    for (t, name) in [(3u8, "requires hidden pair (tier 3)"), (4, "requires X-Wing (tier 4)"), (5, "requires search (tier 5)"), (0, "singles-only (tier 0)")] {
        let p = s.iter().filter(|x| x.tier == t).count() as f64 / n;
        if p > 0.0 {
            let attempts = 1.0 / p;
            println!("  {name}: p={:.3} ({:.1}%) -> expected attempts {:.0}, wall time {:.1} s at {:.2} ms/gen",
                     p, 100.0 * p, attempts, attempts * ms_per_gen / 1000.0, ms_per_gen);
        } else {
            println!("  {name}: p=0 in sample (n={:.0}) -> expected attempts > {:.0}, wall time > {:.1} s",
                     n, n, n * ms_per_gen / 1000.0);
        }
    }
    flush();
    }

    // sanity: rng actually used
    let _ = rng.random::<u8>();
}
