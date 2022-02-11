use wordle_rs::{
    harness::{Harness, WasmWrapper},
    strategy::Word,
    WordleError,
};
use wordle_strategies::{Basic, Common};

const WASM_PATH: &'static str = "../target/wasm32-unknown-unknown/wasm/opt2.wasm";
const CRATE_PATH: &'static str = "../wordle_strategies";

fn main() -> Result<(), WordleError> {
    env_logger::builder().format_timestamp(None).init();

    let words = wordle_rs::words::ANSWERS
        .iter()
        .map(|&i| Word::from_index(i))
        .collect::<Result<Vec<_>, _>>()?;

    let harness = Harness::new()
        .verbose(true)
        .parallel(false)
        .add_strategy(Box::new(Common), None)
        .add_strategy(
            Box::new(WasmWrapper::new_from_crate(CRATE_PATH, "common")?),
            None,
        )
        // .add_strategy(
        //     Box::new(Basic::new().first_word(Word::from_str("aahed").unwrap())),
        //     None,
        // )
        // .add_strategy(
        //     Box::new(WasmWrapper::new_from_wasm(WASM_PATH, "basic")?),
        //     None,
        // )
        // .add_strategy(Box::new(
        //     Basic::new().first_word(Word::from_str("qajaq").unwrap()),
        // ))
        .load_baseline("basic_qajaq", None)?
        // .test_num(200);
        .test_all();

    let perfs = harness.debug_run(Some(&words))?;
    // let perfs = harness.run()?;

    perfs.print_report()
}
