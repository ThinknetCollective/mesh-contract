use std::collections::HashMap;

use crate::difficulty::Difficulty;
use crate::puzzle::PuzzleInstance;
use crate::traits::Puzzle;

/// Built-in word-scramble puzzle.
///
/// Given a scrambled word, the solver must guess the original word.
/// Word lists are selected by difficulty; the scramble order is
/// derived from the seed for reproducibility.
pub struct WordScramblePuzzle;

impl WordScramblePuzzle {
    pub fn new() -> Self {
        WordScramblePuzzle
    }

    fn words_for_difficulty(difficulty: Difficulty) -> &'static [&'static str] {
        match difficulty {
            Difficulty::Easy => &["cat", "dog", "sun", "hat", "run", "cup", "bed", "pen"],
            Difficulty::Medium => &[
                "planet", "garden", "bridge", "castle", "rocket", "forest", "ocean", "winter",
            ],
            Difficulty::Hard => &[
                "algorithm", "adventure", "chocolate", "butterfly", "telephone", "dinosaur",
                "microscope", "universe",
            ],
            Difficulty::Expert => &[
                "cryptographic", "extraordinary", "consciousness", "revolutionary",
                "infrastructure", "pharmaceutical", "quintessential", "interstellar",
            ],
        }
    }

    /// Deterministic scramble: rotate characters by an amount derived from seed.
    fn scramble(word: &str, seed: u64) -> String {
        let chars: Vec<char> = word.chars().collect();
        let len = chars.len();
        if len <= 1 {
            return word.to_string();
        }
        let shift = (seed % (len as u64 - 1)) + 1;
        let mut scrambled = chars.clone();
        for i in 0..len {
            scrambled[i] = chars[(i + shift as usize) % len];
        }
        // Make sure the scrambled version differs from the original
        if scrambled == chars {
            scrambled.reverse();
        }
        scrambled.into_iter().collect()
    }
}

impl Default for WordScramblePuzzle {
    fn default() -> Self {
        Self::new()
    }
}

impl Puzzle for WordScramblePuzzle {
    fn name(&self) -> &str {
        "word_scramble"
    }

    fn generate(&self, difficulty: Difficulty, seed: u64) -> PuzzleInstance {
        let words = Self::words_for_difficulty(difficulty);
        let word = words[(seed as usize) % words.len()];
        let scrambled = Self::scramble(word, seed);

        let id = format!("word_{}_{}", difficulty, seed);
        let question = format!("Unscramble this word: {}", scrambled);
        let hints = vec![
            format!("The word has {} letters", word.len()),
            format!("The first letter is '{}'", word.chars().next().unwrap()),
            format!("The word is related to everyday things"),
            format!("The answer is '{}'", word),
        ];

        let mut metadata = HashMap::new();
        metadata.insert("original_word".to_string(), word.to_string());
        metadata.insert("scrambled".to_string(), scrambled);

        let mut instance =
            PuzzleInstance::new(id, question, hints, word.to_string(), difficulty);
        instance.metadata = metadata;
        instance
    }

    fn check_answer(&self, instance: &PuzzleInstance, answer: &str) -> bool {
        instance.check(answer)
    }

    fn hint(&self, instance: &PuzzleInstance, hint_level: u32) -> String {
        let idx = (hint_level as usize).min(instance.hints.len().saturating_sub(1));
        instance.hints[idx].clone()
    }

    fn difficulty(&self) -> Difficulty {
        Difficulty::Medium
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_scramble_easy() {
        let puzzle = WordScramblePuzzle::new();
        let instance = puzzle.generate(Difficulty::Easy, 0);
        // seed=0: word = words[0%8] = "cat", scramble with shift=0%2+1=1
        assert!(puzzle.check_answer(&instance, "cat"));
        assert!(!puzzle.check_answer(&instance, "dog"));
    }

    #[test]
    fn test_word_scramble_deterministic() {
        let puzzle = WordScramblePuzzle::new();
        let a = puzzle.generate(Difficulty::Easy, 42);
        let b = puzzle.generate(Difficulty::Easy, 42);
        assert_eq!(a.question, b.question);
        assert_eq!(a.answer(), b.answer());
    }

    #[test]
    fn test_word_scramble_hints() {
        let puzzle = WordScramblePuzzle::new();
        let instance = puzzle.generate(Difficulty::Medium, 0);
        let h0 = puzzle.hint(&instance, 0);
        assert!(h0.contains("letters"));
        let last = puzzle.hint(&instance, 10);
        assert!(last.contains("answer"));
    }

    #[test]
    fn test_word_scramble_difficulty_default() {
        let puzzle = WordScramblePuzzle::new();
        assert_eq!(puzzle.difficulty(), Difficulty::Medium);
    }

    #[test]
    fn test_scramble_differs_from_original() {
        // Verify scramble always produces a different string
        for seed in 0..100 {
            let puzzle = WordScramblePuzzle::new();
            let instance = puzzle.generate(Difficulty::Easy, seed);
            let scrambled = instance.metadata.get("scrambled").unwrap();
            let original = instance.answer();
            assert_ne!(scrambled, original, "Scrambled word should differ from original (seed={})", seed);
        }
    }
}
