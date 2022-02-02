use wordle_rs::{harness::Harness, strategy::Word, WordleError};
use wordle_strategies::{Basic, Common};

fn main() -> Result<(), WordleError> {
    let harness = Harness::new()
        .verbose()
        .add_strategy(Box::new(Common), "common")
        .add_strategy(
            Box::new(Basic::new().first_word(Word::from_str("pints").unwrap())),
            "basic_pints",
        )
        // .add_strategy(Box::new(
        //     Basic::new().first_word(Word::from_str("qajaq").unwrap()),
        // ))
        .load_baseline("basic_qajaq", None)?
        // .test_num(200);
        .test_all();
    let perfs = harness.run()?;

    perfs.print_report()
}
