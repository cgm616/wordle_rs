use std::{collections::HashMap, fmt::Display};

use regex::Regex;

use wordle_perf::{
    strategy::{Attempts, Grade, Puzzle, Strategy, Word},
    words::GUESSES,
};

const RATIO: usize = 8930;

/// A Wordle strategy that guesses the first word that could be correct.
///
/// The `Basic` strategy simply looks through the wordlist until it finds
/// a word that could be the correct answer. It then guesses that word,
/// learns new information about the answer, and searches again.
pub struct Basic {
    good_start: bool,
}

impl Default for Basic {
    fn default() -> Self {
        Basic { good_start: true }
    }
}

impl Basic {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn good_start(self) -> Self {
        Basic {
            good_start: true,
            ..self
        }
    }

    pub fn bad_start(self) -> Self {
        Basic {
            good_start: false,
            ..self
        }
    }
}

fn generate_regex(correct: &[(usize, char)], incorrect: &str, almost: &HashMap<char, u8>) -> Regex {
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
                for (d, locator) in almost.iter() {
                    if *locator & (1 << i) != 0 {
                        str.push(*d);
                    }
                }
                str.push_str("]");
            }
        }
    }

    Regex::new(&str).unwrap()
}

impl Strategy for Basic {
    fn solve(&self, puzzle: &Puzzle) -> Attempts {
        let mut attempts = Attempts::new();
        let mut correct = Vec::new();
        let mut incorrect = String::new();
        let mut almost: HashMap<char, u8> = HashMap::new();

        while !attempts.finished() {
            let guess = if self.good_start && attempts.inner().len() == 0 {
                Word::from_wordlist(RATIO).unwrap()
            } else {
                let regex = generate_regex(&correct, &incorrect, &almost);
                Word::from_wordlist(
                    GUESSES
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| regex.is_match(s))
                        .filter(|(_, s)| {
                            let mut works = true;

                            for (d, _) in almost.iter() {
                                if !s.contains(*d) {
                                    works = false;
                                    break;
                                }
                            }

                            works
                        })
                        .nth(0)
                        .map(|(i, _)| i)
                        .expect("some work should work!"),
                )
                .unwrap()
            };

            let (grades, got_it) = puzzle.check(&guess, &mut attempts).unwrap();
            if got_it {
                break;
            }
            for (i, grade) in grades.iter().enumerate() {
                let c = guess.chars().nth(i).unwrap();
                match grade {
                    Grade::Correct => correct.push((i, c)),
                    Grade::Almost => {
                        let locator = almost.entry(c).or_insert(0);
                        *locator &= 1 << i;
                    }
                    Grade::Incorrect => incorrect.push(c),
                }
            }
        }

        attempts
    }
}

impl Display for Basic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wordle_strategies::Basic ")?;
        if self.good_start {
            write!(f, "(good start)")
        } else {
            write!(f, "(bad start)")
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use wordle_perf::strategy::{Puzzle, Strategy, Word};
    use wordle_perf::words::ANSWERS;

    #[test]
    fn guesses() {
        let strat = Basic::new().good_start();
        let word = Word::from_wordlist(ANSWERS[500]).unwrap();
        println!("{}\n-----", word);
        let puzzle = Puzzle::new(word, false);
        let attempts = strat.solve(&puzzle);
        println!("{}", attempts);
    }
}
