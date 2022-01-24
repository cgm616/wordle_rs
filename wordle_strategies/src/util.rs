use std::collections::{BTreeMap, HashMap};

use itertools::Itertools;
use regex::bytes::{Regex, RegexBuilder};

use wordle_rs::strategy::{Grade, Word};

pub fn generate_regex<'a>(
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

pub fn letter_occurrences<'a>(words: impl IntoIterator<Item = &'a str>) -> BTreeMap<char, u32> {
    let mut map = BTreeMap::new();

    words.into_iter().flat_map(|s| s.chars()).for_each(|c| {
        let key = map.entry(c).or_insert(0);
        *key += 1;
    });

    map
}

pub const OCCURRENCES: [u32; 26] = [
    5990, 1627, 2028, 2453, 6662, 1115, 1644, 1760, 3759, 291, 1505, 3371, 1976, 2952, 4438, 2019,
    112, 4158, 6665, 3295, 2511, 694, 1039, 288, 2074, 434,
];

/// Returns the number of times a lowercase ascii letter appears in the Wordle
/// guess wordlist.
///
/// WARNING: this method WILL panic if you do not provide a lowercase ascii
/// alphabetic char.
pub fn occurrences(c: char) -> u32 {
    if c.is_ascii_alphabetic() && c.is_ascii_lowercase() {
        OCCURRENCES[c as usize - 0x61]
    } else {
        panic!("did not provide ascii lowercase letter")
    }
}

#[derive(Default, Debug)]
pub struct Information {
    pub correct: Vec<(usize, char)>,
    pub incorrect: String,
    pub almost: HashMap<char, (u8, u8)>,
}

impl Information {
    pub fn new() -> Self {
        Self::default()
    }

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
                    if !self.almost.contains_key(&c) && !self.incorrect.contains(c) {
                        self.incorrect.push(c)
                    }
                }
            }
        }
    }

    pub fn hardmode_regex(&self) -> Regex {
        generate_regex(&self.correct, &self.incorrect, self.almost.iter())
    }
}
