use wordle_rs::{
    harness::Harness,
    perf::Summary,
    strategy::{stupid::Stupid, Word},
};
use wordle_strategies::{Basic, Common};

fn main() {
    let harness = Harness::new()
        .verbose()
        .add_strategy(Box::new(Common))
        .add_strategy(Box::new(
            Basic::new().first_word(Word::from_str("pints").unwrap()),
        ))
        .and_baseline()
        .add_strategy(Box::new(
            Basic::new().first_word(Word::from_str("qajaq").unwrap()),
        ))
        .test_num(200);
    // .test_all();
    let perfs = harness.run().unwrap();

    perfs.print_report();
}
