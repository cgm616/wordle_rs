//! Tools for defining Wordle strategies.

use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use serde::{Deserialize, Serialize};

use crate::{words::GUESSES, WordleError};

/// A Wordle word.
///
/// This struct represents a possible Wordle guess, and its construction
/// is validated to ensure that every instance is a possible word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct Word {
    index: usize,
}

impl Word {
    /// Creates a new [`Word`] from an index into [`GUESSES`](crate::words::GUESSES).
    ///
    /// Returns an error if the index provided is out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::ops::Deref;
    /// # use wordle_rs::{strategy::Word, words::GUESSES};
    /// #
    /// let aahed = Word::from_index(0)?;
    /// assert_eq!(aahed.deref(), "aahed");
    ///
    /// assert!(Word::from_index(GUESSES.len()).is_err());
    /// #
    /// # Ok::<_, wordle_rs::WordleError>(())
    /// ```
    pub fn from_index(index: usize) -> Result<Self, WordleError> {
        if index < GUESSES.len() {
            Ok(Word { index })
        } else {
            Err(WordleError::InvalidIndex(index))
        }
    }

    /// Creates a new [`Word`] from a a five letter string.
    ///
    /// Returns an error if the string provided is not a valid Wordle word.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::ops::Deref;
    /// # use wordle_rs::strategy::Word;
    /// #
    /// let pints = Word::from_str("pints")?;
    /// assert_eq!(pints.deref(), "pints");
    ///
    /// assert!(Word::from_str("tlamp").is_err());
    /// #
    /// # Ok::<_, wordle_rs::WordleError>(())
    /// ```
    pub fn from_str(word: &str) -> Result<Self, WordleError> {
        GUESSES
            .binary_search(&word)
            .map(|index| Word { index })
            .map_err(|_| WordleError::NotInWordlist(word.to_string()))
    }
}

impl Deref for Word {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        crate::words::GUESSES[self.index]
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.deref())
    }
}

/// A specific Wordle puzzle to solve.
///
/// Implementers of [`Strategy`] receive an instance of this struct
/// in the [`solve()`](Strategy::solve()) function. You can use it to check
/// whether or not a guess is correct, and it will return all of the partial
/// information that Wordle provides.
///
/// # Examples
///
/// Here, we create a new puzzle and solve it with a strategy from
/// [`wordle_strategies`]().
///
/// ```rust
/// # use wordle_strategies::Basic;
/// # use wordle_rs::strategy::Word;
/// use wordle_rs::{strategy::Puzzle, Strategy};
///
/// let puzzle = Puzzle::new(Word::from_str("earth")?);
/// let strategy = Basic::new();
/// let attempts = strategy.solve(&puzzle);
/// #
/// # Ok::<_, wordle_rs::WordleError>(())
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Copy)]
pub struct Puzzle {
    word: Word,
}

impl Puzzle {
    /// Creates a new puzzle from a [`Word`].
    pub fn new(word: Word) -> Self {
        Puzzle { word }
    }

    /// Checks if a guess is correct and returns partial information.
    ///
    /// This function checks the `guess` parameter against the puzzle word
    /// and returns a tuple containing five [`Grade`]s and a [`bool`]. The
    /// bool denotes whether or not the guess is correct, and the grades
    /// provide information about how correct each letter in the guess is.
    /// The array grades each letter of the guess in order, so the first element
    /// corresponds to the first letter of the guess, the second to the second
    /// letter, etc.
    ///
    /// In the case that a guess contains two or more of the same letter,
    /// the following is true:
    ///
    /// 1. The function will return [`Grade::Correct`] for all of those letters
    ///    in the correct position.
    /// 2. The function will not return more copies of [`Grade::Correct`] and
    ///    [`Grade::Almost`] together than the copies of that letter in the
    ///    answer. For instance, if the answer is `sober` and you guess
    ///    `spool`, this function will provide [`Grade::Almost`] for the first
    ///    `o` and [`Grade::Incorrect`] for the second. If you then guess
    ///    `soaks`, the first `s` will receive [`Grade::Correct`] and the
    ///    second will receive [`Grade::Incorrect`].
    ///
    /// The function also updates the [`Attempts`] struct used to track
    /// a strategy's guesses.
    ///
    /// When `hardmode` is `true`, this function returns an error if `guess`
    /// does use all of the information previously provided. If `hardmode` is false,
    /// this function always returns `Ok(_)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use wordle_rs::strategy::Puzzle;
    /// use wordle_rs::strategy::{Word, Attempts, Grade::*};
    ///
    /// let puzzle = Puzzle::new(Word::from_str("earth")?);
    /// let mut attempts = Attempts::new();
    ///
    /// // The first guess always succeeds.
    /// let (grades, correct) = puzzle
    ///     .check(&Word::from_str("ratio")?, &mut attempts, true)
    ///     .unwrap();
    /// assert!(!correct);
    /// assert_eq!(grades, [Almost, Correct, Almost, Incorrect, Incorrect]);
    /// assert_eq!(attempts.inner().len(), 1);
    ///
    /// // This guess does not incorporate all of the information, so it should fail!
    /// assert!(puzzle.check(&Word::from_str("trick")?, &mut attempts, true).is_err());
    /// assert_eq!(attempts.inner().len(), 1);
    ///
    /// // Guessing the same word on easymode is okay.
    /// let (grades, correct) = puzzle
    ///     .check(&Word::from_str("trick")?, &mut attempts, false)
    ///     .unwrap();
    /// assert!(!correct);
    /// assert_eq!(grades, [Almost, Almost, Incorrect, Incorrect, Incorrect]);
    /// assert_eq!(attempts.inner().len(), 2);
    ///
    /// # Ok::<_, wordle_rs::WordleError>(())
    /// ```
    pub fn check(
        &self,
        guess: &Word,
        attempts: &mut Attempts,
        hardmode: bool,
    ) -> Result<([Grade; 5], bool), ()> {
        if let Err(_) = attempts.push(*guess) {
            return Err(());
        }

        let mut res = [Grade::Incorrect; 5];
        let mut correct = true;
        for (i, (guess, answer)) in guess.chars().zip(self.word.chars()).enumerate() {
            if guess == answer {
                res[i] = Grade::Correct;
            } else if self.word.contains(guess) {
                res[i] = Grade::Almost;
                correct = false;
            } else {
                correct = false;
            }
        }

        // TODO: follow hardmode rules
        // TODO: fix multiple letter problems (see docs and tests)

        Ok((res, correct))
    }
}

