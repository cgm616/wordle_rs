#![doc = include_str!("../README.md")]

// Required to rename serde
#[cfg(feature = "serde")]
extern crate serde_crate as serde;

use thiserror::Error;

pub mod strategy;
pub use strategy::Strategy;

pub mod words;

pub mod harness;
pub use harness::Harness;

pub mod perf;
pub use perf::{Perf, Summary};

#[cfg(feature = "stats")]
mod stats;

/// The errors that `wordle_rs` can produce.
#[derive(Debug, Error)]
pub enum WordleError {
    #[error("puzzle encountered error")]
    Puzzle {
        #[from]
        kind: PuzzleError,
    },

    #[error("general IO error")]
    Printing(#[from] std::io::Error),

    #[error("cannot compare a strategy with itself")]
    SelfComparison,

    #[error("the test harness encountered an error")]
    Harness {
        #[from]
        kind: HarnessError,
    },
}

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

    /// The word provided to [`Puzzle::guess()`](strategy::Puzzle::guess())
    /// does not follow Wordle hardmode rules.
    #[error("that guess does not follow hardmode rules")]
    InvalidHardmodeGuess,
}

#[derive(Debug, Error)]
pub enum HarnessError {
    #[error("test harness already has a baseline")]
    BaselineAlreadySet,

    #[error("cannot save baseline unless one is set to run")]
    BaselineNotRun,

    #[error("could not read or write baseline file")]
    BaselineIo(#[from] std::io::Error),

    #[error("a baseline file of that name does not exist")]
    BaselineDoesntExist,

    #[cfg(feature = "serde")]
    #[error("trouble serializing or deserializing baseline")]
    Serde(#[from] serde_json::Error),

    #[error("no strategies have been added to the harness")]
    NoStrategiesAdded,

    /// A strategy created an unauthorized instance of [`Attempts`] and used it
    /// to gain more information about its puzzle.
    #[error("the strategy {0} cheated")]
    StrategyCheated(String),
}
