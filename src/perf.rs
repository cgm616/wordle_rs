use std::fmt::Display;

use comfy_table::{Cell, Color, ColumnConstraint, Row, Table, Width};
use serde::{Deserialize, Serialize};

use crate::strategy::{Attempts, Strategy, Word};

/// A record of one strategy's guesses after run by the
/// [test harness](crate::Harness).
///
/// This struct can provide statistics about the attempts on its own, but it
/// is recommended to produce [PerfSummary] first to cache the computations.
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Perf {
    pub(crate) tries: Vec<(Word, Attempts)>,
    strategy_name: String,
}

impl Perf {
    /// Create a new empty performance record.
    pub(crate) fn new(strat: &dyn Strategy) -> Self {
        Perf {
            tries: Vec::new(),
            strategy_name: format!("{} v{}", strat, strat.version()),
        }
    }

    /// Get the name of the strategy that produced this performance record.
    pub fn strategy_name(&self) -> &str {
        &self.strategy_name
    }

    /// Get the number of puzzles attempted by the strategy.
    pub fn num_tried(&self) -> u32 {
        self.tries.len() as u32
    }

    /// Get the number of puzzles solved by the strategy.
    ///
    /// This function will always return a number less than `num_tried()`.
    pub fn num_solved(&self) -> u32 {
        self.tries
            .iter()
            .filter(|(word, attempts)| attempts.solved(word))
            .count() as u32
    }

    /// Get the fraction of puzzles solved by the strategy.
    pub fn frac_solved(&self) -> f32 {
        (self.num_solved() as f32) / (self.num_tried() as f32)
    }

    /// Get the number of guesses across all puzzle attempts.
    pub fn cumulative_guesses(&self) -> u32 {
        self.tries.iter().map(|(_, a)| a.inner().len() as u32).sum()
    }

    /// Get the number of guesses across all solved puzzles.
    pub fn cumulative_guesses_solved(&self) -> u32 {
        self.tries
            .iter()
            .filter(|(word, attempts)| attempts.solved(word))
            .map(|(_, a)| a.inner().len() as u32)
            .sum()
    }

    /// Get the average number of guesses needed to solve a puzzle.
    ///
    /// This function does not include guesses made on puzzles that the
    /// strategy was unable to solve.
    pub fn guesses_per_solution(&self) -> f32 {
        (self.cumulative_guesses_solved() as f32) / (self.num_solved() as f32)
    }

    /// Get the number of puzzles the strategy could not solve.
    ///
    /// This function will always return a number less than or equal to
    /// [Perf::num_tried()].
    pub fn num_missed(&self) -> u32 {
        self.num_tried() - self.num_solved()
    }

    /// Get the fraction of puzzles the strategy could not solve.
    pub fn frac_missed(&self) -> f32 {
        (self.num_missed() as f32) / (self.num_tried() as f32)
    }

    /// Summarize the performance of a strategy.
    ///
    /// (This just uses the [Display] implementation.)
    pub fn summarize(&self) {
        print!("{}", self)
    }

    /// Print the strategy's summary and then output a table showing the
    /// strategy's attempts for each puzzle.
    pub fn print(&self) {
        self.summarize();
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

    /// Convert this performance record to a pre-calculated summary.
    pub fn to_summary(&self) -> PerfSummary {
        let mut histogram = [0; 6];

        self.tries
            .iter()
            .filter(|(word, attempts)| attempts.solved(word))
            .map(|(_, attempts)| attempts.inner().len())
            .filter(|&n| n <= 6)
            .for_each(|n| histogram[n] += 1);

        assert_eq!(
            histogram.iter().sum::<u32>(),
            self.cumulative_guesses_solved()
        );

        PerfSummary {
            strategy_name: &self.strategy_name,
            num_tried: self.num_tried(),
            num_solved: self.num_solved(),
            cumulative_guesses: self.cumulative_guesses(),
            histogram,
        }
    }
}

impl Display for Perf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let perf_summary = self.to_summary();
        write!(f, "{}", perf_summary)
    }
}

/// A summary of a strategy's [Perf] generated by the
/// [test harness](crate::Harness).
///
/// It is recommended to convert the [Perf] struct to this via the
/// [Perf::to_summary()] method when you want to use the performance to run
/// statistics.
pub struct PerfSummary<'a> {
    strategy_name: &'a str,
    num_tried: u32,
    num_solved: u32,
    cumulative_guesses: u32,
    histogram: [u32; 6],
}

impl<'a> PerfSummary<'a> {
    /// Get the name of the strategy that produced this performance record.
    pub fn strategy_name(&self) -> &'a str {
        self.strategy_name
    }

    /// Get the number of puzzles attempted by the strategy.
    pub fn num_tried(&self) -> u32 {
        self.num_tried
    }

    /// Get the number of puzzles solved by the strategy.
    ///
    /// This function will always return a number less than or equal to
    /// [PerfSummary::num_tried()].
    pub fn num_solved(&self) -> u32 {
        self.num_solved
    }

    /// Get the fraction of puzzles solved by the strategy.
    pub fn frac_solved(&self) -> f32 {
        (self.num_solved as f32) / (self.num_tried as f32)
    }

    /// Get the number of guesses across all puzzle attempts.
    pub fn cumulative_guesses(&self) -> u32 {
        self.cumulative_guesses
    }

    /// Get the number of guesses across all solved puzzles.
    pub fn cumulative_guesses_solved(&self) -> u32 {
        self.histogram.iter().sum::<u32>()
    }

    /// Get the average number of guesses needed to solve a puzzle.
    ///
    /// This function does not include guesses made on puzzles that the
    /// strategy was unable to solve.
    pub fn guesses_per_solution(&self) -> f32 {
        (self.cumulative_guesses_solved() as f32) / (self.num_solved as f32)
    }

    /// Get the number of puzzles the strategy could not solve.
    ///
    /// This function will always return a number less than `num_tried()`.
    pub fn num_missed(&self) -> u32 {
        self.num_tried - self.num_solved
    }

    /// Get the fraction of puzzles the strategy could not solve.
    pub fn frac_missed(&self) -> f32 {
        (self.num_missed() as f32) / (self.num_tried as f32)
    }

    /// Summarize the performance of a strategy.
    ///
    /// (This just uses the [Display] implementation.)
    pub fn summarize(&self) {
        print!("{}", self)
    }
}

impl<'a> Display for PerfSummary<'a> {
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
