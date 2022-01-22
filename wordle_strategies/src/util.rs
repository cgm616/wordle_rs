use regex::bytes::{Regex, RegexBuilder};
use std::collections::{BTreeMap, HashMap};
use wordle_perf::strategy::{Grade, Word};

pub fn generate_regex<'a>(
    correct: &[(usize, char)],
    incorrect: &str,
    almost: impl IntoIterator<Item = (&'a char, &'a u8)> + Clone,
) -> Regex {
    let mut str = String::new();

    for i in 0..5_usize {
        if let Some((_, c)) = correct.iter().find(|(j, _)| *j == i) {
            str.push(*c);
        } else {
            if incorrect.is_empty() {
                str.push_str("[a-z]");
            } else {
                str.push_str("[^");

                str.push_str(&incorrect);
                for (d, locator) in almost.clone() {
                    if *locator & (1 << i) != 0 {
                        str.push(*d);
                    }
                }
                str.push_str("]");
            }
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

#[derive(Default)]
pub struct Information {
    pub correct: Vec<(usize, char)>,
    pub incorrect: String,
    pub almost: HashMap<char, u8>,
}

impl Information {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, guess: &Word, grades: &[Grade]) {
        for (i, (grade, c)) in grades.iter().zip(guess.chars()).enumerate() {
            match grade {
                Grade::Correct => self.correct.push((i, c)),
                Grade::Almost => {
                    let locator = self.almost.entry(c).or_insert(0);
                    *locator |= 1 << i;
                }
                Grade::Incorrect => self.incorrect.push(c),
            }
        }
    }

    pub fn hardmode_regex(&self) -> Regex {
        generate_regex(&self.correct, &self.incorrect, self.almost.iter())
    }
}
