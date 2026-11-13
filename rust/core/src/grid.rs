//! Grid model: cells, units, peers, candidates, validation, and the
//! puzzle-string / pretty-print formats. Spec: `../SPEC.md` §2.

use std::fmt;

pub const CELL_COUNT: usize = 81;

pub const fn row_of(i: usize) -> usize {
    i / 9
}
pub const fn col_of(i: usize) -> usize {
    i % 9
}
pub const fn box_of(i: usize) -> usize {
    (i / 27) * 3 + (i % 9) / 3
}

const fn shares_unit(i: usize, j: usize) -> bool {
    row_of(i) == row_of(j) || col_of(i) == col_of(j) || box_of(i) == box_of(j)
}

const fn build_units() -> [[usize; 9]; 27] {
    let mut units = [[0usize; 9]; 27];
    let mut r = 0;
    while r < 9 {
        let mut c = 0;
        while c < 9 {
            units[r][c] = r * 9 + c;
            c += 1;
        }
        r += 1;
    }
    let mut c = 0;
    while c < 9 {
        let mut r = 0;
        while r < 9 {
            units[9 + c][r] = r * 9 + c;
            r += 1;
        }
        c += 1;
    }
    let mut b = 0;
    while b < 9 {
        let br = (b / 3) * 3;
        let bc = (b % 3) * 3;
        let mut k = 0;
        while k < 9 {
            units[18 + b][k] = (br + k / 3) * 9 + (bc + k % 3);
            k += 1;
        }
        b += 1;
    }
    units
}

/// The 27 units: 9 rows, then 9 columns, then 9 boxes.
pub const UNITS: [[usize; 9]; 27] = build_units();

const fn build_peers() -> [[usize; 20]; 81] {
    let mut peers = [[0usize; 20]; 81];
    let mut i = 0;
    while i < CELL_COUNT {
        let mut n = 0;
        let mut j = 0;
        while j < CELL_COUNT {
            if j != i && shares_unit(i, j) {
                peers[i][n] = j;
                n += 1;
            }
            j += 1;
        }
        i += 1;
    }
    peers
}

/// `PEERS[i]` = the 20 cells sharing a unit with cell `i`, ascending.
pub const PEERS: [[usize; 20]; 81] = build_peers();

/// A 9x9 grid, row-major, `0` = empty cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Grid(pub [u8; CELL_COUNT]);

/// Error from [`Grid::from_puzzle_str`]. Disallowed characters are filtered
/// out before the length check (Racket `string->grid` quirk, kept for parity).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseGridError {
    pub given: String,
}

impl fmt::Display for ParseGridError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid puzzle string: expected: 81-character string of digits 0-9; given: {:?}",
            self.given
        )
    }
}

impl std::error::Error for ParseGridError {}

impl Grid {
    pub fn empty() -> Grid {
        Grid([0; CELL_COUNT])
    }

    /// Parse an 81-char puzzle string (`1`-`9`, `0` or `.` for empty).
    pub fn from_puzzle_str(s: &str) -> Result<Grid, ParseGridError> {
        let filtered: Vec<char> = s
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if filtered.len() != CELL_COUNT {
            return Err(ParseGridError {
                given: s.to_string(),
            });
        }
        let mut g = Grid::empty();
        for (i, c) in filtered.iter().enumerate() {
            g.0[i] = if *c == '.' { 0 } else { *c as u8 - b'0' };
        }
        Ok(g)
    }

    /// 81-char row-major string, `0` for empty cells.
    pub fn to_puzzle_string(&self) -> String {
        self.0
            .iter()
            .map(|&v| if v == 0 { '0' } else { (b'0' + v) as char })
            .collect()
    }

    pub fn get(&self, i: usize) -> u8 {
        self.0[i]
    }

    pub fn set(&mut self, i: usize, v: u8) {
        self.0[i] = v;
    }

    pub fn is_empty_cell(&self, i: usize) -> bool {
        self.0[i] == 0
    }

