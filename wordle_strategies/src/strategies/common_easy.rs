use std::fmt::Display;

use itertools::Itertools;
use lazy_static::lazy_static;
use wordle_perf::{strategy::Attempts, Strategy};

use crate::util::{occurrences, Information};

/// An easymode Wordle strategy that works the same way as
/// [Common](crate::Common) but makes calculated easymode guesses.
pub struct CommonEasy;

impl Strategy for CommonEasy {
    fn solve(&self, puzzle: &wordle_perf::strategy::Puzzle) -> Attempts {
        lazy_static! {
            static ref SORTED: Vec<&'static str> = {
                let mut words = Vec::from(wordle_perf::words::GUESSES);
                words.sort_unstable_by_key(|s: &&str| {
                    -s.chars()
                        .unique()
                        .map(|c| occurrences(c) as i32)
                        .sum::<i32>()
                });
                words
            };
        }

        let mut attempts = Attempts::new();
        let mut info = Information::new();

        for i in 0..6 {
            todo!()
        }

        attempts
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
