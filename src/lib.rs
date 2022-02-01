#![doc = include_str!("../README.md")]

use either::Either;
use thiserror::Error;

pub mod strategy;
pub use strategy::Strategy;

pub mod words;

pub mod harness;
pub use harness::Harness;

pub mod perf;
pub use perf::{Perf, Summary};

mod stats;

/// The errors that `wordle_rs` can produce.
#[derive(Debug, Error)]
pub enum WordleError {
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

    /// The word provided to [`Puzzle::guess()`](strategy::Puzzle::guess())
    /// does not follow Wordle hardmode rules.
    #[error("that guess does not follow hardmode rules")]
    InvalidHardmodeGuess,

    /// A strategy created an unauthorized instance of [`Attempts`] and used it
    /// to gain more information about its puzzle.
    #[error("the strategy {0} cheated")]
    StrategyCheated(String),

    #[error("general IO error")]
    Io(#[from] std::io::Error),

    #[error("cannot compare a strategy with itself")]
    SelfComparison,

    #[error("test harness already has a baseline")]
    BaselineAlreadySet,

    #[error("cannot save baseline unless one is set to run")]
    BaselineNotRun,

    #[error("could not read or write baseline file")]
    BaselineFile(#[source] Option<Either<std::io::Error, serde_json::Error>>),

    #[error("no strategies have been added to the harness")]
    NoStrategiesAdded,
}
