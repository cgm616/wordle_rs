use std::fmt::Display;

use wordle_rs::strategy::{Attempts, Puzzle, Strategy, Word};

/// A Wordle strategy that only ever guesses the first few words in the wordlist.
///
/// This exists to show how [wordle_rs::Strategy] is implemented.
pub struct Stupid;

impl Strategy for Stupid {
    fn solve(&self, puzzle: &Puzzle) -> Attempts {
        let mut attempts = Attempts::new();

        for i in 0..6 {
            let word = Word::new(i).unwrap();
            let (_, correct) = puzzle.check(&word, &mut attempts, false).unwrap();
            if correct {
                break;
            }
        }

        attempts
    }

    fn version(&self) -> &'static str {
        "0.10"
    }

    fn hardmode(&self) -> bool {
        false
    }
}

impl Display for Stupid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wordle_strategies::Stupid")
    }
}
