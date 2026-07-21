use crate::Difficulty;

/// A generated puzzle instance ready to be presented to a solver.
#[derive(Debug, Clone)]
pub struct PuzzleInstance {
    /// Unique identifier for this puzzle instance.
    pub id: String,
    /// The puzzle question/text presented to the solver.
    pub question: String,
    /// Available hints (from mild to revealing).
    pub hints: Vec<String>,
    /// The correct answer (kept secret from the solver).
    answer: String,
    /// Difficulty level of this instance.
    pub difficulty: Difficulty,
    /// Arbitrary metadata (e.g. "op" => "addition" for math puzzles).
    pub metadata: std::collections::HashMap<String, String>,
}

impl PuzzleInstance {
    /// Create a new puzzle instance.
    pub fn new(
        id: String,
        question: String,
        hints: Vec<String>,
        answer: String,
        difficulty: Difficulty,
    ) -> Self {
        Self {
            id,
            question,
            hints,
            answer,
            difficulty,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Verify whether `attempt` matches the correct answer (case-insensitive, trimmed).
    pub fn check(&self, attempt: &str) -> bool {
        self.answer.eq_ignore_ascii_case(&attempt.trim())
    }

    /// Return the stored answer (for testing / debugging).
    pub fn answer(&self) -> &str {
        &self.answer
    }
}
