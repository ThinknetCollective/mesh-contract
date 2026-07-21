use crate::difficulty::Difficulty;
use crate::puzzle::PuzzleInstance;

/// Core trait for all puzzle types.
///
/// Implement this trait to create a new puzzle category that can be
/// registered with [`PuzzleRegistry`](crate::registry::PuzzleRegistry)
/// and dispatched without modifying core logic.
pub trait Puzzle {
    /// Human-readable name of this puzzle type (e.g. "math", "word_scramble").
    fn name(&self) -> &str;

    /// Generate a new puzzle instance at the given difficulty using a seed
    /// for deterministic reproducibility.
    fn generate(&self, difficulty: Difficulty, seed: u64) -> PuzzleInstance;

    /// Check whether the supplied answer is correct for the given instance.
    fn check_answer(&self, instance: &PuzzleInstance, answer: &str) -> bool;

    /// Return a hint string for the given instance.
    /// `hint_level` controls how revealing the hint is (0 = mild, higher = more revealing).
    fn hint(&self, instance: &PuzzleInstance, hint_level: u32) -> String;

    /// The default difficulty for this puzzle type.
    fn difficulty(&self) -> Difficulty;
}
