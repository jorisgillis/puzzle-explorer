//! DLX solver: Knuth's Algorithm X with Dancing Links (arXiv:cs/0011047)
//! over the canonical exact-cover matrix of 9x9 sudoku: 729 placement rows
//! (r,c,d) x 324 primary columns = four constraint groups of 81 (cell rc,
//! row-digit rn, column-digit cn, box-digit bn). Each row has exactly four
//! 1s, one per group. Givens are pre-placed by covering all four columns of
//! their placement row. The search branches on the minimum-size column
//! (MRV over constraints); cover/uncover are O(1) per link via circular
//! doubly linked L/R/U/D lists. Invalid input never enters the search:
//! `(None, zero stats)` / 0. Besides the uniform `SearchStats`, DLX reports
//! `link_updates`, Knuth's machine-independent cost metric (see
//! `solve_with_dlx_stats`).

use crate::grid::{CELL_COUNT, Grid, box_of, col_of, row_of};
use crate::solver::SearchStats;

use super::Solver;

pub struct Dlx;

/// DLX stats: the uniform search stats plus link updates.
pub struct DlxStats {
    pub stats: SearchStats,
    pub link_updates: u64,
}

const COL_COUNT: usize = 324;
const ROW_COUNT: usize = 729;
const HEADER: usize = 0;
const FIRST_NODE: usize = COL_COUNT + 1; // node 0 = header, 1..=324 = columns

const fn rc_col(r: usize, c: usize) -> usize {
    1 + r * 9 + c
}
const fn rn_col(r: usize, d: usize) -> usize {
    1 + 81 + r * 9 + (d - 1)
}
const fn cn_col(c: usize, d: usize) -> usize {
    1 + 162 + c * 9 + (d - 1)
}
const fn bn_col(b: usize, d: usize) -> usize {
    1 + 243 + b * 9 + (d - 1)
}

/// placement row id for cell (r,c) with digit d
fn placement_row(r: usize, c: usize, d: usize) -> usize {
    (r * 9 + c) * 9 + (d - 1)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Solve,
    Count,
}

struct State {
    l: Vec<usize>,
    r: Vec<usize>,
    u: Vec<usize>,
    d: Vec<usize>,
    s: Vec<usize>, // column sizes, indexed by column header
    c: Vec<usize>, // node -> column header
    node_row: Vec<usize>,
    rid_node: Vec<usize>, // placement row -> its first node
    chosen: Vec<usize>,
    nodes: u64,
    max_depth: u64,
    updates: u64,
    mode: Mode,
    cap: u64,
    count: u64,
    solution: Option<Grid>,
}

fn build(g: &Grid, mode: Mode, cap: u64) -> State {
    let size = FIRST_NODE + 4 * ROW_COUNT;
    let mut l = vec![HEADER; size];
    let mut r = vec![HEADER; size];
    let mut u = vec![HEADER; size];
    let mut d = vec![HEADER; size];
    let mut s = vec![0usize; COL_COUNT + 1];
    let mut c = vec![HEADER; size];
    let mut node_row = vec![0usize; size];
    let mut rid_node = vec![0usize; ROW_COUNT];
    // header <-> column headers, circular horizontally
    for col in 1..=COL_COUNT {
        l[col] = col - 1;
        r[col - 1] = col;
    }
    l[HEADER] = COL_COUNT;
    r[COL_COUNT] = HEADER;
    // column vertical lists start empty: u[h] = d[h] = h
    for col in 1..=COL_COUNT {
        u[col] = col;
        d[col] = col;
    }
    // placement rows: four nodes each, circular horizontally, appended to
    // their column's vertical list
    let mut next = FIRST_NODE;
    for row in 0..9 {
        for col_i in 0..9 {
            for dig in 1..=9usize {
                let rid = placement_row(row, col_i, dig);
                let cols = [
                    rc_col(row, col_i),
                    rn_col(row, dig),
                    cn_col(col_i, dig),
                    bn_col(box_of(row * 9 + col_i), dig),
                ];
                let mut ns = [0usize; 4];
                for (k, &colh) in cols.iter().enumerate() {
                    let n = next;
                    next += 1;
                    c[n] = colh;
                    node_row[n] = rid;
                    s[colh] += 1;
                    // append n to column colh's vertical list (just above header)
                    let last = u[colh];
                    u[n] = last;
                    d[n] = colh;
                    u[colh] = n;
                    d[last] = n;
                    ns[k] = n;
                }
                rid_node[rid] = ns[0];
                for k in 0..4 {
                    r[ns[k]] = ns[(k + 1) % 4];
                    l[ns[(k + 1) % 4]] = ns[k];
                }
            }
        }
    }
    let mut st = State {
        l,
        r,
        u,
        d,
        s,
        c,
        node_row,
        rid_node,
        chosen: Vec::new(),
        nodes: 0,
        max_depth: 0,
        updates: 0,
        mode,
        cap,
        count: 0,
        solution: None,
    };
    // pre-place givens: select each given's placement row, covering ALL
    // four of its columns
    for i in 0..CELL_COUNT {
        let v = g.get(i);
        if v != 0 {
            place_given(&mut st, placement_row(row_of(i), col_of(i), v as usize));
        }
    }
    st
}

