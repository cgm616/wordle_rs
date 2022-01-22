use std::collections::HashMap;
use std::fmt::Display;

use itertools::Itertools;
use lazy_static::lazy_static;

use wordle_perf::strategy::{Attempts, Grade, Strategy, Word};

use crate::util::{generate_regex, occurrences, Information};

/// A hardmode Wordle strategy that uses pre-computed letter counts
/// to make better guesses.
///
/// Each round, it will guess the next word that could be the answer (given
/// what it knows) containing the most common letters. It is essentially the
/// same as [Basic](crate::Basic) but a little smarter.
#[derive(Debug)]
pub struct Common;

impl Strategy for Common {
    fn solve(&self, puzzle: &wordle_perf::strategy::Puzzle) -> wordle_perf::strategy::Attempts {
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

        while !attempts.finished() {
            let regex = info.hardmode_regex();
            let guess = Word::from_str(
                SORTED
                    .iter()
                    .enumerate()
                    .filter(|(i, s)| regex.is_match(s.as_bytes()))
                    .filter(|(_, s)| {
                        let mut works = true;

                        for (d, _) in info.almost.iter() {
                            if !s.contains(*d) {
                                works = false;
                                break;
                            }
                        }

                        works
                    })
                    .next()
                    .map(|(_, s)| *s)
                    .expect("some word should work!"),
            )
            .unwrap();

            let (grades, got_it) = puzzle.check(&guess, &mut attempts, true).unwrap();
            if got_it {
                break;
            }
            info.update(&guess, &grades);
        }

        attempts
    }

    fn version(&self) -> &'static str {
        "0.1.1"
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
