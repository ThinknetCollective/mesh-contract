use std::collections::HashMap;

use crate::difficulty::Difficulty;
use crate::puzzle::PuzzleInstance;
use crate::traits::Puzzle;

/// Built-in arithmetic puzzle.
///
/// Generates math problems (addition, subtraction, multiplication)
/// whose complexity scales with difficulty.
pub struct MathPuzzle;

impl MathPuzzle {
    pub fn new() -> Self {
        MathPuzzle
    }
}

impl Default for MathPuzzle {
    fn default() -> Self {
        Self::new()
    }
}

impl Puzzle for MathPuzzle {
    fn name(&self) -> &str {
        "math"
    }

    fn generate(&self, difficulty: Difficulty, seed: u64) -> PuzzleInstance {
        let (a, b, op, op_symbol) = match difficulty {
            Difficulty::Easy => {
                let a = (seed % 10) + 1;
                let b = ((seed / 7) % 10) + 1;
                (a, b, 0u8, "+")
            }
            Difficulty::Medium => {
                let a = (seed % 50) + 10;
                let b = ((seed / 7) % 50) + 10;
                (a, b, 1u8, "-")
            }
            Difficulty::Hard => {
                let a = (seed % 12) + 2;
                let b = ((seed / 7) % 12) + 2;
                (a, b, 2u8, "*")
            }
            Difficulty::Expert => {
                let a = (seed % 50) + 10;
                let b = ((seed / 7) % 20) + 5;
                (a, b, 0u8, "+")
            }
        };

        let (answer, question) = match op {
            0 => (format!("{}", a + b), format!("What is {} + {}?", a, b)),
            1 => {
                let (big, small) = if a >= b { (a, b) } else { (b, a) };
                (format!("{}", big - small), format!("What is {} - {}?", big, small))
            }
            2 => (format!("{}", a * b), format!("What is {} * {}?", a, b)),
            _ => unreachable!(),
        };

        let id = format!("math_{}_{}", difficulty, seed);
        let hints = vec![
            format!("The answer is between {} and {}", answer.parse::<i64>().unwrap_or(0).saturating_sub(10), answer.parse::<i64>().unwrap_or(0) + 10),
            format!("The first digit is {}", answer.chars().next().unwrap_or('0')),
            format!("The answer is {}", answer),
        ];

        let mut metadata = HashMap::new();
        metadata.insert("op".to_string(), op_symbol.to_string());
        metadata.insert("a".to_string(), a.to_string());
        metadata.insert("b".to_string(), b.to_string());

        let mut instance = PuzzleInstance::new(id, question, hints, answer, difficulty);
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
        Difficulty::Easy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math_easy_addition() {
        let puzzle = MathPuzzle::new();
        let instance = puzzle.generate(Difficulty::Easy, 42);
        // seed=42: a=(42%10)+1=3, b=((42/7)%10)+1=(6%10)+1=7 => 3+7=10
        assert_eq!(instance.answer(), "10");
        assert!(puzzle.check_answer(&instance, "10"));
        assert!(!puzzle.check_answer(&instance, "11"));
        assert!(instance.question.contains("+"));
    }

    #[test]
    fn test_math_medium_subtraction() {
        let puzzle = MathPuzzle::new();
        let instance = puzzle.generate(Difficulty::Medium, 100);
        // seed=100: a=(100%50)+10=10, b=((100/7)%50)+10=(14%50)+10=24
        // big=24, small=10 => 24-10=14
        assert_eq!(instance.answer(), "14");
        assert!(puzzle.check_answer(&instance, "14"));
        assert!(instance.question.contains("-"));
    }

    #[test]
    fn test_math_hard_multiplication() {
        let puzzle = MathPuzzle::new();
        let instance = puzzle.generate(Difficulty::Hard, 30);
        // seed=30: a=(30%12)+2=8, b=((30/7)%12)+2=(4%12)+2=6 => 8*6=48
        assert_eq!(instance.answer(), "48");
        assert!(puzzle.check_answer(&instance, "48"));
        assert!(instance.question.contains("*"));
    }

    #[test]
    fn test_math_hints_progressive() {
        let puzzle = MathPuzzle::new();
        let instance = puzzle.generate(Difficulty::Easy, 42);
        let h0 = puzzle.hint(&instance, 0);
        let h1 = puzzle.hint(&instance, 1);
        let h2 = puzzle.hint(&instance, 2);
        // Last hint reveals the answer
        assert!(h2.contains("10"));
        assert!(!h0.is_empty());
        assert!(!h1.is_empty());
    }

    #[test]
    fn test_math_difficulty_default() {
        let puzzle = MathPuzzle::new();
        assert_eq!(puzzle.difficulty(), Difficulty::Easy);
    }

    #[test]
    fn test_math_case_insensitive_answer() {
        let puzzle = MathPuzzle::new();
        let instance = puzzle.generate(Difficulty::Easy, 42);
        assert!(puzzle.check_answer(&instance, " 10 "));
    }
}
