#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod strategies;

pub mod util;

pub use strategies::*;

#[cfg(target_family = "wasm")]
pub mod wasm;
