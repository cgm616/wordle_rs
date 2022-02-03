//! Tools for defining Wordle strategies.

use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use itertools::Itertools;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    words::GUESSES,
    {PuzzleError, Result, WordleError},
};

pub mod stupid;

/// A Wordle word.
///
/// This struct represents a possible Wordle guess, and its construction
/// is validated to ensure that every instance is a possible word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
pub struct Word {
    pub(crate) index: usize,
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
    pub fn from_index(index: usize) -> Result<Self> {
        if index < GUESSES.len() {
            Ok(Word { index })
        } else {
            Err(PuzzleError::InvalidIndex(index).into())
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
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(word: &str) -> Result<Self> {
        GUESSES
            .binary_search(&word)
            .map(|index| Word { index })
            .map_err(|_| PuzzleError::NotInWordlist(word.to_string()).into())
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
/// When an [`Attempts`] created with the [`cheat()`](Attempts::cheat())
/// function is passed to [`check()`](Puzzle::check()), the puzzle will
/// become "poisoned." The [test harness](crate::Harness) checks for this
/// and will refuse to produce performance results for a strategy that has
/// passed such an instance to its puzzle.
///
/// # Examples
///
/// Here, we create a new puzzle and solve it with a bad strategy.
///
/// ```rust
/// # use wordle_rs::strategy::Word;
/// use wordle_rs::{strategy::{stupid::Stupid, Puzzle, AttemptsKey}, Strategy};
///
/// let mut puzzle = Puzzle::new(Word::from_str("earth")?);
/// let key = AttemptsKey::new_cheat(false);
/// let strategy = Stupid;
///
/// let attempts = strategy.solve(&mut puzzle, key);
/// #
/// # Ok::<_, wordle_rs::WordleError>(())
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Puzzle {
    word: Word,
    pub(crate) poisoned: bool,
}

impl Puzzle {
    /// Creates a new puzzle from a [`Word`].
    pub fn new(word: Word) -> Self {
        Puzzle {
            word,
            poisoned: false,
        }
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
    /// The function also updates `attempts`. If it is full, this function
    /// returns an error.
    ///
    /// When the strategy reports that it runs on hardmode, this function also
    /// returns an error if `guess` does use all of the information previously
    /// provided.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use wordle_rs::strategy::Puzzle;
    /// use wordle_rs::strategy::{Word, Attempts, Grade::*};
    ///
    /// let mut puzzle = Puzzle::new(Word::from_str("earth")?);
    /// let mut attempts = Attempts::cheat(true);
    ///
    /// // The first guess always succeeds.
    /// let (grades, correct) = puzzle
    ///     .check(&Word::from_str("ratio")?, &mut attempts)
    ///     .unwrap();
    /// assert!(!correct);
    /// assert_eq!(grades, [Almost, Correct, Almost, Incorrect, Incorrect]);
    /// assert_eq!(attempts.inner().len(), 1);
    ///
    /// // This guess does not incorporate all of the information, so it should fail!
    /// assert!(puzzle.check(&Word::from_str("trick")?, &mut attempts).is_err());
    /// assert_eq!(attempts.inner().len(), 1);
    ///
    /// // Attempting the same sequence of guesses is okay on easymode.
    /// let mut attempts = Attempts::cheat(false);
    /// let _ = puzzle.check(&Word::from_str("ratio")?, &mut attempts).unwrap();
    /// let _ = puzzle.check(&Word::from_str("trick")?, &mut attempts).unwrap();
    /// #
    /// # Ok::<_, wordle_rs::WordleError>(())
    /// ```
    pub fn check(&mut self, guess: &Word, attempts: &mut Attempts) -> Result<([Grade; 5], bool)> {
        if attempts.cheat {
            self.poisoned = true;
        }

        if attempts.hard {
            for previous in attempts.inner().iter().rev() {
                let (previous_grades, _) = self.check_inner(previous);
                self.hardmode_guard(previous, &previous_grades, guess)?;
            }
        }

        if attempts.push(*guess).is_err() {
            return Err(PuzzleError::OutOfGuesses.into());
        }

        Ok(self.check_inner(guess))
    }

    fn check_inner(&self, guess: &Word) -> ([Grade; 5], bool) {
        use std::cmp::Ordering;

        let mut used = String::new();
        let mut res = [Grade::Incorrect; 5];
        let mut correct = true;

        // go through correct letters first, since those get priority
        for (i, (guess, answer)) in guess
            .chars()
            .zip(self.word.chars())
            .enumerate()
            .sorted_unstable_by(|&(a_i, (a_guess, a_answer)), &(b_i, (b_guess, b_answer))| {
                let a_correct = a_guess == a_answer;
                let b_correct = b_guess == b_answer;
                match a_correct.cmp(&b_correct).reverse() {
                    Ordering::Equal => a_i.cmp(&b_i),
                    other => other,
                }
            })
        {
            if guess == answer {
                used.push(guess);
                res[i] = Grade::Correct;
            } else {
                match self.word.chars().filter(|&c| c == guess).count() {
                    0 => correct = false,
                    n if n >= 1 && used.chars().filter(|&c| c == guess).count() < n => {
                        used.push(guess);
                        res[i] = Grade::Almost;
                        correct = false;
                    }
                    _ => correct = false,
                }
            }
        }

        (res, correct)
    }

    fn hardmode_guard(&self, previous: &Word, grades: &[Grade], guess: &Word) -> Result<()> {
        // We need to check that `guess` incorporates all _revealed_ guesses.
        // That means that it uses the all of the almosts and correctly uses
        // all of the corrects.
        let mut almost_lookup = [0_u8; 26];
        const A_ASCII: usize = 0x61;
        let i = |c: char| c as usize - A_ASCII;

        for (prev, grade, new) in previous
            .chars()
            .zip(grades.iter())
            .zip(guess.chars())
            .map(|c| (c.0 .0, c.0 .1, c.1))
            .sorted_unstable_by_key(|c| c.1)
        {
            match grade {
                Grade::Correct => {
                    // make sure prev == new since they know where this letter goes
                    if prev != new {
                        return Err(PuzzleError::InvalidHardmodeGuess.into());
                    }
                }
                Grade::Incorrect => {}
                Grade::Almost => {
                    // make sure that enough of this letter are in the word
                    almost_lookup[i(prev)] += 1;
                    if guess.chars().filter(|&c| c == prev).count()
                        < almost_lookup[i(prev)] as usize
                    {
                        return Err(PuzzleError::InvalidHardmodeGuess.into());
                    }
                }
            }
        }

        Ok(())
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

/// A key provided to [`Strategy::solve()`] to produce [`Attempts`].
///
/// This exists to allow strategies to produce only one instance of
/// [`Attempts`] while running.
pub struct AttemptsKey {
    hard: bool,
    cheat: bool,
}

impl AttemptsKey {
    pub(crate) fn new(hard: bool) -> AttemptsKey {
        AttemptsKey { hard, cheat: false }
    }

    pub fn new_cheat(hard: bool) -> AttemptsKey {
        AttemptsKey { hard, cheat: true }
    }

    /// Use the key to produce an instance of [`Attempts`].
    pub fn unlock(self) -> Attempts {
        Attempts::new(self.hard, self.cheat)
    }
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
/// let mut attempts = Attempts::cheat(true);
/// let mut puzzle = Puzzle::new(Word::from_str("limit").unwrap());
/// let (_, _) = puzzle
///     .check(&Word::from_str("tithe").unwrap(), &mut attempts)
///     .unwrap();
///
/// assert_eq!(attempts.inner().len(), 1);
/// assert_eq!(attempts.inner()[0].deref(), "tithe");
/// assert!(!attempts.finished());
/// ```
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Attempts {
    inner: Vec<Word>,
    pub(crate) hard: bool,
    pub(crate) cheat: bool,
}

impl Attempts {
    /// Creates a new [`Attempts`].
    pub(crate) fn new(hard: bool, cheat: bool) -> Self {
        Attempts {
            hard,
            cheat,
            ..Self::default()
        }
    }

    /// Creates a new [`Attempts`] for use other than in a strategy.
    ///
    /// Passing an instance created this way to [`Puzzle::check()`] will poison
    /// the puzzle, so do not use do that inside [`Strategy::solve()`]!
    pub fn cheat(hard: bool) -> Self {
        Attempts {
            hard,
            cheat: true,
            ..Self::default()
        }
    }

    /// Adds an attempt to an [`Attempts`].
    ///
    /// This will return an error if `inner` already has six elements.
    /// Otherwise, this function will succeed.
    pub(crate) fn push(&mut self, word: Word) -> Result<usize> {
        if self.inner.len() < 6 {
            self.inner.push(word);
            Ok(self.inner.len() - 1)
        } else {
            Err(PuzzleError::OutOfGuesses.into())
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
        matches!(self.inner().last(), Some(s) if s == word)
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
/// # use wordle_rs::{Strategy, strategy::{Puzzle, Attempts, AttemptsKey}};
/// #
/// # #[derive(Debug)]
/// # struct MyCoolStrategy;
/// #
/// # impl Display for MyCoolStrategy {
/// #     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// #         write!(f, "MyCoolStrategy")
/// #     }
/// # }
/// #
/// impl Strategy for MyCoolStrategy {
///     fn version(&self) -> &'static str {
///         "0.1.0"
///     }
///
///     fn hardmode(&self) -> bool {
///         true
///     }
///
///     // snip
/// #
/// #    fn solve(&self, puzzle: &mut Puzzle, key: AttemptsKey) -> Attempts {
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
/// use wordle_rs::strategy::{Puzzle, Attempts, AttemptsKey, Word};
///
/// # impl Display for MyCoolStrategy {
/// #     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// #         write!(f, "MyCoolStrategy")
/// #     }
/// # }
/// #
/// impl Strategy for MyCoolStrategy {
///
///     // snip
/// #     fn version(&self) -> &'static str {
/// #         "0.1.0"
/// #     }
/// #
/// #     fn hardmode(&self) -> bool {
/// #         true
/// #     }
///
///     fn solve(&self, puzzle: &mut Puzzle, key: AttemptsKey) -> Attempts {
///         let mut attempts = key.unlock();
///         while !attempts.finished() {
///             // Make guesses!
///             let (_, _) = puzzle.check(&Word::from_str("tithe").unwrap(), &mut attempts).unwrap();
///         }
///         attempts
///     }
/// }
/// ```
pub trait Strategy: Display + Debug + Sync {
    /// Tries to solve the given [`Puzzle`] and returns a list of attempts.
    ///
    /// This is the main function to implement in this trait. The list of
    /// attempts is managed by the `puzzle` parameter. Use the `key`
    /// parameter to produce the [`Attempts`] to return via
    /// [`AttemptsKey::unlock()`].
    fn solve(&self, puzzle: &mut Puzzle, key: AttemptsKey) -> Attempts;

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

    fn str_to_grades(input: &str) -> [Grade; 5] {
        let mut res = [Grade::Incorrect; 5];
        for (i, c) in input.chars().enumerate() {
            match c {
                'c' => res[i] = Grade::Correct,
                'a' => res[i] = Grade::Almost,
                _ => {}
            }
        }
        res
    }

    macro_rules! puzzle_test {
        (I $answer:expr; $puzzle:ident, $attempts:ident, $count:ident; $guess:expr, $works:expr, $res:expr) => {{
            if $works {
                let (grades, correct) = $puzzle
                    .check(&Word::from_str($guess)?, &mut $attempts)
                    .unwrap();
                $count += 1;
                assert_eq!($attempts.inner().len(), $count);
                assert_eq!(correct, $answer == $guess);
                assert_eq!(grades, str_to_grades($res));
            } else {
                assert!($puzzle
                    .check(&Word::from_str($guess)?, &mut $attempts)
                    .is_err());
            }
        }};

        ($fn_name:ident[hard = $hard:expr, $answer:expr => $( [$guess:expr, $works:expr, $res:expr] );*]) => {
            puzzle_test! { $fn_name [hard = $hard, $answer => $( [$guess, $works, $res] );*] {} }
        };

        ($fn_name:ident[hard = $hard:expr, $answer:expr => $( [$guess:expr, $works:expr, $res:expr] );*] $other:block) => {
            #[test]
            fn $fn_name() -> Result<(), WordleError> {
                let mut attempts = Attempts::cheat($hard);
                let mut puzzle = Puzzle::new(Word::from_str($answer)?);
                let mut count = 0;

                $(puzzle_test!(I $answer; puzzle, attempts, count; $guess, $works, $res);)*

                $other

                Ok(())
            }
        };
    }

    puzzle_test! { repeat_letter_guesses [hard = true, "sober" =>
        ["spool", true, "ciaii"];
        ["soaks", true, "cciii"]]
    }

    puzzle_test! { repeat_letter_guesses_before [hard = true, "tills" =>
        ["pines", true, "iciic"];
        ["sills", true, "icccc"]]
    }

    puzzle_test! { repeat_letter_answer [hard = true, "spoon" =>
        ["odors", true, "aicia"]]
    }

    // A test taken directly from the hardmode behavior of Wordle 218.
    puzzle_test! { wordle_crimp_props_primp [hard = true, "crimp" =>
        ["props", true, "aciii"];
        ["pinup", false, ""];
        ["primp", true, "icccc"];
        ["crimp", true, "ccccc"]]
    }

    // A test taken directly from the hardmode behavior of Wordle 218.
    puzzle_test! { wordle_crimp_error_order_trier [hard = true, "crimp" =>
        ["error", true, "iciii"];
        ["order", true, "iciii"];
        ["right", false, ""];
        ["trier", true, "iccii"];
        ["crimp", true, "ccccc"]]
    }

    // A test taken directly from the hardmode behavior of Wordle 218.
    puzzle_test! { wordle_crimp_lints_limit_minis [hard = true, "crimp" =>
        ["lints", true, "iaiii"];
        ["limit", true, "iaaii"];
        ["lipid", false, ""];
        ["minis", true, "aaiii"];
        ["crimp", true, "ccccc"]]
    }

    // A test taken directly from the hardmode behavior of Wordle 218.
    puzzle_test! { wordle_crimp_bolts_prick [hard = true, "crimp" =>
        ["bolts", true, "iiiii"];
        ["prick", true, "accai"];
        ["crimp", true, "ccccc"]]
    }

    puzzle_test! { filling_up [hard = true, "right" =>
        ["allay", true, "iiiii"];
        ["tough", true, "aiiaa"];
        ["spits", false, ""];
        ["might", true, "icccc"];
        ["night", true, "icccc"];
        ["fight", true, "icccc"];
        ["sight", true, "icccc"]]
    }

    puzzle_test! { repeat_hardmode_almost[hard = true, "spill" =>
        ["alloy", true, "iaaii"];
        ["limes", false, ""];
        ["spilt", false, ""];
        ["level", true, "aiiic"];
        ["petal", false, ""];
        ["spill", true, "ccccc"]]
    }

    puzzle_test! { more_are_allowed[hard = true, "earth" =>
        ["alloy", true, "aiiii"];
        ["drama", true, "iaaii"]]
    }

    puzzle_test! { hardmode_correct[hard = true, "tills" =>
        ["pines", true, "iciic"];
        ["butts", false, ""];
        ["right", false, ""];
        ["earth", false, ""];
        ["mills", true, "icccc"];
        ["tight", false, ""];
        ["tails", false, ""];
        ["sills", true, "icccc"];
        ["tills", true, "ccccc"]]
    }

    puzzle_test! { hardmode_almost[hard = true, "spots" =>
        ["crass", true, "iiiac"];
        ["wisps", true, "iiaac"];
        ["slots", false, ""];
        ["spots", true, "ccccc"]]
    }
}
