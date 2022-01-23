use wordle_rs::{harness::Harness, strategy::Word};
use wordle_strategies::{Basic, Common};

fn main() {
    let harness = Harness::new()
        .verbose()
        .add_strategy(Box::new(Common))
        .add_strategy(Box::new(
            Basic::new().first_word(Word::from_str("qajaq").unwrap()),
        ))
        .add_strategy(Box::new(
            Basic::new().first_word(Word::from_str("pints").unwrap()),
        ))
        // .test_num(2000);
        .test_all();
    let _perfs = harness.run_and_summarize();

    // for perf in perfs {
    //     perf.print();
    //     println!();
    // }
}
