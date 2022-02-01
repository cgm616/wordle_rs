use wordle_rs::{
    harness::Harness,
    perf::Summary,
    strategy::{stupid::Stupid, Word},
    WordleError,
};
use wordle_strategies::{Basic, Common};

fn main() -> Result<(), WordleError> {
    let harness = Harness::new()
        .verbose()
        .add_strategy(Box::new(Common))
        .add_strategy(Box::new(
            Basic::new().first_word(Word::from_str("pints").unwrap()),
        ))
        // .add_strategy(Box::new(
        //     Basic::new().first_word(Word::from_str("qajaq").unwrap()),
        // ))
        .add_saved_baseline("basic_qajaq", None)?
        // .test_num(200);
        .test_all();
    let perfs = harness.run().unwrap();

    perfs.print_report()
}
