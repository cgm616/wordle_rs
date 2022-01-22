/// Tools for making strategies and some pre-made ones to try.
pub mod strategy;
pub use strategy::Strategy;

/// The wordlists used by Wordle.
pub mod words;

/// Tools for testing and comparing strategies.
pub mod harness;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WError {
    InvalidWord,
    InvalidIndex,
    NotInWordlist,
}