fn cover(st: &mut State, col: usize) {
    st.l[st.r[col]] = st.l[col];
    st.r[st.l[col]] = st.r[col];
    let mut i = st.d[col];
    while i != col {
        let mut j = st.r[i];
        while j != i {
            st.d[st.u[j]] = st.d[j];
            st.u[st.d[j]] = st.u[j];
            let jcol = st.c[j];
            st.s[jcol] -= 1;
            st.updates += 1;
            j = st.r[j];
        }
        i = st.d[i];
    }
}

fn uncover(st: &mut State, col: usize) {
    let mut i = st.u[col];
    while i != col {
        let mut j = st.l[i];
        while j != i {
            let jcol = st.c[j];
            st.s[jcol] += 1;
            st.u[st.d[j]] = j;
            st.d[st.u[j]] = j;
            st.updates += 1;
            j = st.l[j];
        }
        i = st.u[i];
    }
    st.l[st.r[col]] = col;
    st.r[st.l[col]] = col;
}

/// Select a placement row as a given: push it and cover all four of its
/// columns (unlike `select_row`, which leaves the branch column covered by
/// the caller).
fn place_given(st: &mut State, rid: usize) {
    st.chosen.push(rid);
    let n0 = st.rid_node[rid];
    let mut j = n0;
    loop {
        cover(st, st.c[j]);
        let j2 = st.r[j];
        if j2 == n0 {
            break;
        }
        j = j2;
    }
}

/// Select a row during search: cover every column it touches except the
/// branch column itself, remember it on the stack.
fn select_row(st: &mut State, i: usize) {
    st.chosen.push(st.node_row[i]);
    let mut j = st.r[i];
    while j != i {
        cover(st, st.c[j]);
        j = st.r[j];
    }
}

fn deselect_row(st: &mut State, i: usize) {
    let mut j = st.l[i];
    while j != i {
        uncover(st, st.c[j]);
        j = st.l[j];
    }
    st.chosen.pop();
}

/// The uncovered column with the fewest remaining rows (min S = MRV).
fn min_column(st: &State) -> usize {
    let mut best = HEADER;
    let mut best_s = usize::MAX;
    let mut col = st.r[HEADER];
    while col != HEADER {
        if st.s[col] < best_s {
            best = col;
            best_s = st.s[col];
        }
        col = st.r[col];
    }
    best
}

fn extract_solution(st: &State) -> Grid {
    let mut g = Grid::empty();
    for &rid in &st.chosen {
        let r = rid / 81;
        let rem = rid % 81;
        g.set(r * 9 + rem / 9, (rem % 9 + 1) as u8);
    }
    g
}

