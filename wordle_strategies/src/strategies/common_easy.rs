use std::fmt::Display;

use itertools::Itertools;
use lazy_static::lazy_static;
use wordle_rs::{strategy::Attempts, Strategy};

use crate::util::occurrences;

/// An easymode Wordle strategy that works the same way as
/// [Common](crate::Common) but makes calculated easymode guesses.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommonEasy;

impl Strategy for CommonEasy {
    fn solve(&self, puzzle: &wordle_rs::strategy::Puzzle) -> Attempts {
        lazy_static! {
            static ref SORTED: Vec<&'static str> = {
                let mut words = Vec::from(wordle_rs::words::GUESSES);
                words.sort_unstable_by_key(|s: &&str| {
                    -s.chars()
                        .unique()
                        .map(|c| occurrences(c) as i32)
                        .sum::<i32>()
                });
                words
            };
        }

        //let mut attempts = Attempts::new();
        //let mut info = Information::new();

        // attempts
        todo!("{:?}", puzzle)
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn hardmode(&self) -> bool {
        false
    }
}

impl Display for CommonEasy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wordle_strategies::CommonEasy")
    }
}
