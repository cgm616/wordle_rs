use std::fmt::Display;

use crate::{Attempts, AttemptsKey, Puzzle, Strategy, Word};

#[derive(Debug, Clone)]
pub(crate) struct Mock {
    guesses: Option<Vec<&'static str>>,
}

impl Mock {
    pub(crate) fn new(guesses: impl Into<Option<Vec<&'static str>>>) -> Self {
        Self {
            guesses: guesses.into(),
        }
    }
}

impl Strategy for Mock {
    fn solve(&self, puzzle: &mut Puzzle, key: AttemptsKey) -> Attempts {
        let mut attempts = key.unlock();

        let guesses = match &self.guesses {
            None => &["nerds", "tithe", "doubt", "point", "parka", "sword"],
            Some(v) => v.as_slice(),
        };

        for guess in guesses {
            let (_, correct) = puzzle
                .check(&Word::from_str(guess).unwrap(), &mut attempts)
                .unwrap();
            if correct {
                break;
            }
        }

        attempts
    }

    fn version(&self) -> &'static str {
        "1.2.4"
    }

    fn hardmode(&self) -> bool {
        false
    }
}

impl Display for Mock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mock {:?}", self.guesses)
    }
}
