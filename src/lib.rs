//! A property testing library, inspired by hedgehog and
//! hypothesis. Wheras conventional unit testing is usually designed
//! to test specific examples module usage, property testing allows you to
//! specify invariants, or properties that hold for a given set of inputs.
//! When we find an example of an input that causes a failure, then we
//! automatically find the smallest failing input.
//!
//! ### Why use this.
//!
//! #### Flexibility through combinators
//! Rather taking Quickcheck's usual approach of defining an arbitrary trait,
//! and implementing that for each kind of data we want to create,
//!
//! #### Shrinking for Free.
//! Instead of having to define a shrinking mechanism for each individual type
//! that we want to create, we rely on the fact our data generation mechanism
//! draws data from an underlying pool of bytes (eg: a random source), and
//! that generators will generally create smaller values for smaller inputs.
//! (See the [data module](data/index.html) for more on that).

//! ### Examples
//! One common way to define a property is to check that running a piece
//! of data then it's inverse results in the original value. The canonical
//! example of this would be reversing a list of booleans:

//!
//! ```rust
//! #[test]
//! fn reversing_a_vector_twice_results_in_identity() {
//!     property(vecs(booleans())).check(|l| {
//!         let rev = l.iter().cloned().rev().collect::<Vec<_>>();
//!         let rev2 = rev.into_iter().rev().collect::<Vec<_>>();
//!         return rev2 == l;
//!     })
//! }
//! ```

//! Another common example is to verify that values can be round-tripped through a serialisation mechanism.

#![deny(warnings)]
#![warn(missing_docs)]

#[cfg(test)]
extern crate env_logger;
extern crate hex_slice;
#[macro_use]
extern crate log;
extern crate rand;

pub mod data;
pub mod generators;
mod properties;

pub use properties::*;
