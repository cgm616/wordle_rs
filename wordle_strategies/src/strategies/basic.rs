use std::{collections::HashMap, fmt::Display};

use wordle_perf::{
    strategy::{Attempts, Grade, Puzzle, Strategy, Word},
    words::GUESSES,
};

use crate::util::{generate_regex, Information};

/// A hardmode Wordle strategy that guesses the first word that could be
/// correct.
///
/// The `Basic` strategy simply looks through the wordlist until it finds
/// a word that could be the correct answer. It then guesses that word,
/// learns new information about the answer, and searches again.
pub struct Basic {
    first_word: Option<Word>,
}

impl Default for Basic {
    fn default() -> Self {
        Basic { first_word: None }
    }
}

impl Basic {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn first_word(self, word: Word) -> Self {
        Basic {
            first_word: Some(word),
            ..self
        }
    }

    pub fn no_first_word(self) -> Self {
        Basic {
            first_word: None,
            ..self
        }
    }
}

impl Strategy for Basic {
    fn solve(&self, puzzle: &Puzzle) -> Attempts {
        let mut attempts = Attempts::new();
        let mut info = Information::new();

        while !attempts.finished() {
            let guess = if self.first_word.is_some() && attempts.inner().len() == 0 {
                self.first_word.clone().unwrap()
            } else {
                let regex = info.hardmode_regex();
                Word::new(
                    GUESSES
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| regex.is_match(s.as_bytes()))
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
                        .nth(0)
                        .map(|(i, _)| i)
                        .expect("some word should work!"),
                )
                .unwrap()
            };

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

impl Display for Basic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wordle_strategies::Basic")?;
        if let Some(word) = &self.first_word {
            write!(f, " (start: {})", word)?;
        }
        Ok(())
    }
}
