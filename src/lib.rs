#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

// Required to rename serde
#[cfg(feature = "serde")]
extern crate serde_crate as serde;

use std::error::Error as StdError;

use thiserror::Error;

pub mod strategy;
#[doc(inline)]
pub use strategy::{Attempts, AttemptsKey, Grade, Puzzle, Strategy, Word};

pub mod words;

#[cfg(not(target_family = "wasm"))]
pub mod harness;
#[doc(inline)]
#[cfg(not(target_family = "wasm"))]
pub use harness::{Harness, Record};

#[cfg(not(target_family = "wasm"))]
pub mod perf;
#[doc(inline)]
#[cfg(not(target_family = "wasm"))]
pub use perf::{Comparison, Perf, PrintOptions, Summary};

#[cfg(all(feature = "stats", not(target_family = "wasm")))]
mod stats;

#[cfg(test)]
mod mock;

/// A convenient redefinition of [`std::result::Result`] that uses [`WordleError`]
/// as the error type.
pub type Result<T> = std::result::Result<T, WordleError>;

/// The errors that `wordle_rs` can produce.
#[derive(Debug, Error)]
pub enum WordleError {
    /// An error belonging to the part of this crate used to implement
    /// strategies.
    #[error(transparent)]
    Puzzle {
        /// The kind of error reached.
        #[from]
        kind: PuzzleError,
    },

    #[cfg(not(target_family = "wasm"))]
    /// Could not print.
    #[error("IO error while printing")]
    Printing(#[from] std::io::Error),

    #[cfg(not(target_family = "wasm"))]
    /// Attempted to compare a strategy with itself.
    #[error("cannot compare a strategy with itself")]
    SelfComparison,

    #[cfg(not(target_family = "wasm"))]
    /// Attempted to run stats on bad performance data.
    #[error("can not run stats on this data")]
    Stats,

    #[cfg(not(target_family = "wasm"))]
    /// An error belonging to the part of this crate used to run strategies
    /// (i.e. the test harness).
    #[error(transparent)]
    Harness {
        /// The kind of error reached.
        #[from]
        kind: HarnessError,
    },
}

/// The errors that the "puzzle" side of this crate can produce.
///
/// This type can be wrapped in a [`WordleError`] with the
/// [`Puzzle`](WordleError::Puzzle) variant, and that is often how
/// consumers of this crate will find it.
///
/// # Examples
/// ```
/// # use wordle_rs::{PuzzleError, WordleError};
/// let error = PuzzleError::OutOfGuesses;
/// let wrapped: WordleError = error.into();
/// ```
#[derive(Debug, Error)]
pub enum PuzzleError {
    /// The index provided when constructing a Wordle word does not correspond
    /// to a Wordle word.
    #[error("the index {0} does not correspond to a possible Wordle word")]
    InvalidIndex(usize),

    /// The string provided when constructing a Wordle word is not a valid
    /// Wordle word.
    #[error("the string \"{0}\" is not in the Wordle wordlist")]
    NotInWordlist(String),

    /// The puzzle has already evaluated six guesses.
    #[error("the puzzle has already evaluated six guesses")]
    OutOfGuesses,

    /// The word provided to [`Puzzle::check()`](strategy::Puzzle::check())
    /// does not follow Wordle hardmode rules.
    #[error("that guess does not follow hardmode rules")]
    InvalidHardmodeGuess,
}

/// The errors that the "harness" side of this crate can produce.
///
/// This type can be wrapped in a [`WordleError`] with the
/// [`Puzzle`](WordleError::Harness) variant, and this is often how
/// consumers of this crate will find it.
///
/// # Examples
/// ```
/// # use wordle_rs::{HarnessError, WordleError};
/// let error = HarnessError::BaselineAlreadySet;
/// let wrapped: WordleError = error.into();
/// ```
#[cfg(not(target_family = "wasm"))]
#[derive(Debug, Error)]
pub enum HarnessError {
    /// The test harness already has a baseline.
    #[error("test harness already has a baseline")]
    BaselineAlreadySet,

    /// The test harness could not find and deserialize a baseline file
    /// with the specified name.
    #[error("could not read or write baseline file")]
    BaselineRead(#[source] Box<dyn StdError + Send>),

    /// The test harness could not write the strategy records to disk.
    #[cfg(feature = "serde")]
    #[error("could not write summary to disk")]
    SummaryWrite(#[source] Box<dyn StdError + Send>),

    /// The test harness cannot run without adding at least one strategy.
    #[error("no strategies have been added to the harness")]
    NoStrategiesAdded,

    /// The test harness cannot run on zero words.
    #[error("test harness configured to run on 0 words")]
    NoWordsSelected,

    /// A strategy created an unauthorized instance of [`Attempts`] and used it
    /// to gain more information about its puzzle.
    #[error("the strategy {0} cheated")]
    StrategyCheated(String),

    #[error("could not load, compile, or instantiate wasm module:\n{0}")]
    Wasm(#[source] Box<dyn StdError + Send>),
}
