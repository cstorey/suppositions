//! This module describes how data gets generated from the underlying representation
//! in the [`suppositions::data`](../data/index.html) module.

mod collections;
mod core;
mod numbers;
mod tuples;

pub use self::collections::*;
pub use self::core::*;
pub use self::numbers::*;
pub use self::tuples::*;
