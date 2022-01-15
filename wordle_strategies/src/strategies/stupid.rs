use std::fmt::Display;

use wordle_perf::strategy::{Attempts, Puzzle, Strategy, Word};

/// A Wordle strategy that only ever guesses the first few words in the wordlist.
pub struct Stupid;

impl Strategy for Stupid {
    fn solve(&self, puzzle: &Puzzle) -> Attempts {
        let mut attempts = Attempts::new();

        for i in 0..6 {
            let word = Word::from_wordlist(i).unwrap();
            let (_, correct) = puzzle.check(&word, &mut attempts).unwrap();
            if correct {
                break;
            }
        }

        attempts
    }
}

impl Display for Stupid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wordle_strategies::Stupid")
    }
}
