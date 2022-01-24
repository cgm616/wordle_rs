`wordle_rs`
-----------

[![build](https://github.com/cgm616/wordle_rs/actions/workflows/cargo.yml/badge.svg)](https://github.com/cgm616/wordle_rs/actions/workflows/cargo.yml)
[![license](https://img.shields.io/crates/l/wordle_rs)](https://github.com/cgm616/wordle_rs/blob/master/LICENSE)
[![docs](https://img.shields.io/docsrs/wordle_rs)](https://docs.rs/wordle_rs/latest/wordle_rs/)
[![changelog](https://img.shields.io/badge/changelog--blue)](https://github.com/cgm616/wordle_rs/blob/master/CHANGELOG.md)

Have you ever gotten so obsessed with [Wordle](https://www.powerlanguage.co.uk/wordle/) that you wanted to evaluate different strategies programmatically? If so, you're in the right place.

This crate is a part of the `wordle_rs` project, which has three parts:
- `wordle_rs`, a library with tools you can use to write and evaluate your own Wordle strategies,
- `wordle_strategies`, a library demonstrating a few strategies that I wrote, and
- `wordle_runner`, a binary that can run and compare Wordle strategies written with `wordle_rs`.

Please feel free to contribute your own strategies to `wordle_strategies`!

## Using `wordle_rs` to write and evaluate a strategy

Add the following to your `Cargo.toml`:

```toml
[dependencies]
wordle_rs = "0.1"
```

Then, define a new struct and implement the `Strategy` trait for it.

```rust,ignore
use wordle_rs::Strategy;

struct MyCoolStrategy;

impl Strategy for MyCoolStrategy {
    // snip
}
```

Finally, configure and run the test harness on your strategy.

```rust,ignore
use wordle_rs::{harness::Harness};

fn main() {
    let harness = Harness::new()
        .add_strategy(Box::new(MyCoolStrategy))
        .test_all();
    let perfs = harness.run_and_summarize();
}
```

Or, use `wordle_runner` to run the strategy for you!

## Using `wordle_runner`

Forthcoming.

## License

Everything in this project is licensed under the [MIT license](LICENSE).