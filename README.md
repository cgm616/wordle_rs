`wordle_rs`
-----------

[![build](https://github.com/cgm616/wordle_rs/actions/workflows/cargo.yml/badge.svg)](https://github.com/cgm616/wordle_rs/actions/workflows/cargo.yml)
[![license](https://img.shields.io/crates/l/wordle_rs)](https://github.com/cgm616/wordle_rs/blob/master/LICENSE)
[![docs](https://img.shields.io/docsrs/wordle_rs)](https://docs.rs/wordle_rs/latest/wordle_rs/)
[![changelog](https://img.shields.io/badge/changelog--blue)](https://github.com/cgm616/wordle_rs/blob/master/CHANGELOG.md)

WARNING: this project is still unstable, so minor updates may break backwards incompatibility.

Have you ever gotten so obsessed with [Wordle](https://www.powerlanguage.co.uk/wordle/) that you wanted to evaluate different strategies programmatically? If so, you're in the right place.

This crate is a part of the `wordle_rs` project, which has three parts:
- [`wordle_rs`](https://crates.io/crates/wordle_rs), a library with tools you can use to write and evaluate your own Wordle strategies,
- [`wordle_strategies`](https://crates.io/crates/wordle_strategies), a library demonstrating a few strategies that I wrote, and
- `wordle_runner` (WIP), a binary that can run and compare Wordle strategies written with `wordle_rs`.

Please feel free to contribute your own strategies to `wordle_strategies`!

## Using `wordle_rs` to write a strategy

Add the following to your `Cargo.toml`:

```toml
[dependencies]
wordle_rs = "0.1.2"
```

Then, define a new struct and implement the `Strategy` trait for it.

```rust,ignore
use wordle_rs::Strategy;

struct MyCoolStrategy;

impl Strategy for MyCoolStrategy {
    // snip
}
```

Then, configure and run the test harness on your strategy.
You can see how to do this below.
You can also use `wordle_runner` to run your strategy for you, though this is still a work in progress.

## Running strategies from [`wordle_strategies`](https://crates.io/crates/wordle_strategies)

To run a pre-made strategy (possibly against your own!), first add the following to your `Cargo.toml`:

```toml
[dependencies]
wordle_rs = "0.1.2"
wordle_strategies = "0.1.2"
```

Then, import a strategy and run the `wordle_rs` test harness on your strategy.

## Running the `wordle_rs` test harness

Simply import the harness and configure it to run the strategies that you want to test.
You can add strategies from any location, including those that you write yourself.
The `Harness::add_strategy` method accepts anything that implements the `Strategy` trait.

```rust,ignore
use wordle_rs::{harness::Harness, WordleError};
use wordle_strategies::Common;

fn main() -> Result<(), WordleError> {
    let harness = Harness::new()
        .add_strategy(Box::new(Common), None)
        .test_num(10);
    let perfs = harness.run()?;
    perfs.print_report()?;

    Ok(())
}
```

## Using `wordle_runner`

Forthcoming.

## License

Everything in this project is licensed under the [MIT license](LICENSE).