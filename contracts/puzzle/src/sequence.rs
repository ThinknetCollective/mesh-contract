use std::collections::HashMap;

use crate::difficulty::Difficulty;
use crate::puzzle::PuzzleInstance;
use crate::traits::Puzzle;

/// Built-in number-sequence puzzle.
///
/// Presents a numeric sequence and asks the solver to find the next number.
/// Sequence patterns (arithmetic, geometric, Fibonacci-like) depend on difficulty.
pub struct SequencePuzzle;

impl SequencePuzzle {
    pub fn new() -> Self {
        SequencePuzzle
    }
}

impl Default for SequencePuzzle {
    fn default() -> Self {
        Self::new()
    }
}

impl Puzzle for SequencePuzzle {
    fn name(&self) -> &str {
        "sequence"
    }

    fn generate(&self, difficulty: Difficulty, seed: u64) -> PuzzleInstance {
        let (terms, answer, pattern_desc) = match difficulty {
            Difficulty::Easy => {
                // Arithmetic: start + increment each step
                let start = (seed % 10) + 1;
                let step = ((seed / 7) % 5) + 2;
                let terms: Vec<u64> = (0..5).map(|i| start + i * step).collect();
                let next = start + 5 * step;
                (terms, next, format!("add {} each time", step))
            }
            Difficulty::Medium => {
                // Geometric: multiply by a constant
                let start = (seed % 5) + 2;
                let ratio = ((seed / 7) % 3) + 2;
                let mut terms = Vec::new();
                let mut val = start;
                for _ in 0..5 {
                    terms.push(val);
                    val *= ratio;
                }
                let next = val;
                (terms, next, format!("multiply by {} each time", ratio))
            }
            Difficulty::Hard => {
                // Fibonacci-like: each term is sum of two preceding
                let a = (seed % 5) + 1;
                let b = ((seed / 7) % 5) + 1;
                let mut terms = vec![a, b];
                for _ in 2..6 {
                    let next = terms[terms.len() - 1] + terms[terms.len() - 2];
                    terms.push(next);
                }
                // The terms vector contains the first 6 values; display the first 5 and
                // use the 6th value as the "next" term.
                let next_val = terms[terms.len() - 1];
                let display_terms = terms[..5].to_vec();
                (display_terms, next_val, "each number is the sum of the two before it".to_string())
            }
            Difficulty::Expert => {
                // Squares: n^2 + offset
                let offset = (seed % 10) as i64;
                let start_n = ((seed / 7) % 5) + 1;
                let terms: Vec<i64> = (0..5)
                    .map(|i| ((start_n + i) as i64).pow(2) + offset)
                    .collect();
                let next = ((start_n + 5) as i64).pow(2) + offset;
                (terms.into_iter().map(|v| v as u64).collect(), next as u64, "each term is n² + constant".to_string())
            }
        };

        let terms_str: Vec<String> = terms.iter().map(|t| t.to_string()).collect();
        let question = format!(
            "What comes next? {}",
            terms_str.join(", ")
        );

        let id = format!("seq_{}_{}", difficulty, seed);
        let hints = vec![
            format!("The answer is a {}-digit number", answer.to_string().len()),
            format!("Pattern: {}", pattern_desc),
            format!("The first digit of the answer is {}", answer.to_string().chars().next().unwrap_or('0')),
            format!("The answer is {}", answer),
        ];

        let mut metadata = HashMap::new();
        metadata.insert("pattern".to_string(), pattern_desc);
        metadata.insert("terms".to_string(), terms_str.join(","));

        let mut instance =
            PuzzleInstance::new(id, question, hints, answer.to_string(), difficulty);
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
    fn test_sequence_easy() {
        let puzzle = SequencePuzzle::new();
        let instance = puzzle.generate(Difficulty::Easy, 42);
        // seed=42: start=(42%10)+1=3, step=((42/7)%5)+2=(6%5)+2=3
        // terms: 3,6,9,12,15  next=18
        assert_eq!(instance.answer(), "18");
        assert!(puzzle.check_answer(&instance, "18"));
        assert!(!puzzle.check_answer(&instance, "19"));
    }

    #[test]
    fn test_sequence_medium() {
        let puzzle = SequencePuzzle::new();
        let instance = puzzle.generate(Difficulty::Medium, 14);
        // seed=14: start=(14%5)+2=(4)+2=6, ratio=((14/7)%3)+2=(2%3)+2=4
        // terms: 6,24,96,384,1536  next=6144
        assert_eq!(instance.answer(), "6144");
        assert!(puzzle.check_answer(&instance, "6144"));
    }

    #[test]
    fn test_sequence_hard() {
        let puzzle = SequencePuzzle::new();
        let instance = puzzle.generate(Difficulty::Hard, 20);
        // seed=20: a=(20%5)+1=1, b=((20/7)%5)+1=(2%5)+1=3
        // fib-like: 1,3,4,7,11  next=18
        assert_eq!(instance.answer(), "18");
        assert!(puzzle.check_answer(&instance, "18"));
    }

    #[test]
    fn test_sequence_deterministic() {
        let puzzle = SequencePuzzle::new();
        let a = puzzle.generate(Difficulty::Easy, 99);
        let b = puzzle.generate(Difficulty::Easy, 99);
        assert_eq!(a.question, b.question);
        assert_eq!(a.answer(), b.answer());
    }

    #[test]
    fn test_sequence_hints() {
        let puzzle = SequencePuzzle::new();
        let instance = puzzle.generate(Difficulty::Easy, 42);
        let h0 = puzzle.hint(&instance, 0);
        assert!(h0.contains("digit"));
        let h_last = puzzle.hint(&instance, 100);
        assert!(h_last.contains("18"));
    }
}
