//! Evaluating and comparing strategies.

use std::{
    ffi::OsStr,
    fmt::Display,
    fs::File,
    io::Write,
    ops::Deref,
    path::{Path, PathBuf},
};

#[cfg(feature = "fancy")]
use comfy_table::{Cell, Color, ColumnConstraint, Row, Table, Width};
#[cfg(feature = "stats")]
use fishers_exact::FishersExactPvalues;
#[cfg(feature = "fancy")]
use owo_colors::{AnsiColors, OwoColorize, Stream};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    harness::BaselineOpt,
    strategy::{Attempts, Strategy, Word},
    {HarnessError, WordleError},
};

#[cfg(feature = "stats")]
use crate::stats::{Tails, WelchsT};

/// A record of one strategy's guesses after run by the
/// [test harness](crate::Harness).
///
/// This struct can provide statistics about the attempts on its own, but it
/// is recommended to produce [`Summary`] first to cache the computations.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Perf {
    pub(crate) tries: Vec<(Word, Attempts)>,
    strategy_name: String,
}

impl Perf {
    /// Creates a new empty performance record.
    pub(crate) fn new(strat: &dyn Strategy) -> Self {
        Perf {
            tries: Vec::new(),
            strategy_name: format!("{} v{}", strat, strat.version()),
        }
    }

    /// Gets the name of the strategy that produced this performance record.
    pub fn strategy_name(&self) -> &str {
        &self.strategy_name
    }

    pub fn tries(&self) -> &[(Word, Attempts)] {
        &self.tries
    }

    /// Gets the number of puzzles attempted by the strategy.
    pub fn num_tried(&self) -> u32 {
        self.tries.len() as u32
    }

    /// Gets the number of puzzles solved by the strategy.
    ///
    /// This function always returns a number less than [`num_tried()`](Self::num_tried()).
    pub fn num_solved(&self) -> u32 {
        self.tries
            .iter()
            .filter(|(word, attempts)| attempts.solved(word))
            .count() as u32
    }

    /// Gets the fraction of puzzles solved by the strategy.
    pub fn frac_solved(&self) -> f32 {
        (self.num_solved() as f32) / (self.num_tried() as f32)
    }

    /// Gets the number of guesses across all puzzle attempts.
    pub fn cumulative_guesses(&self) -> u32 {
        self.tries.iter().map(|(_, a)| a.inner().len() as u32).sum()
    }

    /// Gets the number of guesses across all solved puzzles.
    pub fn cumulative_guesses_solved(&self) -> u32 {
        self.tries
            .iter()
            .filter(|(word, attempts)| attempts.solved(word))
            .map(|(_, a)| a.inner().len() as u32)
            .sum()
    }

    /// Gets the average number of guesses needed to solve a puzzle.
    ///
    /// This function does not include guesses made on puzzles that the
    /// strategy was unable to solve.
    pub fn guesses_per_solution(&self) -> f32 {
        (self.cumulative_guesses_solved() as f32) / (self.num_solved() as f32)
    }

    /// Gets the number of puzzles the strategy could not solve.
    ///
    /// This function always returns a number less than or equal to
    /// [`num_tried()`](Self::num_tried()).
    pub fn num_missed(&self) -> u32 {
        self.num_tried() - self.num_solved()
    }

    /// Gets the fraction of puzzles the strategy could not solve.
    pub fn frac_missed(&self) -> f32 {
        (self.num_missed() as f32) / (self.num_tried() as f32)
    }

