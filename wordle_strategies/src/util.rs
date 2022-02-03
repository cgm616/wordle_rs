//! Utilities for building strategies.

use std::collections::HashMap;

use itertools::Itertools;
use regex::bytes::{Regex, RegexBuilder};

use wordle_rs::strategy::{Grade, Word};

fn generate_regex<'a>(
    correct: &[(usize, char)],
    incorrect: &str,
    almost: impl IntoIterator<Item = (&'a char, &'a (u8, u8))> + Clone,
) -> Regex {
    let mut str = String::new();

    for i in 0..5_usize {
        if let Some((_, c)) = correct.iter().find(|(j, _)| *j == i) {
            str.push(*c);
        } else if incorrect.is_empty() {
            str.push_str("[a-z]");
        } else {
            str.push_str("[^");

            str.push_str(incorrect);
            for (d, (loc, _)) in almost.clone() {
                if loc & (1 << i) != 0 {
                    str.push(*d);
                }
            }
            str.push(']');
        }
    }

    let mut rb = RegexBuilder::new(&str);
    rb.unicode(false);

    rb.build().unwrap()
}

const OCCURRENCES: [u32; 26] = [
    5990, 1627, 2028, 2453, 6662, 1115, 1644, 1760, 3759, 291, 1505, 3371, 1976, 2952, 4438, 2019,
    112, 4158, 6665, 3295, 2511, 694, 1039, 288, 2074, 434,
];

/// Returns the number of times a lowercase ascii letter appears in the Wordle
/// guess wordlist.
///
/// # Panics
///
/// This method will panic if `c` is not a lowercase ascii alphabetic char.
pub fn occurrences(c: char) -> u32 {
    if c.is_ascii_alphabetic() && c.is_ascii_lowercase() {
        OCCURRENCES[c as usize - 0x61]
    } else {
        panic!("did not provide ascii lowercase letter")
    }
}

/// A struct that can track the information returned by
/// [`Puzzle::check()`](wordle_rs::Puzzle::check).
#[derive(Default, Debug)]
pub struct Information {
    /// A list of correct letters and their positions.
    pub correct: Vec<(usize, char)>,

    /// A list of letters that are not in the word.
    pub incorrect: String,

    /// A map from letters to the partial information we have about them.
    ///
    /// The key is the letter and the value is a tuple `(loc, count)`, where
    /// `loc` contains information about where we know the letter does not
    /// appear and `count` tells us how many times that letter must
    /// appear in the word.
    ///
    /// # Examples
    ///
    /// If [`Puzzle::check()`](wordle_rs::Puzzle::check) returns
    /// a grade of `Almost` for `'a'` in the first position (say we guessed
    /// "apple"), then this will give the value `(1 << 0, 1)` for the key `'a'`.
    /// You can see this example in the documentation on [`update()`](Information::update).
    pub almost: HashMap<char, (u8, u8)>,
}

impl Information {
    /// Creates a new, empty instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates the information with the grades from a guess.
    ///
    /// # Examples
    ///
    /// ```
    /// # use wordle_rs::{Grade::*, Word};
    /// # use wordle_strategies::util::Information;
    /// let mut info = Information::new();
    ///
    /// let guess = Word::from_str("apple")?;
    /// let grades = [Almost, Almost, Incorrect, Incorrect, Correct];
    ///
    /// info.update(&guess, &grades);
    ///
    /// assert_eq!(info.correct, vec![(4, 'e')]);
    /// assert_eq!(info.incorrect.as_str(), "l");
    /// assert_eq!(info.almost.get(&'a'), Some(&(0b00001, 1)));
    /// assert_eq!(info.almost.get(&'p'), Some(&(0b00110, 1)));
    /// #
    /// # Ok::<_, wordle_rs::WordleError>(())
    /// ```
    pub fn update(&mut self, guess: &Word, grades: &[Grade]) {
        let mut almost_lookup = [0_u8; 26];
        const A_ASCII: usize = 0x61;
        let index = |c: char| c as usize - A_ASCII;

        for (i, (grade, c)) in grades
            .iter()
            .zip(guess.chars())
            .enumerate()
            .sorted_unstable_by_key(|(_, (g, _))| match g {
                Grade::Correct => 1,
                Grade::Almost => 2,
                Grade::Incorrect => 3,
            })
        {
            match grade {
                Grade::Correct => {
                    if !self.correct.iter().any(|&e| e.0 == i) {
                        self.correct.push((i, c));
                    }
                }
                Grade::Almost => {
                    let (loc, count) = self.almost.entry(c).or_insert((0, 0));
                    *loc |= 1 << i;
                    almost_lookup[index(c)] += 1;
                    if *count < almost_lookup[index(c)] {
                        *count = almost_lookup[index(c)];
                    }
                }
                Grade::Incorrect => {
                    if almost_lookup[index(c)] > 0 {
                        let (loc, _) = self.almost.get_mut(&c).unwrap();
                        *loc |= 1 << i;
                    }

                    if !self.almost.contains_key(&c) && !self.incorrect.contains(c) {
                        self.incorrect.push(c)
                    }
                }
            }
        }
    }

    /// Generates a regex that matches words that fill all of the information
    /// provided to this instance EXCEPT the number of times that the word
    /// contains a letter marked `Almost`, since that radically complicates
    /// the regex.
    ///
    /// You can use the `almost` field of this struct to get a count that tells
    /// you how many times `Almost` letters must appear and do the filtering
    /// yourself.
    pub fn hardmode_regex(&self) -> Regex {
        generate_regex(&self.correct, &self.incorrect, self.almost.iter())
    }
}
