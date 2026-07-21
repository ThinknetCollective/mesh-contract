use puzzle::{Puzzle, PuzzleInstance, PuzzleRegistry, Difficulty};

// Example of a third-party puzzle implementation and registration.
// Build with: cargo run -p mesh-puzzle-examples --example third_party_puzzle

struct EchoPuzzle;

impl EchoPuzzle {
    fn new() -> Self { EchoPuzzle }
}

impl Puzzle for EchoPuzzle {
    fn name(&self) -> &str { "echo" }

    fn generate(&self, difficulty: Difficulty, seed: u64) -> PuzzleInstance {
        let id = format!("echo_{}_{}", difficulty, seed);
        let question = format!("Echo puzzle (seed={}): say the secret word", seed);
        let hints = vec!["It's a friendly greeting".into(), "The answer is 'hello'".into()];
        let answer = "hello".to_string();
        PuzzleInstance::new(id, question, hints, answer, difficulty)
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

fn main() {
    let mut reg = PuzzleRegistry::new();
    // Register builtins (optional) and a third-party puzzle
    puzzle::register_builtins(&mut reg);
    reg.register(Box::new(EchoPuzzle::new()));

    // Generate a puzzle via the registry
    let inst = reg.generate("echo", Difficulty::Easy, 7).expect("should generate");
    println!("Generated: {}\nQuestion: {}", inst.id, inst.question);

    // Hint and check
    println!("Hint level 0: {}", reg.hint("echo", &inst, 0).unwrap());
    println!("Answer check 'hello': {}", reg.check_answer("echo", &inst, "hello").unwrap());
}
