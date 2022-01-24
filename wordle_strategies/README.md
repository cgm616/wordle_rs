`wordle_rs`
-----------

[Changelog](../CHANGELOG.md)

Have you ever gotten so obsessed with [Wordle](https://www.powerlanguage.co.uk/wordle/) that you wanted to evaluate different strategies programmatically? If so, you're in the right place.

This crate is a part of the [`wordle_rs`](https://github.com/cgm616/wordle_rs) project, which has three parts:
- `wordle_rs`, a library with tools you can use to write and evaluate your own Wordle strategies,
- `wordle_strategies`, a library demonstrating a few strategies that I wrote, and
- `wordle_runner` (WIP), a binary that can run and compare Wordle strategies written with `wordle_rs`.

Please feel free to contribute your own strategies to `wordle_strategies`!

## Running strategies from `wordle_strategies`

To run a strategy from this crate, first add the following to your `Cargo.toml`:

```toml
[dependencies]
wordle_rs = "0.1"
wordle_strategies = "0.1"
```

Then, import a strategy and run the `wordle_rs` test harness on your strategy.

```rust
use wordle_rs::{harness::Harness};
use wordle_strategies::Common;

fn main() {
    let harness = Harness::new()
        .add_strategy(Box::new(Common))
        .test_num(10);
    let perfs = harness.run_and_summarize();
}
```

## License

Everything in this project is licensed under the [MIT license](../LICENSE).