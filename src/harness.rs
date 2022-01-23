use std::sync::{Arc, Mutex};

use indicatif::ParallelProgressIterator;
use rand::seq::index::sample;
use rayon::prelude::*;

use crate::{
    perf::Perf,
    strategy::{Puzzle, Strategy, Word},
    words::ANSWERS,
};

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn verbose(self) -> Self {
        Harness {
            verbose: true,
            ..self
        }
    }

    pub fn quiet(self) -> Self {
        Harness {
            verbose: false,
            ..self
        }
    }

    pub fn add_strategy(self, strat: Box<dyn Strategy>) -> Self {
        let mut strategies = self.strategies;
        strategies.push(strat);
        Harness { strategies, ..self }
    }

    pub fn add_strategies(self, strats: Vec<Box<dyn Strategy>>) -> Self {
        let mut strategies = self.strategies;
        strategies.extend(strats);
        Harness { strategies, ..self }
    }

    pub fn test_all(self) -> Self {
        Harness {
            num_guesses: None,
            ..self
        }
    }

    pub fn test_num(self, n: usize) -> Self {
        Harness {
            num_guesses: Some(n.clamp(0, ANSWERS.len())),
            ..self
        }
    }

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

    pub fn run_and_summarize(&self) -> Vec<Perf> {
        let perfs = self.run();
        for perf in &perfs {
            println!("{}", perf);
        }
        perfs
    }
}
