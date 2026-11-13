//! Complete-grid generation and uniqueness-checked clue removal with
//! difficulty targets (easy 40 / medium 32 / hard 26 givens).
//! Spec: `../SPEC.md` §4.

use rand::RngExt;
use rand::seq::SliceRandom;

use crate::grid::Grid;
use crate::solver::count_solutions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl std::str::FromStr for Difficulty {
    type Err = String;

    fn from_str(s: &str) -> Result<Difficulty, String> {
        match s {
            "easy" => Ok(Difficulty::Easy),
            "medium" => Ok(Difficulty::Medium),
            "hard" => Ok(Difficulty::Hard),
            _ => Err(format!(
                "difficulty-givens: contract violation; expected: 'easy, 'medium or 'hard; given: '{s}'"
            )),
        }
    }
}

impl Difficulty {
    /// Target givens count for this difficulty.
    pub fn target_givens(self) -> usize {
        match self {
            Difficulty::Easy => 40,
            Difficulty::Medium => 32,
            Difficulty::Hard => 26,
        }
    }
}

/// Random complete grid: backtracking with shuffled candidate order.
pub fn generate_complete_grid<R: RngExt + ?Sized>(rng: &mut R) -> Grid {
    let mut g = Grid::empty();
    fill(&mut g, 0, rng);
    g
}

fn fill<R: RngExt + ?Sized>(g: &mut Grid, i: usize, rng: &mut R) -> bool {
    if i == 81 {
        return true;
    }
    if !g.is_empty_cell(i) {
        return fill(g, i + 1, rng);
    }
    let mut cs = g.candidates(i);
    cs.shuffle(rng);
    for v in cs {
        g.set(i, v);
        if fill(g, i + 1, rng) {
            return true;
        }
        g.set(i, 0);
    }
    false
}

/// Remove clues from `solution` following the symmetric `order` pairs,
/// keeping a removal only if the puzzle still has a unique solution, until
/// the givens count reaches `target`.
pub fn remove_clues(solution: &Grid, target: usize, order: &[(usize, usize)]) -> Grid {
    let mut puzzle = *solution;
    for &(a, b) in order {
        if puzzle.givens() <= target {
            break;
        }
        for i in [a, b] {
            if puzzle.get(i) != 0 {
                let saved = puzzle.get(i);
                puzzle.set(i, 0);
                if count_solutions(&puzzle, 2) != 1 {
                    puzzle.set(i, saved);
                }
            }
        }
    }
    puzzle
}

/// Generate a puzzle of the given difficulty plus its solution: 11 attempts
/// (initial + 10), each with a freshly shuffled symmetric pair order, keeping
/// the candidate with the fewest givens. Randomized — tests assert invariants,
/// not golden grids.
pub fn generate_puzzle<R: RngExt + ?Sized>(difficulty: Difficulty, rng: &mut R) -> (Grid, Grid) {
    let target = difficulty.target_givens();
    let solution = generate_complete_grid(rng);
    let base_pairs: Vec<(usize, usize)> = (0..41).map(|i| (i, 80 - i)).collect();
    let mut best = remove_clues_shuffled(&solution, target, &base_pairs, rng);
    for _ in 0..10 {
        let candidate = remove_clues_shuffled(&solution, target, &base_pairs, rng);
        if candidate.givens() < best.givens() {
            best = candidate;
        }
    }
    (best, solution)
}

/// Generate a puzzle using the thread-local random generator.
pub fn generate_random(difficulty: Difficulty) -> (Grid, Grid) {
    generate_puzzle(difficulty, &mut rand::rng())
}

fn remove_clues_shuffled<R: RngExt + ?Sized>(
    solution: &Grid,
    target: usize,
    base_pairs: &[(usize, usize)],
    rng: &mut R,
) -> Grid {
    let mut pairs = base_pairs.to_vec();
    pairs.shuffle(rng);
    remove_clues(solution, target, &pairs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::solve;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn difficulty_targets_and_errors() {
        assert_eq!(Difficulty::Easy.target_givens(), 40);
        assert_eq!(Difficulty::Medium.target_givens(), 32);
        assert_eq!(Difficulty::Hard.target_givens(), 26);
        assert!("impossible".parse::<Difficulty>().is_err());
    }

    #[test]
    fn complete_grid_is_complete_and_valid() {
        let mut rng = StdRng::seed_from_u64(42);
        let g = generate_complete_grid(&mut rng);
        assert!(g.is_complete());
        assert!(g.is_valid());
    }

    #[test]
    fn generated_puzzle_invariants() {
        let mut rng = StdRng::seed_from_u64(7);
        for difficulty in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
            let (puzzle, solution) = generate_puzzle(difficulty, &mut rng);
            assert!(puzzle.is_valid());
            assert_eq!(count_solutions(&puzzle, 2), 1, "puzzle must be unique");
            assert_eq!(solve(&puzzle), Some(solution), "solve(puzzle) == solution");
            assert!(puzzle.givens() <= difficulty.target_givens());
            // puzzle is a subset of its solution
            for i in 0..81 {
                if puzzle.get(i) != 0 {
                    assert_eq!(puzzle.get(i), solution.get(i));
                }
            }
        }
    }
}
