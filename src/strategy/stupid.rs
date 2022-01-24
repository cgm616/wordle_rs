//! A single bad strategy to show how they are written.

use std::fmt::Display;

use crate::strategy::{Attempts, AttemptsKey, Puzzle, Strategy, Word};

/// A Wordle strategy that only ever guesses the first few words in the wordlist.
///
/// This exists to show how [`Strategy`](super::Strategy) is implemented. It
/// is not recommended to run your own strategies against it.
/// For that, check out the [`wordle_strategies`]() crate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stupid;

impl Strategy for Stupid {
    fn solve(&self, puzzle: &mut Puzzle, key: AttemptsKey) -> Attempts {
        let mut attempts = key.unlock();

        for i in 0..6 {
            let word = Word::from_index(i).unwrap();
            let (_, correct) = puzzle.check(&word, &mut attempts).unwrap();
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
