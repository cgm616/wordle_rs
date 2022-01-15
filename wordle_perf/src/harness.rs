use std::fmt::Display;

use rand::seq::index::sample;

use crate::{
    strategy::{Attempts, Puzzle, Strategy, Word},
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
        let mut perfs = Vec::new();
        for strat in &self.strategies {
            perfs.push(Perf::new(strat.as_ref()))
        }
        let mut rng = rand::thread_rng();

        if let Some(n) = self.num_guesses {
            // try only some random words
            for i in sample(&mut rng, ANSWERS.len(), n) {
                self.run_inner(ANSWERS[i], &mut perfs);
            }
        } else {
            for i in 0..ANSWERS.len() {
                self.run_inner(ANSWERS[i], &mut perfs);
            }
        }

        perfs
    }

    fn run_inner(&self, index: usize, perfs: &mut [Perf]) {
        let word = Word::from_wordlist(index).unwrap();
        let puzzle = Puzzle::new(word.clone(), false);

        for (i, strategy) in self.strategies.iter().enumerate() {
            perfs[i].tries.push((word.clone(), strategy.solve(&puzzle)));
        }

        // let word = puzzle.destroy();
    }

    pub fn run_and_summarize(&self) -> Vec<Perf> {
        let perfs = self.run();
        for perf in &perfs {
            perf.summarize();
            println!();
        }
        perfs
    }
}

#[derive(Clone, Debug)]
pub struct Perf {
    tries: Vec<(Word, Attempts)>,
    strategy_name: String,
}

impl Perf {
    pub(crate) fn new(strat: &dyn Strategy) -> Self {
        Perf {
            tries: Vec::new(),
            strategy_name: format!("{}", strat),
        }
    }

    pub fn num_tried(&self) -> usize {
        self.tries.len()
    }

    pub fn num_solved(&self) -> usize {
        self.tries
            .iter()
            .filter(|(word, attempts)| attempts.solved(word))
            .count()
    }

    pub fn frac_solved(&self) -> f32 {
        (self.num_solved() as f32) / (self.num_tried() as f32)
    }

    pub fn cumulative_guesses_solved(&self) -> usize {
        self.tries
            .iter()
            .filter(|(word, attempts)| attempts.solved(word))
            .map(|(_, a)| a.inner().len())
            .sum()
    }

    pub fn cumulative_guesses(&self) -> usize {
        self.tries.iter().map(|(_, a)| a.inner().len()).sum()
    }

    pub fn guesses_per_solution(&self) -> f32 {
        (self.cumulative_guesses_solved() as f32) / (self.num_solved() as f32)
    }

    pub fn num_missed(&self) -> usize {
        self.num_tried() - self.num_solved()
    }

    pub fn frac_missed(&self) -> f32 {
        (self.num_missed() as f32) / (self.num_tried() as f32)
    }

    pub fn summarize(&self) {
        print!("{}", self)
    }
}

impl Display for Perf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:-^80}", self.strategy_name)?;
        writeln!(
            f,
            "Guessed {} ({:.2}%) correctly, {} ({:.2}%) incorrectly out of {} words",
            self.num_solved(),
            self.frac_solved() * 100.,
            self.num_missed(),
            self.frac_missed() * 100.,
            self.num_tried()
        )?;
        writeln!(
            f,
            "Correct guesses took {:.2} attempts on average",
            self.guesses_per_solution()
        )?;

        Ok(())
    }
}