    /// Prints the strategy's summary and then output a table showing the
    /// strategy's attempts for each puzzle.
    #[cfg(feature = "fancy")]
    pub fn print(&self) {
        let mut table = Table::new();
        if !table.is_tty() {
            table.set_table_width(80);
        } else {
            table.load_preset(comfy_table::presets::UTF8_FULL);
        }
        let columns = (table.get_table_width().unwrap() / 9) as usize;
        for chunk in self.tries.chunks(columns) {
            let mut row = Row::new();
            for (word, attempts) in chunk {
                let mut cell = Cell::new(format!("{}\n-----\n{}", word, attempts));
                if !attempts.solved(word) {
                    cell = cell.bg(Color::Red).fg(Color::Black);
                }
                row.add_cell(cell);
            }
            table.add_row(row);
        }
        table.set_constraints(vec![
            ColumnConstraint::LowerBoundary(Width::Fixed(5));
            columns
        ]);
        println!("{}", table);
    }

    /// Converts this performance record to a pre-calculated summary.
    pub fn to_summary(&self) -> Summary {
        let mut bins = [0; 6];

        self.tries
            .iter()
            .filter(|(word, attempts)| attempts.solved(word))
            .map(|(_, attempts)| attempts.inner().len())
            .for_each(|n| bins[n - 1] += 1);

        assert_eq!(bins.iter().sum::<u32>(), self.num_solved());

        Summary {
            strategy_name: self.strategy_name.clone(),
            num_tried: self.num_tried(),
            num_solved: self.num_solved(),
            cumulative_guesses: self.cumulative_guesses(),
            histogram: bins.into(),
        }
    }
}

/// A summary of a strategy's performance generated by the
/// [test harness](crate::Harness).
///
/// It is recommended to convert the [`Perf`] struct to this via the
/// [`Perf::to_summary()`] method when you want to use the performance to run
/// statistics.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
pub struct Summary {
    strategy_name: String,
    num_tried: u32,
    num_solved: u32,
    cumulative_guesses: u32,
    histogram: Histogram,
}

impl Summary {
    /// Gets the name of the strategy that produced this performance record.
    pub fn strategy_name(&self) -> &str {
        &self.strategy_name
    }

    /// Gets the number of puzzles attempted by the strategy.
    pub fn num_tried(&self) -> u32 {
        self.num_tried
    }

    /// Gets the number of puzzles solved by the strategy.
    ///
    /// This function always returns a number less than or equal to
    /// [`num_tried()`](PerfSummary::num_tried()).
    pub fn num_solved(&self) -> u32 {
        self.num_solved
    }

    /// Gets the fraction of puzzles solved by the strategy.
    pub fn frac_solved(&self) -> f32 {
        (self.num_solved as f32) / (self.num_tried as f32)
    }

    /// Gets the number of guesses across all puzzle attempts.
    pub fn cumulative_guesses(&self) -> u32 {
        self.cumulative_guesses
    }

    /// Gets the number of guesses across all solved puzzles.
    pub fn cumulative_guesses_solved(&self) -> u32 {
        self.histogram
            .iter()
            .enumerate()
            .map(|(i, v)| i as u32 * v)
            .sum::<u32>()
    }

    /// Gets the average number of guesses needed to solve a puzzle.
    ///
    /// This function does not include guesses made on puzzles that the
    /// strategy was unable to solve.
    pub fn mean_guesses(&self) -> f32 {
        (self.cumulative_guesses_solved() as f32) / (self.num_solved as f32)
    }

    /// Gets the number of puzzles the strategy could not solve.
    ///
    /// This function will always return a number less than [`num_tried()`](Self::num_tried()).
    pub fn num_missed(&self) -> u32 {
        self.num_tried - self.num_solved
    }

    /// Gets the fraction of puzzles the strategy could not solve.
    pub fn frac_missed(&self) -> f32 {
        (self.num_missed() as f32) / (self.num_tried as f32)
    }

    pub fn compare<'a, 'b>(
        &'a self,
        baseline: &'b Summary,
    ) -> Result<Comparison<'a, 'b>, WordleError> {
        if self == baseline {
            return Err(WordleError::SelfComparison);
        }

        Ok(Comparison::compare(
            self,
            baseline,
            #[cfg(feature = "stats")]
            0.05,
        ))
    }

