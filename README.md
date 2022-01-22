`wordle_rs`
-----------

[Changelog](CHANGELOG.md)

Have you ever gotten so obsessed with [Wordle](https://www.powerlanguage.co.uk/wordle/) that you wanted to evaluate different strategies programmatically? If so, you're in the right place.

`wordle_rs` has three parts:
- `wordle_perf`, a library with tools you can use to write and evaluate your own Wordle strategies,
- `wordle_strategies`, a library demonstrating few strategies that I wrote, and
- `wordle_runner`, a binary that can run and compare Wordle strategies written with `wordle_perf`.

Please feel free to contribute your own strategies to `wordle_strategies`!

## Using `wordle_perf` to write and evaluate a strategy

Add the following to your `Cargo.toml`:

```toml
[dependencies]
wordle_perf = "0.1"
```

Then, define a new struct and implement the `Strategy` trait for it.

```rust
use wordle_rs::Strategy;

struct MyCoolStrategy;

impl Strategy for MyCoolStrategy {
    // snip
}
```

Finally, configure and run the test harness on your strategy.

```rust
use wordle_perf::{harness::Harness};

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

Everything in this repository is licensed under the [MIT license](LICENSE).