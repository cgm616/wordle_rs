use wordle_perf::harness::Harness;
use wordle_strategies::Basic;
use wordle_strategies::Stupid;

fn main() {
    let harness = Harness::new()
        .verbose()
        .add_strategy(Box::new(Basic::new().bad_start()))
        .add_strategy(Box::new(Basic::new().good_start()))
        // .test_num(200);
        .test_all();
    let _perfs = harness.run_and_summarize();
}
