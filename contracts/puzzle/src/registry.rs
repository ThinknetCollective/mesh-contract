use std::collections::HashMap;

use crate::difficulty::Difficulty;
use crate::puzzle::PuzzleInstance;
use crate::traits::Puzzle;

/// Registry that holds puzzle implementations keyed by name.
///
/// New puzzle types can be registered at startup without touching
/// any dispatch logic — just call [`register`](PuzzleRegistry::register).
pub struct PuzzleRegistry {
    puzzles: HashMap<String, Box<dyn Puzzle>>,
}

impl PuzzleRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            puzzles: HashMap::new(),
        }
    }

    /// Register a puzzle implementation.
    /// The puzzle's [`name()`](Puzzle::name) is used as the key.
    pub fn register(&mut self, puzzle: Box<dyn Puzzle>) {
        let name = puzzle.name().to_string();
        self.puzzles.insert(name, puzzle);
    }

    /// Look up a puzzle type by name.
    pub fn get(&self, name: &str) -> Option<&dyn Puzzle> {
        self.puzzles.get(name).map(|p| p.as_ref())
    }

    /// List all registered puzzle type names.
    pub fn list(&self) -> Vec<String> {
        self.puzzles.keys().cloned().collect()
    }

    /// Convenience: generate a puzzle instance by type name.
    pub fn generate(
        &self,
        name: &str,
        difficulty: Difficulty,
        seed: u64,
    ) -> Option<PuzzleInstance> {
        self.get(name).map(|p| p.generate(difficulty, seed))
    }

    /// Convenience: check an answer by type name.
    pub fn check_answer(
        &self,
        name: &str,
        instance: &PuzzleInstance,
        answer: &str,
    ) -> Option<bool> {
        self.get(name).map(|p| p.check_answer(instance, answer))
    }

    /// Convenience: get a hint by type name.
    pub fn hint(
        &self,
        name: &str,
        instance: &PuzzleInstance,
        hint_level: u32,
    ) -> Option<String> {
        self.get(name).map(|p| p.hint(instance, hint_level))
    }

    /// Convenience: get default difficulty by type name.
    pub fn difficulty(&self, name: &str) -> Option<Difficulty> {
        self.get(name).map(|p| p.difficulty())
    }

    /// Number of registered puzzle types.
    pub fn len(&self) -> usize {
        self.puzzles.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.puzzles.is_empty()
    }
}

impl Default for PuzzleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
