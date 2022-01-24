//! Some strategies for use with `wordle_rs`.
//!
//! Each strategy consists of a single struct, and everything you need to
//! configure the strategy should exist as a method.

mod basic;
pub use basic::Basic;

mod common;
pub use common::Common;

mod narrowing;
pub use narrowing::Narrowing;

mod common_easy;
pub use common_easy::CommonEasy;
