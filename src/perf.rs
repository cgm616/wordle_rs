//! Evaluating and comparing strategies.

use std::{borrow::Borrow, fmt::Display, io::Write, ops::Deref};

use comfy_table::{Cell, Color, ColumnConstraint, Row, Table, Width};
use either::Either;
use fishers_exact::FishersExactPvalues;
use nanostat::Difference;
use owo_colors::{AnsiColors, OwoColorize, Stream};
use serde::{Deserialize, Serialize};

use crate::{
    strategy::{Attempts, Strategy, Word},
    WordleError,
};

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
    pub fn print(&self) {
        print!("{}", self);
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
            strategy_name: &self.strategy_name,
            num_tried: self.num_tried(),
            num_solved: self.num_solved(),
            cumulative_guesses: self.cumulative_guesses(),
            histogram: bins.into(),
        }
    }
}

impl Display for Perf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let perf_summary = self.to_summary();
        write!(f, "{}", perf_summary)
    }
}

/// A summary of a strategy's performance generated by the
/// [test harness](crate::Harness).
///
/// It is recommended to convert the [`Perf`] struct to this via the
/// [`Perf::to_summary()`] method when you want to use the performance to run
/// statistics.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Summary<'a> {
    strategy_name: &'a str,
    num_tried: u32,
    num_solved: u32,
    cumulative_guesses: u32,
    histogram: Histogram,
}

impl<'a> Summary<'a> {
    /// Gets the name of the strategy that produced this performance record.
    pub fn strategy_name(&self) -> &'a str {
        self.strategy_name
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

    pub fn compare<'b>(&self, baseline: &Summary<'b>) -> Result<Comparison<'a, 'b>, WordleError> {
        if self == baseline {
            return Err(WordleError::SelfComparison);
        }

        Ok(Comparison::compare(self.clone(), baseline.clone(), 95.))
    }

    pub fn print(&self, options: SummaryPrintOptions) -> Result<(), WordleError> {
        let mut stdout = std::io::stdout();
        match options.compare {
            Some(baseline) => {
                let comparison = self.compare(&baseline)?;

                writeln!(stdout, "{:-^80}", self.strategy_name)?;
                writeln!(
                    stdout,
                    "Ran {} words and comp. with {}, {} words",
                    self.num_tried(),
                    baseline.strategy_name(),
                    baseline.num_tried()
                )?;

                let solved_sig = match comparison.solved {
                    Either::Left(f) => f.two_tail_pvalue < 0.05,
                    Either::Right(c) => todo!(),
                };

                if solved_sig {
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
                            }
                        ),
                        self.num_missed(),
                        "a sig. diff.".if_supports_color(Stream::Stdout, |text| text.bold())
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

                if comparison.guesses.is_significant() {
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
                } else {
                    writeln!(
                        stdout,
                        "Correct guesses took {:.2} ({:+.2}) attempts on average, not a sig. diff.",
                        self.mean_guesses(),
                        comparison.mean_guesses_diff(),
                    )?;
                }
            }
            None => {
                    writeln!(stdout, "{:-^80}", self.strategy_name)?;
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

    pub fn print_options() -> SummaryPrintOptions<'a> {
        SummaryPrintOptions::default()
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SummaryPrintOptions<'a> {
    compare: Option<Summary<'a>>,
    histogram: bool,
}

impl<'a> SummaryPrintOptions<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compare(self, baseline: &Summary<'a>) -> Self {
        Self {
            compare: Some(baseline.clone()),
            ..self
        }
    }

    pub fn histogram(self, histogram: bool) -> Self {
        Self { histogram, ..self }
    }
}

impl<'a> Display for Summary<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:-^80}", self.strategy_name)?;
        writeln!(f, "Ran {} words", self.num_tried(),)?;

        writeln!(
            f,
            "Guessed {} correctly, or {:.1}%, and {} incorrectly",
            self.num_solved(),
            self.frac_solved() * 100.,
            self.num_missed()
        )?;

        writeln!(
            f,
            "Correct guesses took {:.2} attempts on average",
            self.mean_guesses(),
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Comparison<'a, 'b> {
    this: Summary<'a>,
    baseline: Summary<'b>,
    solved: Either<FishersExactPvalues, ChiSquared>,
    guesses: Difference,
}

impl<'a, 'b> Comparison<'a, 'b> {
    pub fn compare(this: Summary<'a>, baseline: Summary<'b>, confidence: f64) -> Self {
        let this_stats_vec: Vec<_> = this
            .histogram
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64 + 1.) * v as f64)
            .collect();
        let baseline_stats_vec: Vec<_> = baseline
            .histogram
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64 + 1.) * v as f64)
            .collect();
        let this_stats: nanostat::Summary = this_stats_vec.iter().collect();
        let baseline_stats: nanostat::Summary = baseline_stats_vec.iter().collect();

        let guesses = this_stats.compare(&baseline_stats, confidence);

        let solved = if this.num_tried().min(baseline.num_tried()) <= 10000 {
            // Run fisher's
            let res = fishers_exact::fishers_exact(&[
                this.num_solved(),
                baseline.num_solved(),
                this.num_missed(),
                baseline.num_missed(),
            ])
            .unwrap();
            Either::Left(res)
        } else {
            // Run chi-squared

            todo!()
        };

        Self {
            this,
            baseline,
            solved,
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

#[derive(Clone, Debug)]
struct ChiSquared {
    pvalue: f64,
}

impl ChiSquared {
    fn test() {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
