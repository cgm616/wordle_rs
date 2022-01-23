//! The test harness for running Wordle strategies.

use std::sync::{Arc, Mutex};

use indicatif::ParallelProgressIterator;
use rand::seq::index::sample;
use rayon::prelude::*;

use crate::{
    perf::Perf,
    strategy::{Puzzle, Strategy, Word},
    words::ANSWERS,
};

/// A test harness that can run many strategies on many puzzles.
///
/// When you want to test your strategies, create a new test harness
/// with [`new()`](Harness::new()). You can then configure it using various
/// methods. Note that these configuration methods consume the existing
/// [`Harness`] and return a new one.
///
/// # Examples
///
/// ```rust
/// # use wordle_rs::harness::Harness;
/// use wordle_strategies::Basic;
///
/// let harness = Harness::new()
///     .quiet()
///     .add_strategy(Box::new(Basic::new()))
///     .test_num(50);
///
/// let results = harness.run();
/// ```
#[derive(Debug)]
pub struct Harness {
    strategies: Vec<Box<dyn Strategy>>,
    verbose: bool,
    num_guesses: Option<usize>,
}

impl Default for Harness {
    fn default() -> Self {
        Harness {
            strategies: Vec::new(),
            verbose: false,
            num_guesses: Some(100),
        }
    }
}

impl Harness {
    /// Creates a new test harness with default configuration.
    ///
    /// Defaults:
    /// 1. tests no strategies
    /// 2. quiet mode
    /// 3. runs each strategy on 100 puzzles chosen at random
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes the harness verbose while testing.
    ///
    /// As of right now, this consists of a progress bar and nothing else.
    pub fn verbose(self) -> Self {
        Harness {
            verbose: true,
            ..self
        }
    }

    /// Makes the harness silent while testing.
    pub fn quiet(self) -> Self {
        Harness {
            verbose: false,
            ..self
        }
    }

    /// Adds a strategy to the harness for testing.
    pub fn add_strategy(self, strat: Box<dyn Strategy>) -> Self {
        let mut strategies = self.strategies;
        strategies.push(strat);
        Harness { strategies, ..self }
    }

    /// Adds a [`Vec`] of strategies to the harness for testing.
    pub fn add_strategies(self, strats: Vec<Box<dyn Strategy>>) -> Self {
        let mut strategies = self.strategies;
        strategies.extend(strats);
        Harness { strategies, ..self }
    }

    /// Sets the harness to test each strategy on each possible Wordle answer.
    pub fn test_all(self) -> Self {
        Harness {
            num_guesses: None,
            ..self
        }
    }

    /// Sets the harness to test each strategy on `n` random Wordle answers.
    pub fn test_num(self, n: usize) -> Self {
        Harness {
            num_guesses: Some(n.clamp(0, ANSWERS.len())),
            ..self
        }
    }

    /// Runs the harness and produces performances for each strategy.
    ///
    /// The [`Perf`]s will be in the same order as the strategies were added
    /// to the harness.
    pub fn run(&self) -> Vec<Perf> {
        let perfs = Arc::new(Mutex::new(Vec::new()));
        {
            let mut perfs = perfs.lock().unwrap();
            for strat in &self.strategies {
                perfs.push(Perf::new(strat.as_ref()))
            }
        }

        let mut rng = rand::thread_rng();

        if let Some(n) = self.num_guesses {
            // try only some random words

            if self.verbose {
                sample(&mut rng, ANSWERS.len(), n)
                    .iter()
                    .par_bridge()
                    .progress_count(n as u64)
                    .for_each(|i| self.run_inner(ANSWERS[i], perfs.clone()))
            } else {
                sample(&mut rng, ANSWERS.len(), n)
                    .iter()
                    .par_bridge()
                    .for_each(|i| self.run_inner(ANSWERS[i], perfs.clone()))
            }
        } else {
            if self.verbose {
                (0..ANSWERS.len())
                    .into_par_iter()
                    .progress()
                    .for_each(|i| self.run_inner(ANSWERS[i], perfs.clone()))
            } else {
                (0..ANSWERS.len())
                    .into_par_iter()
                    .for_each(|i| self.run_inner(ANSWERS[i], perfs.clone()))
            }
        }

        Arc::try_unwrap(perfs).unwrap().into_inner().unwrap()
    }

    fn run_inner(&self, index: usize, perfs: Arc<Mutex<Vec<Perf>>>) {
        let word = Word::from_index(index).unwrap();
        let puzzle = Puzzle::new(word.clone());

        for (i, strategy) in self.strategies.iter().enumerate() {
            let solution = strategy.solve(&puzzle);
            {
                let mut perfs = perfs.lock().unwrap();
                perfs[i].tries.push((word.clone(), solution));
            }
        }
    }

    /// Runs the harness (see [`run()`](Harness::run())) and prints performance
    /// summaries of each strategy.
    pub fn run_and_summarize(&self) -> Vec<Perf> {
        let perfs = self.run();
        for perf in &perfs {
            println!("{}", perf);
        }
        perfs
    }
}
