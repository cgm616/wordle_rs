#![doc = include_str!("../README.md")]

use thiserror::Error;

/// Tools for defining Wordle strategies.
pub mod strategy;
pub use strategy::Strategy;

/// The wordlists used by Wordle.
pub mod words;

/// The test harness for running Wordle strategies.
pub mod harness;
pub use harness::Harness;

/// Evaluating and comparing strategies.
pub mod perf;
pub use perf::{Perf, PerfSummary};

/// The errors that `wordle_rs` can produce.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum WordleError {
    /// The index provided when constructing a Wordle word does not correspond
    /// to a Wordle word.
    #[error("the index {0} does not correspond to a possible Wordle word")]
    InvalidIndex(usize),

    /// The string provided when constructing a Wordle word is not a valid
    /// Wordle word.
    #[error("the string \"{0}\" is not in the Wordle wordlist")]
    NotInWordlist(String),
}