    pub fn print(&self, options: SummaryPrintOptions) -> Result<(), WordleError> {
        let mut stdout = std::io::stdout();
        match options.compare {
            Some(baseline) => {
                let comparison = self.compare(&baseline)?;

                writeln!(stdout, "{:-^80}", self.strategy_name)?;
                writeln!(
                    stdout,
                    "Ran {} words against {} on {} words",
                    self.num_tried(),
                    baseline.strategy_name(),
                    baseline.num_tried()
                )?;

                #[cfg(feature = "stats")]
                if comparison.solved.two_tail_pvalue < 0.05 {
                    #[cfg(feature = "fancy")]
                    writeln!(
                        stdout,
                        "Guessed {} correctly, or {:.1}% ({:+.1}%), and {} incorrectly, {}",
                        self.num_solved(),
                        self.frac_solved() * 100.,
                        (comparison.frac_solved_diff() * 100.).if_supports_color(
                            Stream::Stdout,
                            |text| {
                                if comparison.frac_solved_diff().is_sign_positive() {
                                    text.color(AnsiColors::Green)
                                } else {
                                    text.color(AnsiColors::Red)
                                }
                            },
                        ),
                        self.num_missed(),
                        "a sig. diff.".if_supports_color(Stream::Stdout, |text| text.bold())
                    )?;

                    #[cfg(not(feature = "fancy"))]
                    writeln!(
                        stdout,
                        "Guessed {} correctly, or {:.1}% ({:+.1}%), and {} incorrectly, a sig. diff.",
                        self.num_solved(),
                        self.frac_solved() * 100.,
                        comparison.frac_solved_diff() * 100.,
                        self.num_missed(),
                    )?;
                } else {
                    writeln!(
                        stdout,
                        "Guessed {} correctly, or {:.1}% ({:+.1}%), and {} incorrectly, not a sig. diff.",
                        self.num_solved(),
                        self.frac_solved() * 100.,
                        comparison.frac_solved_diff() * 100.,
                        self.num_missed()
                    )?;
                }

                #[cfg(all(not(feature = "stats"), feature = "fancy"))]
                writeln!(
                    stdout,
                    "Guessed {} correctly, or {:.1}% ({:+.1}%), and {} incorrectly",
                    self.num_solved(),
                    self.frac_solved() * 100.,
                    (comparison.frac_solved_diff() * 100.).if_supports_color(
                        Stream::Stdout,
                        |text| {
                            if comparison.frac_solved_diff().is_sign_positive() {
                                text.color(AnsiColors::Green)
                            } else {
                                text.color(AnsiColors::Red)
                            }
                        },
                    ),
                    self.num_missed()
                )?;

                #[cfg(all(not(feature = "stats"), not(feature = "fancy")))]
                writeln!(
                    stdout,
                    "Guessed {} correctly, or {:.1}% ({:+.1}%), and {} incorrectly",
                    self.num_solved(),
                    self.frac_solved() * 100.,
                    (comparison.frac_solved_diff() * 100.),
                    self.num_missed()
                )?;

                #[cfg(feature = "stats")]
                if comparison.guesses.is_significant() {
                    #[cfg(feature = "fancy")]
                    writeln!(
                        stdout,
                        "Correct guesses took {:.2} ({:.2}) attempts on average, {}",
                        self.mean_guesses(),
                        comparison
                            .mean_guesses_diff()
                            .if_supports_color(Stream::Stdout, |text| {
                                if comparison.mean_guesses_diff().is_sign_negative() {
                                    text.color(AnsiColors::Green)
                                } else {
                                    text.color(AnsiColors::Red)
                                }
                            }),
                        "a sig. diff.".if_supports_color(Stream::Stdout, |text| text.bold())
                    )?;

                    #[cfg(not(feature = "fancy"))]
                    writeln!(
                        stdout,
                        "Correct guesses took {:.2} ({:.2}) attempts on average, a sig. diff.",
                        self.mean_guesses(),
                        comparison.mean_guesses_diff(),
                    )?;
                } else {
                    writeln!(
                        stdout,
                        "Correct guesses took {:.2} ({:+.2}) attempts on average, not a sig. diff.",
                        self.mean_guesses(),
                        comparison.mean_guesses_diff(),
                    )?;
                }

                #[cfg(all(not(feature = "stats"), feature = "fancy"))]
                writeln!(
                    stdout,
                    "Correct guesses took {:.2} ({:.2}) attempts on average",
                    self.mean_guesses(),
                    comparison
                        .mean_guesses_diff()
                        .if_supports_color(Stream::Stdout, |text| {
                            if comparison.mean_guesses_diff().is_sign_negative() {
                                text.color(AnsiColors::Green)
                            } else {
                                text.color(AnsiColors::Red)
                            }
                        }),
                )?;

                #[cfg(all(not(feature = "stats"), not(feature = "fancy")))]
                writeln!(
                    stdout,
                    "Correct guesses took {:.2} ({:.2}) attempts on average",
                    self.mean_guesses(),
                    comparison.mean_guesses_diff(),
                )?;

                #[cfg(feature = "stats")]
                writeln!(stdout, "P-value: {}", comparison.guesses.p)?;
            }
            None => {
                if let Some(s) = options.baseline {
                    writeln!(stdout, "Baseline{:-^72}", self.strategy_name)?;
                    writeln!(stdout, "{}", s)?;
                } else {
                    writeln!(stdout, "{:-^80}", self.strategy_name)?;
                }
                writeln!(stdout, "Ran {} words", self.num_tried(),)?;

                writeln!(
                    stdout,
                    "Guessed {} correctly, or {:.1}%, and {} incorrectly",
                    self.num_solved(),
                    self.frac_solved() * 100.,
                    self.num_missed()
                )?;

                writeln!(
                    stdout,
                    "Correct guesses took {:.2} attempts on average",
                    self.mean_guesses(),
                )?;
            }
        }

        if options.histogram {
            write!(stdout, "{}", self.histogram)?;
        }

        Ok(())
    }