/// A Wordle "grade" that indicates the correctness of a letter in a guess.
///
/// The [`Puzzle::check()`] function returns an array of five of these, one
/// corresponding to each letter in the guess, to indicate how correct
/// each letter is. `Correct` means that the letter is in the correct position.
/// `Almost` means that the letter is in the word, but not in that position.
/// `Incorrect` means that the word does not contain that letter.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Grade {
    /// A grade that indicates the letter guessed is in the correct position.
    Correct,

    /// A grade that indicates the letter guessed is in the word, but not there.
    Almost,

    /// A grade that indicates the letter guesses is not in the word.
    Incorrect,
}

/// A collection of attempts to solve a Wordle puzzle.
///
/// Strategies must return this, and the struct simply wraps a [`Vec`] to
/// ensure that strategies cannot inflate their performance. While implementing
/// [`Strategy::solve()`], you must create an instance of [`Attempts`] and
/// pass a mutable reference to it to [`Puzzle::check()`], which updates it.
///
/// There are some helper methods to make stopping your algorithms possible.
///
/// # Examples
///
/// ```rust
/// # use std::ops::Deref;
/// # use wordle_rs::strategy::Attempts;
/// use wordle_rs::strategy::{Puzzle, Word};
///
/// let mut attempts = Attempts::new();
/// let mut puzzle = Puzzle::new(Word::from_str("limit").unwrap());
/// let (_, _) = puzzle
///     .check(&Word::from_str("tithe").unwrap(), &mut attempts, true)
///     .unwrap();
///
/// assert_eq!(attempts.inner().len(), 1);
/// assert_eq!(attempts.inner()[0].deref(), "tithe");
/// assert!(!attempts.finished());
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Attempts {
    inner: Vec<Word>,
}

impl Attempts {
    /// Creates a new [`Attempts`].
    pub fn new() -> Self {
        Attempts { inner: Vec::new() }
    }

    /// Adds an attempt to an [`Attempts`].
    ///
    /// This will return an error if `inner` already has six elements.
    /// Otherwise, this function will succeed.
    pub(crate) fn push(&mut self, word: Word) -> Result<usize, Word> {
        if self.inner.len() < 6 {
            self.inner.push(word);
            Ok(self.inner.len() - 1)
        } else {
            Err(word)
        }
    }

    /// Returns a slice into the underlying data.
    pub fn inner(&self) -> &[Word] {
        self.inner.as_slice()
    }

    /// Returns true if this instance is full and false otherwise.
    ///
    /// An instance of [`Attempts`] is full when it has been used for six
    /// guesses.
    pub fn finished(&self) -> bool {
        self.inner.len() >= 6
    }

    /// Returns true if the last word in this attempt list matches `word`.
    pub(crate) fn solved(&self, word: &Word) -> bool {
        match self.inner().last() {
            Some(s) if s == word => true,
            _ => false,
        }
    }
}

impl Display for Attempts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((last, rest)) = self.inner.split_last() {
            for word in rest {
                writeln!(f, "{}", word)?;
            }
            write!(f, "{}", last)?;
        }
        Ok(())
    }
}

