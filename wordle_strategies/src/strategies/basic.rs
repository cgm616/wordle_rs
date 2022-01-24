use std::fmt::Display;

use wordle_rs::{
    strategy::{Attempts, AttemptsKey, Puzzle, Strategy, Word},
    words::GUESSES,
};

use crate::util::Information;

/// A hardmode Wordle strategy that guesses the first word that could be
/// correct.
///
/// The `Basic` strategy simply looks through the wordlist until it finds
/// a word that could be the correct answer. It then guesses that word,
/// learns new information about the answer, and searches again.
///
/// This strategy has one configuration option: the first word it uses.
/// By default, it uses "aahed" (the first word in the Wordle wordlist).
/// To create an instance configured this way, you have two options:
///
/// ```rust
/// # use wordle_strategies::Basic;
/// let first = Basic::new();
/// let second = Basic::new().no_first_word();
/// ```
///
/// To set a specific first word, you must pass a
/// [`Word`](wordle_rs::strategy::Word):
///
/// ```rust
/// # use wordle_strategies::Basic;
/// use wordle_rs::strategy::Word;
///
/// let configured = Basic::new().first_word(Word::from_str("tests")?);
/// # Ok::<_, wordle_rs::WordleError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Basic {
    first_word: Option<Word>,
}

impl Basic {
    /// Creates a new instance with default configuration.
    ///
    /// The default option is to use no specific starting word.
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes the strategy guess a specific first word.
    pub fn first_word(self, word: Word) -> Self {
        Basic {
            first_word: Some(word),
        }
    }

    /// Makes the strategy use its default first word, "aahed" (the first
    /// word in the wordlist.)
    pub fn no_first_word(self) -> Self {
        Basic { first_word: None }
    }
}

impl Strategy for Basic {
    fn solve(&self, puzzle: &mut Puzzle, key: AttemptsKey) -> Attempts {
        let mut attempts = key.unlock();
        let mut info = Information::new();

        while !attempts.finished() {
            let guess = if self.first_word.is_some() && attempts.inner().is_empty() {
                self.first_word.unwrap()
            } else {
                let regex = info.hardmode_regex();
                Word::from_index(
                    GUESSES
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| regex.is_match(s.as_bytes()))
                        .find(|(_, s)| {
                            let mut works = true;

                            for (d, (_, count)) in info.almost.iter() {
                                if s.chars().filter(|c| d == c).count() < *count as usize {
                                    works = false;
                                    break;
                                }
                            }

                            works
                        })
                        .map(|(i, _)| i)
                        .expect("some word should work!"),
                )
                .unwrap()
            };

            let (grades, got_it) = puzzle.check(&guess, &mut attempts).expect(&format!(
                "for some reason, made incorrect hardmode guess!\nInformation: {info:?}\nattempts:\n{attempts}\n{guess} <-- bad guess here\n",
            ));
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

impl Display for Basic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wordle_strategies::Basic")?;
        if let Some(word) = &self.first_word {
            write!(f, " (start: {})", word)?;
        }
        Ok(())
    }
}
