//! The test harness for running Wordle strategies.

use std::{
    ops::Deref,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[cfg(all(feature = "fancy", feature = "parallel"))]
use indicatif::ParallelProgressIterator;
#[cfg(all(feature = "fancy", not(feature = "parallel")))]
use indicatif::ProgressIterator;
use rand::seq::index::sample;
#[cfg(feature = "parallel")]
use rayon::prelude::*;

use crate::{
    perf::Perf,
    strategy::{AttemptsKey, Puzzle, Strategy, Word},
    words::ANSWERS,
    HarnessError, Summary, WordleError,
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
/// use wordle_rs::strategy::stupid::Stupid;
///
/// let harness = Harness::new()
///     .quiet()
///     .add_strategy(Box::new(Stupid), None)
///     .test_num(50);
///
/// let results = harness.run();
/// ```
#[derive(Debug)]
pub struct Harness {
    strategies: Vec<(Box<dyn Strategy>, Option<String>)>,
    verbose: bool,
    num_guesses: Option<usize>,
    baseline: BaselineOpt,
}

impl Default for Harness {
    fn default() -> Self {
        Harness {
            strategies: Vec::new(),
            verbose: false,
            num_guesses: Some(100),
            baseline: BaselineOpt::None,
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
    /// 4. does not compare against a baseline
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
    pub fn add_strategy<'a>(
        self,
        strat: Box<dyn Strategy>,
        save_name: impl Into<Option<&'a str>>,
    ) -> Self {
        let mut strategies = self.strategies;
        strategies.push((strat, save_name.into().map(|s| s.to_string())));
        Harness { strategies, ..self }
    }

    /// Adds a [`Vec`] of strategies to the harness for testing.
    pub fn add_strategies(self, strats: Vec<(Box<dyn Strategy>, Option<String>)>) -> Self {
        let mut strategies = self.strategies;
        strategies.extend(strats);
        Harness { strategies, ..self }
    }

    /// Adds a strategy to the harness for testing and sets it as the baseline
    /// for comparison.
    pub fn add_baseline<'a>(
        self,
        strat: Box<dyn Strategy>,
        save_name: impl Into<Option<&'a str>>,
    ) -> Result<Self, WordleError> {
        self.add_strategy(strat, save_name).and_baseline()
    }

    /// Sets the most recently added strategy as the baseline for comparisons.
    pub fn and_baseline(self) -> Result<Self, WordleError> {
        match self.baseline {
            BaselineOpt::None => Ok(Self {
                baseline: BaselineOpt::Run(
                    self.strategies.len() - 1,
                    self.strategies
                        .last()
                        .ok_or(HarnessError::NoStrategiesAdded)?
                        .1
                        .clone(),
                ),
                ..self
            }),
            _ => Err(HarnessError::BaselineAlreadySet.into()),
        }
    }

    /// Adds a saved performance record as the baseline for comparisons.
    ///
    /// The `name` must match the name of a baseline saved previously.
    #[cfg(feature = "serde")]
    pub fn load_baseline<'a>(
        self,
        name: &str,
        dir: impl Into<Option<&'a Path>>,
    ) -> Result<Self, WordleError> {
        match self.baseline {
            BaselineOpt::None => {
                let dir = get_save_dir(dir)?;
                let baseline = Summary::from_saved(name, dir)?;
                Ok(Self {
                    baseline: BaselineOpt::Saved(Box::new(baseline), name.to_string()),
                    ..self
                })
            }
            _ => Err(HarnessError::BaselineAlreadySet.into()),
        }
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

    fn pre_run_check(&self) -> Result<(), WordleError> {
        if self.strategies.is_empty() {
            return Err(HarnessError::NoStrategiesAdded.into());
        }

        Ok(())
    }

    pub fn debug_run(&self, words: Option<&[Word]>) -> Result<Vec<Perf>, WordleError> {
        use std::panic::{self, AssertUnwindSafe};

        self.pre_run_check()?;

        let mut perfs = Vec::new();
        for strat in &self.strategies {
            perfs.push(Perf::new(strat.0.as_ref()))
        }

        let words = match words {
            Some(w) => Vec::from(w),
            None => ANSWERS
                .iter()
                .map(|&i| Word::from_index(i))
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
        };

        for word in words.iter() {
            for (i, (strategy, _)) in self.strategies.iter().enumerate() {
                let key = AttemptsKey::new(strategy.hardmode());
                let res = {
                    let wrapper = AssertUnwindSafe(strategy);
                    panic::catch_unwind(|| {
                        let mut puzzle = Puzzle::new(*word);
                        let attempts = (*wrapper).solve(&mut puzzle, key);
                        (puzzle, attempts)
                    })
                }
                .map_or_else(
                    |_| {
                        println!("strategy {strategy} panicked on puzzle {word}");
                        println!("------------");
                        None
                    },
                    Some,
                );
                if let Some((puzzle, solution)) = res {
                    perfs[i].tries.push((*word, solution));

                    if puzzle.poisoned {
                        return Err(HarnessError::StrategyCheated(format!("{}", strategy)).into());
                    }
                }
            }
        }

        Ok(perfs)

        // words
        //     .iter()
        //     .progress()
        //     .map(|w| )
        //     .collect::<Result<(), WordleError>>()
        //     .map(|_| Arc::try_unwrap(perfs).unwrap().into_inner().unwrap())
    }

    /// Runs the harness and produces performances for each strategy.
    ///
    /// The [`Perf`]s will be in the same order as the strategies were added
    /// to the harness.
    pub fn run(&self) -> Result<Record, WordleError> {
        self.pre_run_check()?;

        let perfs = Arc::new(Mutex::new(Vec::new()));
        {
            let mut perfs = perfs.lock().unwrap();
            for strat in &self.strategies {
                perfs.push(Perf::new(strat.0.as_ref()))
            }
        }

        let mut rng = rand::thread_rng();

        if let Some(n) = self.num_guesses {
            // try only some random words

            #[cfg(all(feature = "parallel", not(feature = "fancy")))]
            sample(&mut rng, ANSWERS.len(), n)
                .iter()
                .par_bridge()
                .map(|i| self.run_inner(ANSWERS[i], perfs.clone()))
                .collect::<Result<(), WordleError>>()?;

            #[cfg(all(not(feature = "parallel"), not(feature = "fancy")))]
            sample(&mut rng, ANSWERS.len(), n)
                .iter()
                .map(|i| self.run_inner(ANSWERS[i], perfs.clone()))
                .collect::<Result<(), WordleError>>()?;

            #[cfg(feature = "fancy")]
            if self.verbose {
                #[cfg(feature = "parallel")]
                sample(&mut rng, ANSWERS.len(), n)
                    .iter()
                    .par_bridge()
                    .progress_count(n as u64)
                    .map(|i| self.run_inner(ANSWERS[i], perfs.clone()))
                    .collect::<Result<(), WordleError>>()?;

                #[cfg(not(feature = "parallel"))]
                sample(&mut rng, ANSWERS.len(), n)
                    .iter()
                    .progress_count(n as u64)
                    .map(|i| self.run_inner(ANSWERS[i], perfs.clone()))
                    .collect::<Result<(), WordleError>>()?;
            } else {
                #[cfg(feature = "parallel")]
                sample(&mut rng, ANSWERS.len(), n)
                    .iter()
                    .par_bridge()
                    .map(|i| self.run_inner(ANSWERS[i], perfs.clone()))
                    .collect::<Result<(), WordleError>>()?;

                #[cfg(not(feature = "parallel"))]
                sample(&mut rng, ANSWERS.len(), n)
                    .iter()
                    .map(|i| self.run_inner(ANSWERS[i], perfs.clone()))
                    .collect::<Result<(), WordleError>>()?;
            }
        } else {
            // try all words

            #[cfg(feature = "parallel")]
            (0..ANSWERS.len())
                .into_par_iter()
                .map(|i| self.run_inner(ANSWERS[i], perfs.clone()))
                .collect::<Result<(), WordleError>>()?;

            #[cfg(not(feature = "parallel"))]
            (0..ANSWERS.len())
                .into_iter()
                .map(|i| self.run_inner(ANSWERS[i], perfs.clone()))
                .collect::<Result<(), WordleError>>()?;

            #[cfg(feature = "fancy")]
            if self.verbose {
                #[cfg(feature = "parallel")]
                (0..ANSWERS.len())
                    .into_par_iter()
                    .progress()
                    .map(|i| self.run_inner(ANSWERS[i], perfs.clone()))
                    .collect::<Result<(), WordleError>>()?;

                #[cfg(not(feature = "parallel"))]
                (0..ANSWERS.len())
                    .into_iter()
                    .progress()
                    .map(|i| self.run_inner(ANSWERS[i], perfs.clone()))
                    .collect::<Result<(), WordleError>>()?;
            } else {
                #[cfg(feature = "parallel")]
                (0..ANSWERS.len())
                    .into_par_iter()
                    .map(|i| self.run_inner(ANSWERS[i], perfs.clone()))
                    .collect::<Result<(), WordleError>>()?;

                #[cfg(not(feature = "parallel"))]
                (0..ANSWERS.len())
                    .into_iter()
                    .map(|i| self.run_inner(ANSWERS[i], perfs.clone()))
                    .collect::<Result<(), WordleError>>()?;
            }
        }

        let perfs = Arc::try_unwrap(perfs).unwrap().into_inner().unwrap();

        #[cfg(feature = "serde")]
        for ((_, name), perf) in self.strategies.iter().zip(perfs.iter()) {
            if let Some(name) = name {
                let summary = perf.to_summary();
                let dir = get_save_dir(None)?;
                summary.save(name, &dir, false)?;
            }
        }

        Ok(Record::new(perfs, self.baseline.clone()))
    }

    fn run_inner(&self, index: usize, perfs: Arc<Mutex<Vec<Perf>>>) -> Result<(), WordleError> {
        let word = Word::from_index(index).unwrap();
        let mut puzzle = Puzzle::new(word);

        for (i, strategy) in self.strategies.iter().enumerate() {
            let key = AttemptsKey::new(strategy.0.hardmode());
            let solution = strategy.0.solve(&mut puzzle, key);
            {
                let mut perfs = perfs.lock().unwrap();
                perfs[i].tries.push((word, solution));
            }
            if puzzle.poisoned {
                return Err(HarnessError::StrategyCheated(format!("{}", strategy.0)).into());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) enum BaselineOpt {
    None,
    Run(usize, Option<String>),
    #[cfg(feature = "serde")]
    Saved(Box<Summary>, String),
}

impl BaselineOpt {
    pub(crate) fn get_summary(&self, perfs: &[Perf]) -> Option<Summary> {
        match self {
            Self::None => None,
            Self::Run(n, _) => Some(perfs[*n].to_summary()),
            #[cfg(feature = "serde")]
            Self::Saved(s, _) => Some(s.deref().clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    perfs: Vec<Perf>,
    baseline: BaselineOpt,
}

impl Deref for Record {
    type Target = [Perf];

    fn deref(&self) -> &Self::Target {
        &self.perfs
    }
}

impl Default for Record {
    fn default() -> Self {
        Self {
            perfs: Vec::new(),
            baseline: BaselineOpt::None,
        }
    }
}

impl Record {
    fn new(perfs: Vec<Perf>, baseline: BaselineOpt) -> Self {
        Self { perfs, baseline }
    }

    pub fn print_report(&self) -> Result<(), WordleError> {
        match self.baseline.get_summary(&self.perfs) {
            Some(baseline_summary) => {
                let mut printed_baseline = false;
                for perf in self.perfs.iter() {
                    let summary = perf.to_summary();
                    match summary.print(
                        Summary::print_options()
                            .compare(&baseline_summary)
                            .histogram(true),
                    ) {
                        Ok(()) => {}
                        Err(WordleError::SelfComparison) => {
                            printed_baseline = true;
                            summary
                                .print(
                                    Summary::print_options()
                                        .histogram(true)
                                        .baseline(&self.baseline),
                                )
                                .unwrap()
                        }
                        Err(e) => return Err(e),
                    }
                }
                if !printed_baseline {
                    baseline_summary
                        .print(
                            Summary::print_options()
                                .histogram(true)
                                .baseline(&self.baseline),
                        )
                        .unwrap()
                }
            }
            None => {
                for perf in self.perfs.iter() {
                    let summary = perf.to_summary();
                    summary.print(Summary::print_options().histogram(true))?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(feature = "serde")]
fn get_save_dir<'a>(user: impl Into<Option<&'a Path>>) -> Result<PathBuf, WordleError> {
    let user: Option<&Path> = user.into();
    let var = std::env::var_os("WORDLE_BASELINE_DIR");
    let default = std::env::current_dir()
        .map_err(HarnessError::BaselineIo)?
        .join("wordle_baseline");

    let dir = match &user {
        Some(p) => p,
        None => match &var {
            Some(s) => Path::new(s),
            None => default.as_path(),
        },
    };

    Ok(dir.to_path_buf())
}