/// Returns true when the search should stop early: solution found in Solve
/// mode, cap reached in Count mode.
fn search(st: &mut State, depth: u64) -> bool {
    st.nodes += 1;
    st.max_depth = st.max_depth.max(depth);
    if st.r[HEADER] == HEADER {
        // every column covered exactly once: a complete grid
        return match st.mode {
            Mode::Count => {
                st.count += 1;
                st.count >= st.cap
            }
            Mode::Solve => {
                st.solution = Some(extract_solution(st));
                true
            }
        };
    }
    let col = min_column(st);
    cover(st, col);
    let stop = search_column(st, col, depth);
    uncover(st, col);
    stop
}

/// Try every row of column `col` in turn (walking the vertical D links).
fn search_column(st: &mut State, col: usize, depth: u64) -> bool {
    let mut i = st.d[col];
    while i != col {
        select_row(st, i);
        let stop = search(st, depth + 1);
        deselect_row(st, i);
        if stop {
            return true;
        }
        i = st.d[i];
    }
    false
}

impl Dlx {
    /// Solve with DLX stats: uniform search stats plus link updates.
    pub fn solve_with_dlx_stats(&self, g: &Grid) -> (Option<Grid>, DlxStats) {
        if g.is_valid() {
            let mut st = build(g, Mode::Solve, 0);
            search(&mut st, 0);
            (
                st.solution,
                DlxStats {
                    stats: SearchStats {
                        nodes: st.nodes,
                        max_depth: st.max_depth,
                    },
                    link_updates: st.updates,
                },
            )
        } else {
            (
                None,
                DlxStats {
                    stats: SearchStats {
                        nodes: 0,
                        max_depth: 0,
                    },
                    link_updates: 0,
                },
            )
        }
    }
}

impl Solver for Dlx {
    fn name(&self) -> &'static str {
        "dlx"
    }
    fn solve(&self, g: &Grid) -> Option<Grid> {
        self.solve_with_dlx_stats(g).0
    }
    fn solve_with_stats(&self, g: &Grid) -> (Option<Grid>, SearchStats) {
        let (sol, ds) = self.solve_with_dlx_stats(g);
        (sol, ds.stats)
    }
    fn count_solutions(&self, g: &Grid, cap: u64) -> u64 {
        if g.is_valid() {
            let mut st = build(g, Mode::Count, cap);
            search(&mut st, 0);
            st.count
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const README: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    const SOLVED: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
    // AI Escargot (Inkala 2006)
    const ESCARGOT: &str =
        "100007090030020008009600500005300900010080002600004000300000010040000007007000300";

    #[test]
    fn solves_readme_puzzle_with_link_updates() {
        let g = Grid::from_puzzle_str(README).unwrap();
        let (sol, ds) = Dlx.solve_with_dlx_stats(&g);
        assert_eq!(sol.unwrap().to_puzzle_string(), SOLVED);
        assert!(ds.stats.nodes > 0 && ds.stats.max_depth > 0);
        assert!(ds.link_updates > 0);
    }

    #[test]
    fn solves_escargot_and_counts() {
        let e = Grid::from_puzzle_str(ESCARGOT).unwrap();
        let (sol, _) = Dlx.solve_with_dlx_stats(&e);
        assert_eq!(
            sol.unwrap().to_puzzle_string(),
            crate::solver::solve(&e).unwrap().to_puzzle_string()
        );
        assert_eq!(
            Dlx.count_solutions(&Grid::from_puzzle_str(README).unwrap(), 2),
            1
        );
        assert_eq!(Dlx.count_solutions(&Grid::empty(), 2), 2);
        assert_eq!(Dlx.count_solutions(&Grid::empty(), 3), 3);
        let mut bad = Grid::empty();
        bad.set(0, 1);
        bad.set(1, 1);
        assert_eq!(Dlx.count_solutions(&bad, 2), 0);
        assert!(Dlx.solve(&bad).is_none());
    }
}
