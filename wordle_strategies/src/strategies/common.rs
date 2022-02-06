use std::fmt::Display;

use itertools::Itertools;
use lazy_static::lazy_static;

use wordle_rs::{
    strategy::{Attempts, AttemptsKey, Strategy, Word},
    wrappable,
};

use crate::util::{occurrences, Information};

/// A hardmode Wordle strategy that uses pre-computed letter counts
/// to make better guesses.
///
/// Each round, it will guess the next word that could be the answer (given
/// what it knows) containing the most common letters. It is essentially the
/// same as [Basic](crate::Basic) but a little smarter.
///
/// This strategy has no configuration, so you can simply instantiate it
/// with its name:
///
/// ```rust
/// # use wordle_strategies::Common;
/// let strategy = Common;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[wrappable(new = new, name = common)]
pub struct Common;

impl Common {
    /// Creates a new instance.
    pub const fn new() -> Self {
        Common
    }
}

impl Strategy for Common {
    fn solve(&self, puzzle: &mut wordle_rs::strategy::Puzzle, key: AttemptsKey) -> Attempts {
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

        let mut attempts = key.unlock();
        let mut info = Information::new();

        while !attempts.finished() {
            let regex = info.hardmode_regex();
            let guess = Word::from_str(
                SORTED
                    .iter()
                    .filter(|s| regex.is_match(s.as_bytes()))
                    .find(|s| {
                        let mut works = true;

                        for (d, _) in info.almost.iter() {
                            if !s.contains(*d) {
                                works = false;
                                break;
                            }
                        }

                        works
                    })
                    .expect("some word should work!"),
            )
            .unwrap();

            let (grades, got_it) = puzzle.check(&guess, &mut attempts).unwrap();
            if got_it {
                break;
            }
            info.update(&guess, &grades);
        }

        attempts
    }

    fn version(&self) -> &'static str {
        "0.1.2"
    }

    fn hardmode(&self) -> bool {
        true
    }
}

impl Display for Common {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wordle_strategies::Common")
    }
}
