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
        .add_strategy(Box::new(
            Basic::new().first_word(Word::from_str("qajaq").unwrap()),
        ))
        .test_num(20);
    // .test_all();
    let perfs = harness.run().unwrap();

    let basic_pints_perf = &perfs[2];
    let basic_summary = basic_pints_perf.to_summary();

    for perf in &perfs {
        let summary = perf.to_summary();
        match summary.print(
            Summary::print_options()
                .compare(&basic_summary)
                .histogram(true),
        ) {
            Ok(()) => {}
            Err(e) => {
                summary
                    .print(Summary::print_options().histogram(true))
                    .unwrap();
            }
        }
    }
}
