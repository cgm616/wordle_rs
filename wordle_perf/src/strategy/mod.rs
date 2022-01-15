use std::{fmt::Display, ops::Deref};

use crate::{words::GUESSES, WError};

pub mod util;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Word {
    Wordlist(usize),
    Other(String),
}

impl Word {
    pub fn from_wordlist(index: usize) -> Result<Self, WError> {
        if index < crate::words::GUESSES.len() {
            Ok(Self::Wordlist(index))
        } else {
            Err(WError::NotInWordlist)
        }
    }

    pub fn from_other_word(word: &str) -> Result<Self, WError> {
        if word.chars().all(|c| c.is_ascii_alphanumeric()) {
            Ok(Self::Other(word.to_ascii_lowercase()))
        } else {
            Err(WError::InvalidWord)
        }
    }
}

impl Deref for Word {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Wordlist(i) => crate::words::GUESSES[*i],
            Self::Other(s) => s.as_str(),
        }
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.deref())
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Puzzle {
    word: Word,
    hardmode: bool,
}

impl Puzzle {
    pub fn new(word: Word, hardmode: bool) -> Self {
        Puzzle { word, hardmode }
    }

    pub fn check(&self, other: &Word, attempts: &mut Attempts) -> Result<([Grade; 5], bool), ()> {
        if let Word::Other(s) = other {
            if !GUESSES.iter().any(|&t| t == s) {
                return Err(());
            }
        }

        if let Err(_) = attempts.push(other.clone()) {
            return Err(());
        }

        let mut res = [Grade::Incorrect; 5];
        let mut correct = true;
        for (i, (guess, answer)) in other.chars().zip(self.word.chars()).enumerate() {
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

    pub(crate) fn destroy(self) -> Word {
        self.word
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Grade {
    Correct,
    Almost,
    Incorrect,
}

#[derive(Clone, Debug)]
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
        for word in self.inner() {
            writeln!(f, "{}", word)?;
        }

        Ok(())
    }
}

pub trait Strategy: Display {
    fn solve(&self, puzzle: &Puzzle) -> Attempts;
}
