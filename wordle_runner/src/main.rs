use wordle_rs::{harness::Harness, perf::Summary, strategy::Word};
use wordle_strategies::{Basic, Common};

fn main() {
    let harness = Harness::new()
        .verbose()
        .add_strategy(Box::new(Common))
        .add_strategy(Box::new(
            Basic::new().first_word(Word::from_str("pints").unwrap()),
        ))
        // .add_strategy(Box::new(
        //     Basic::new().first_word(Word::from_str("qajaq").unwrap()),
        // ))
        // .add_strategy(Box::new(
        //     Basic::new().first_word(Word::from_str("pints").unwrap()),
        // ))
        .test_num(1500);
    // .test_all();
    let perfs = harness.run().unwrap();
    let common_perf = &perfs[0];
    let pints_perf = &perfs[1];

    let common_summary = common_perf.to_summary();

    common_summary
        .print(
            Summary::print_options()
                .compare(pints_perf.to_summary())
                .histogram(true),
        )
        .unwrap();

    pints_perf
        .to_summary()
        .print(Summary::print_options().histogram(true))
        .unwrap();
}
