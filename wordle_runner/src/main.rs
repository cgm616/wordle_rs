use wordle_rs::{harness::Harness, strategy::Word, WordleError};
use wordle_strategies::{Basic, Common};

fn main() -> Result<(), WordleError> {
    let harness = Harness::new()
        .verbose(true)
        .parallel(true)
        .add_strategy(Box::new(Common), None)
        .add_strategy(
            Box::new(Basic::new().first_word(Word::from_str("qajaq").unwrap())),
            None,
        )
        // .add_strategy(Box::new(
        //     Basic::new().first_word(Word::from_str("qajaq").unwrap()),
        // ))
        .load_baseline("basic_pints", None)?
        // .test_num(200);
        .test_all();
    let perfs = harness.run()?;

    perfs.print_report()
}
