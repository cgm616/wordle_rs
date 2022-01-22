use std::{fmt::Display, ops::Deref};

use serde::{Deserialize, Serialize};

use crate::{words::GUESSES, WordleError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct Word {
    index: usize,
}

impl Word {
    pub fn new(index: usize) -> Result<Self, WordleError> {
        if index < GUESSES.len() {
            Ok(Word { index })
        } else {
            Err(WordleError::InvalidIndex(index))
        }
    }

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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Copy)]
pub struct Puzzle {
    word: Word,
}

impl Puzzle {
    pub fn new(word: Word) -> Self {
        Puzzle { word }
    }

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

        Ok((res, correct))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Grade {
    Correct,
    Almost,
    Incorrect,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Attempts {
    inner: Vec<Word>,
}

impl Attempts {
    pub fn new() -> Self {
        Attempts { inner: Vec::new() }
    }

    pub(crate) fn push(&mut self, word: Word) -> Result<usize, Word> {
        if self.inner.len() < 6 {
            self.inner.push(word);
            Ok(self.inner.len() - 1)
        } else {
            Err(word)
        }
    }

    pub fn inner(&self) -> &[Word] {
        self.inner.as_slice()
    }

    pub fn finished(&self) -> bool {
        self.inner.len() >= 6
    }

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

pub trait Strategy: Display + Sync {
    fn solve(&self, puzzle: &Puzzle) -> Attempts;
    fn version(&self) -> &'static str;
    fn hardmode(&self) -> bool;
}
