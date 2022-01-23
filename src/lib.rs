#![doc = include_str!("../README.md")]

use thiserror::Error;

pub mod strategy;
pub use strategy::Strategy;

pub mod words;

pub mod harness;
pub use harness::Harness;

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