    /// Number of filled (non-zero) cells — the givens of a puzzle, or the
    /// filled cells of a board in progress.
    pub fn givens(&self) -> usize {
        self.0.iter().filter(|&&v| v != 0).count()
    }

    /// No unit contains a repeated non-zero value.
    pub fn is_valid(&self) -> bool {
        for unit in UNITS.iter() {
            let mut seen = [false; 10];
            for &c in unit.iter() {
                let v = self.0[c] as usize;
                if v != 0 {
                    if seen[v] {
                        return false;
                    }
                    seen[v] = true;
                }
            }
        }
        true
    }

    /// All cells filled 1-9 and valid.
    pub fn is_complete(&self) -> bool {
        self.0.iter().all(|&v| (1..=9).contains(&v)) && self.is_valid()
    }

    /// Values 1-9 not present among the peers of cell `i`.
    pub fn candidates(&self, i: usize) -> Vec<u8> {
        let mut used = [false; 10];
        for &p in PEERS[i].iter() {
            used[self.0[p] as usize] = true;
        }
        (1..=9u8).filter(|v| !used[*v as usize]).collect()
    }

    /// The classic ASCII rendering: box separators, `.` for empty cells.
    pub fn to_pretty_string(&self) -> String {
        const H: &str = "+-------+-------+-------+";
        let cell = |i: usize| {
            let v = self.0[i];
            if v == 0 { '.' } else { (b'0' + v) as char }
        };
        let mut out = String::new();
        for r in 0..9 {
            if r % 3 == 0 {
                out.push_str(H);
                out.push('\n');
            }
            let groups: Vec<String> = (0..3)
                .map(|b| {
                    (0..3)
                        .map(|c| cell(r * 9 + b * 3 + c).to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect();
            out.push_str(&format!("| {} |\n", groups.join(" | ")));
        }
        out.push_str(H);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_cell_has_20_distinct_peers() {
        for (i, p) in PEERS.iter().enumerate() {
            assert_eq!(p.len(), 20);
            assert!(!p.contains(&i));
            let mut sorted = p.to_vec();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), 20, "peers of {i} not distinct");
        }
        assert_eq!(
            PEERS[0],
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 18, 19, 20, 27, 36, 45, 54, 63, 72
            ]
        );
    }

    #[test]
    fn row_col_box_of() {
        assert_eq!(row_of(0), 0);
        assert_eq!(col_of(80), 8);
        assert_eq!(box_of(0), 0);
        assert_eq!(box_of(40), 4);
        assert_eq!(box_of(80), 8);
    }

    #[test]
    fn puzzle_string_round_trip_and_dots() {
        let s = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        let g = Grid::from_puzzle_str(s).unwrap();
        assert_eq!(g.to_puzzle_string(), s);
        let dots = s.replacen('0', ".", 1);
        assert_eq!(Grid::from_puzzle_str(&dots).unwrap(), g);
    }

    #[test]
    fn parse_rejects_wrong_length() {
        assert!(Grid::from_puzzle_str("123").is_err());
        // quirk: disallowed chars are filtered before the length check
        assert!(Grid::from_puzzle_str(&format!("{}x", "1".repeat(81))).is_ok());
        assert!(Grid::from_puzzle_str(&"a".repeat(81)).is_err());
    }

    #[test]
    fn valid_and_complete() {
        let solved = Grid::from_puzzle_str(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        )
        .unwrap();
        assert!(solved.is_valid());
        assert!(solved.is_complete());
        assert!(Grid::empty().is_valid());
        assert!(!Grid::empty().is_complete());
        let mut bad = solved;
        bad.set(0, 3); // duplicates a peer's value? cell 0 = 5; set to 3 → box/row conflict
        assert!(!bad.is_valid());
    }

    #[test]
    fn pretty_string_has_13_lines() {
        let g = Grid::from_puzzle_str(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        )
        .unwrap();
        assert_eq!(g.to_pretty_string().lines().count(), 13);
    }
}