    pub fn print_options() -> SummaryPrintOptions {
        SummaryPrintOptions::default()
    }

    #[cfg(feature = "serde")]
    pub fn from_saved(name: &str, dir: impl AsRef<Path>) -> Result<Summary, WordleError> {
        let dir = dir.as_ref();

        let path = dir
            .read_dir()
            .map_err(HarnessError::BaselineIo)?
            .filter(Result::is_ok)
            .map(Result::unwrap)
            .find(|entry| {
                let path = entry.path();
                path.file_stem() == Some(OsStr::new(name))
                    && path.extension() == Some(OsStr::new("json"))
            })
            .ok_or(HarnessError::BaselineDoesntExist)?
            .path();

        let file = File::options()
            .read(true)
            .open(path)
            .map_err(HarnessError::BaselineIo)?;

        let summary = serde_json::from_reader(file).map_err(HarnessError::Serde)?;

        Ok(summary)
    }

    #[cfg(feature = "serde")]
    pub fn save(
        &self,
        name: &str,
        dir: impl AsRef<Path>,
        force: bool,
    ) -> Result<PathBuf, WordleError> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir).map_err(HarnessError::BaselineIo)?;

        let mut path = dir.join(name);
        path.set_extension("json");

        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .create_new(!force)
            .open(&path)
            .map_err(HarnessError::BaselineIo)?;

        serde_json::to_writer(&mut file, self).map_err(HarnessError::Serde)?;

        Ok(path)
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SummaryPrintOptions {
    compare: Option<Summary>,
    histogram: bool,
    baseline: Option<String>,
}

