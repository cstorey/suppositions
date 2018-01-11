//! This module contains the underlying data generation and shrinking
//! mechanism. The main type is the `InfoPool`, which represents a pool of
//! random bytes that can be observed via the `InfoTap` object (obtained via
//! `InfoPool#tap`).
//!
//! Also manages the shrinking process (see [`minimize`](fn.minimize.html)).

mod source;
mod shrinkers;
pub use self::source::*;
pub use self::shrinkers::*;
