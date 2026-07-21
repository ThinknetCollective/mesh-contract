//! # Puzzle Crate
//!
//! Core puzzle trait, built-in implementations, and a dynamic registry
//! for the ThinkMesh protocol.
//!
//! ## Adding a new puzzle type
//!
//! 1. Implement the [`Puzzle`](traits::Puzzle) trait for your type.
//! 2. Register it with [`PuzzleRegistry`](registry::PuzzleRegistry) at startup.
//!
//! No changes to core dispatch logic are needed — the registry handles
//! routing by puzzle name.

pub mod difficulty;
pub mod math;
pub mod puzzle;
pub mod registry;
pub mod sequence;
pub mod traits;
pub mod word_scramble;

// Re-exports for convenience
pub use difficulty::Difficulty;
pub use puzzle::PuzzleInstance;
pub use registry::PuzzleRegistry;
pub use traits::Puzzle;

/// Register all built-in puzzle types into the given registry.
pub fn register_builtins(registry: &mut PuzzleRegistry) {
    registry.register(Box::new(math::MathPuzzle::new()));
    registry.register(Box::new(word_scramble::WordScramblePuzzle::new()));
    registry.register(Box::new(sequence::SequencePuzzle::new()));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_registry() -> PuzzleRegistry {
        let mut reg = PuzzleRegistry::new();
        register_builtins(&mut reg);
        reg
    }

    #[test]
    fn test_registry_has_builtins() {
        let reg = setup_registry();
        assert!(reg.get("math").is_some());
        assert!(reg.get("word_scramble").is_some());
        assert!(reg.get("sequence").is_some());
        assert_eq!(reg.len(), 3);
    }

    #[test]
    fn test_registry_list() {
        let reg = setup_registry();
        let mut names = reg.list();
        names.sort();
        assert_eq!(names, vec!["math", "sequence", "word_scramble"]);
    }

    #[test]
    fn test_registry_generate_and_check() {
        let reg = setup_registry();

        // Math
        let inst = reg.generate("math", Difficulty::Easy, 42).unwrap();
        assert!(reg.check_answer("math", &inst, inst.answer()).unwrap());

        // Word scramble
        let inst = reg.generate("word_scramble", Difficulty::Easy, 0).unwrap();
        assert!(reg.check_answer("word_scramble", &inst, inst.answer()).unwrap());

        // Sequence
        let inst = reg.generate("sequence", Difficulty::Easy, 42).unwrap();
        assert!(reg.check_answer("sequence", &inst, inst.answer()).unwrap());
    }

    #[test]
    fn test_registry_hint() {
        let reg = setup_registry();
        let inst = reg.generate("math", Difficulty::Easy, 42).unwrap();
        let hint = reg.hint("math", &inst, 0).unwrap();
        assert!(!hint.is_empty());
    }

    #[test]
    fn test_registry_difficulty() {
        let reg = setup_registry();
        assert_eq!(reg.difficulty("math"), Some(Difficulty::Easy));
        assert_eq!(reg.difficulty("word_scramble"), Some(Difficulty::Medium));
        assert_eq!(reg.difficulty("sequence"), Some(Difficulty::Medium));
        assert_eq!(reg.difficulty("nonexistent"), None);
    }

    #[test]
    fn test_registry_unknown_puzzle() {
        let reg = setup_registry();
        assert!(reg.get("nonexistent").is_none());
        assert!(reg.generate("nonexistent", Difficulty::Easy, 0).is_none());
        assert!(reg.check_answer("nonexistent", &PuzzleInstance::new(
            "".into(), "".into(), vec![], "".into(), Difficulty::Easy
        ), "x").is_none());
    }

    #[test]
    fn test_custom_puzzle_registration() {
        use crate::traits::Puzzle;
        use crate::difficulty::Difficulty;
        use crate::puzzle::PuzzleInstance;

        struct EchoPuzzle;

        impl Puzzle for EchoPuzzle {
            fn name(&self) -> &str { "echo" }
            fn generate(&self, difficulty: Difficulty, seed: u64) -> PuzzleInstance {
                PuzzleInstance::new(
                    format!("echo_{}", seed),
                    "Say hello".into(),
                    vec!["Just say it".into()],
                    "hello".into(),
                    difficulty,
                )
            }
            fn check_answer(&self, instance: &PuzzleInstance, answer: &str) -> bool {
                instance.check(answer)
            }
            fn hint(&self, instance: &PuzzleInstance, _level: u32) -> String {
                instance.hints[0].clone()
            }
            fn difficulty(&self) -> Difficulty { Difficulty::Easy }
        }

        let mut reg = PuzzleRegistry::new();
        reg.register(Box::new(EchoPuzzle));
        assert!(reg.get("echo").is_some());

        let inst = reg.generate("echo", Difficulty::Easy, 1).unwrap();
        assert!(reg.check_answer("echo", &inst, "hello").unwrap());
        assert!(!reg.check_answer("echo", &inst, "goodbye").unwrap());
    }
}