impl<'a> SummaryPrintOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compare(self, baseline: &Summary) -> Self {
        Self {
            compare: Some(baseline.clone()),
            ..self
        }
    }

    pub fn histogram(self, histogram: bool) -> Self {
        Self { histogram, ..self }
    }

    pub(crate) fn baseline(self, baseline: &BaselineOpt) -> Self {
        let baseline = match baseline {
            BaselineOpt::None => None,
            BaselineOpt::Run(_, Some(name)) => {
                Some(format!("Used as baseline and saved as {}", name))
            }
            BaselineOpt::Run(_, None) => Some("Used as baseline and not saved".to_string()),
            #[cfg(feature = "serde")]
            BaselineOpt::Saved(_, name) => Some(format!("Loaded baseline {} from disk", name)),
        };

        Self { baseline, ..self }
    }
}

#[derive(Debug, Clone)]
pub struct Comparison<'a, 'b> {
    this: &'a Summary,
    baseline: &'b Summary,
    #[cfg(feature = "stats")]
    solved: FishersExactPvalues,
    #[cfg(feature = "stats")]
    guesses: WelchsT<f64>,
}

impl<'a, 'b> Comparison<'a, 'b> {
    pub fn compare(
        this: &'a Summary,
        baseline: &'b Summary,
        #[cfg(feature = "stats")] alpha: f64,
    ) -> Self {
        #[cfg(feature = "stats")]
        let guesses = WelchsT::two_sample(
            this.histogram
                .iter()
                .enumerate()
                .map(|(i, &v)| (i as f64 + 1.) * v as f64),
            baseline
                .histogram
                .iter()
                .enumerate()
                .map(|(i, &v)| (i as f64 + 1.) * v as f64),
            alpha,
            Tails::Two,
        );

        #[cfg(feature = "stats")]
        let solved = fishers_exact::fishers_exact(&[
            this.num_solved(),
            baseline.num_solved(),
            this.num_missed(),
            baseline.num_missed(),
        ])
        .unwrap();

        Self {
            this,
            baseline,
            #[cfg(feature = "stats")]
            solved,
            #[cfg(feature = "stats")]
            guesses,
        }
    }

    pub fn tries_eq(&self) -> bool {
        self.this.num_tried == self.baseline.num_tried
    }

    pub fn num_tried(&self) -> Option<u32> {
        if self.tries_eq() {
            Some(self.this.num_solved() - self.baseline.num_solved())
        } else {
            None
        }
    }

    pub fn num_solved_diff(&self) -> Option<u32> {
        if self.tries_eq() {
            Some(self.this.num_solved() - self.baseline.num_solved())
        } else {
            None
        }
    }

    pub fn num_missed_diff(&self) -> Option<u32> {
        if self.tries_eq() {
            Some(self.this.num_missed() - self.baseline.num_missed())
        } else {
            None
        }
    }

    pub fn frac_solved_diff(&self) -> f32 {
        self.this.frac_solved() - self.baseline.frac_solved()
    }

    pub fn frac_missed_diff(&self) -> f32 {
        self.this.frac_missed() - self.baseline.frac_missed()
    }

    pub fn mean_guesses_diff(&self) -> f32 {
        self.this.mean_guesses() - self.baseline.mean_guesses()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
pub struct Histogram {
    bins: [u32; 6],
}

impl From<[u32; 6]> for Histogram {
    fn from(other: [u32; 6]) -> Self {
        Self { bins: other }
    }
}

impl Deref for Histogram {
    type Target = [u32; 6];

    fn deref(&self) -> &Self::Target {
        &self.bins
    }
}

impl Display for Histogram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max = *self.iter().max().unwrap();
        let digits =
            std::iter::successors(Some(max), |&n| (n >= 10).then(|| n / 10)).count() as u32;
        let count_per_mark = (max as f32 / (80. - digits as f32 - 6.)).max(1.0);

        for (i, &bin) in self.bins.iter().enumerate() {
            write!(f, "{} |", i + 1)?;
            let marks = (bin as f32 / count_per_mark).floor() as usize;
            writeln!(f, "{:■>marks$} ({})", "", bin)?;
        }

        Ok(())

        // TODO: test this to make sure it never outputs a line longer than 80 characters
    }
}