/// Trait defining a Wordle strategy.
///
/// To write a strategy, define a new struct and implement this trait on it.
///
/// # How to implement
///
/// First, you want to make a new struct and implement [`Display`] on it.
/// The test harness will use [`Display`] to format the name of the strategy,
/// so do not use linebreaks.
///
/// ```rust
/// use std::fmt::Display;
///
/// #[derive(Debug)]
/// struct MyCoolStrategy;
///
/// impl Display for MyCoolStrategy {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "MyCoolStrategy")
///     }
/// }
/// ```
///
/// Then, implement [`Strategy`]. The [`version()`](Strategy::version())
/// and [`hardmode()`](Strategy::hardmode()) functions are relatively easy.
///
/// ``` rust
/// # use std::fmt::Display;
/// # use wordle_rs::{Strategy, strategy::{Puzzle, Attempts}};
/// #
/// # #[derive(Debug)]
/// # struct MyCoolStrategy;
/// #
/// # impl Display for MyCoolStrategy {
/// #     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// #         write!(f, "MyCoolStrategy")
/// #     }
/// # }
///
/// impl Strategy for MyCoolStrategy {
///     fn version(&self) -> &'static str {
///         "0.1.0"
///     }
///
///     fn hardmode(&self) -> bool {
///         true
///     }
/// #
/// #    fn solve(&self, puzzle: &Puzzle) -> Attempts {
/// #        todo!()
/// #    }
/// }
/// ```
///
/// Finally, implement [`solve()`](Strategy::solve()). To keep produce an
/// [`Attempts`] struct to return, create a new one at the very beginning of
/// your implementation.
///
/// ``` rust
/// # use std::fmt::Display;
/// # use wordle_rs::Strategy;
/// #
/// # #[derive(Debug)]
/// # struct MyCoolStrategy;
/// #
/// use wordle_rs::strategy::{Puzzle, Attempts, Word};
///
/// # impl Display for MyCoolStrategy {
/// #     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// #         write!(f, "MyCoolStrategy")
/// #     }
/// # }
/// #
/// # impl Strategy for MyCoolStrategy {
/// #     fn version(&self) -> &'static str {
/// #         "0.1.0"
/// #     }
/// #
/// #     fn hardmode(&self) -> bool {
/// #         true
/// #     }
/// #
/// #
/// fn solve(&self, puzzle: &Puzzle) -> Attempts {
///     let mut attempts = Attempts::new();
///     while !attempts.finished() {
///         // Make guesses!
///         let (_, _) = puzzle.check(&Word::from_str("tithe").unwrap(), &mut attempts, false).unwrap();
///     }
///     attempts
/// }
/// # }
/// ```
pub trait Strategy: Display + Debug + Sync {
    /// Tries to solve the given [`Puzzle`] and returns a list of attempts.
    ///
    /// This is the main function to implement in this trait. The list of
    /// attempts is managed by the `puzzle` parameter.
    fn solve(&self, puzzle: &Puzzle) -> Attempts;

    /// Provides a version for this strategy.
    ///
    /// You should ensure that this changes each time you update the logic of
    /// the strategy in order to produce meaningful comparisons. The value of
    /// this function for a particular instance of the strategy
    /// should not change it is configured.
    fn version(&self) -> &'static str;

    /// Describes if this strategy should be run on hardmode or easymode.
    ///
    /// This is not currently used, but in the future [`Puzzle`] will use this
    /// to enforce hardmode rules on strategies that say they want them.
    /// The value of this function should not change for a particular instance
    /// of the strategy after it is configured.
    fn hardmode(&self) -> bool;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::WordleError;

    #[test]
    fn repeat_letter_guesses() -> Result<(), WordleError> {
        use Grade::*;

        let mut attempts = Attempts::new();
        let puzzle = Puzzle::new(Word::from_str("sober")?);

        let (grades, correct) = puzzle
            .check(&Word::from_str("spool")?, &mut attempts, true)
            .unwrap();
        assert!(!correct);
        assert_eq!(grades, [Correct, Incorrect, Almost, Incorrect, Incorrect]);

        let (grades, correct) = puzzle
            .check(&Word::from_str("soaks")?, &mut attempts, true)
            .unwrap();
        assert!(!correct);
        assert_eq!(grades, [Correct, Correct, Incorrect, Incorrect, Incorrect]);

        Ok(())
    }

    #[test]
    fn repeat_letter_answer() -> Result<(), WordleError> {
        use Grade::*;

        let mut attempts = Attempts::new();
        let puzzle = Puzzle::new(Word::from_str("spoon")?);

        let (grades, correct) = puzzle
            .check(&Word::from_str("odors")?, &mut attempts, true)
            .unwrap();
        assert!(!correct);
        assert_eq!(grades, [Almost, Incorrect, Correct, Incorrect, Almost]);

        Ok(())
    }
}
