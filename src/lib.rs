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

extern crate rand;
extern crate hex_slice;
#[macro_use]
extern crate log;

use std::fmt;

pub mod data;
pub mod generators;

use data::*;
use generators::*;

pub use generators::Generator;

const NUM_TESTS: usize = 100;
const MAX_SKIPS: usize = NUM_TESTS * 10;
const DEFAULT_POOL_SIZE: usize = 1024;

pub struct Property<G> {
    gen: G,
}

pub fn property<G>(gen: G) -> Property<G> {
    Property { gen: gen }
}

impl<G: Generator> Property<G>
where
    G::Item: fmt::Debug,
{
    pub fn check<F: Fn(G::Item) -> bool>(self, check: F) {
        let mut tests_run = 0usize;
        let mut items_skipped = 0usize;
        while tests_run < NUM_TESTS {
            let pool = InfoPool::random_of_size(DEFAULT_POOL_SIZE);
            match self.gen.generate(&mut pool.tap()) {
                Ok(arg) => {
                    let res = check(arg);
                    tests_run += 1;
                    if !res {
                        let minpool = find_minimal(&self.gen, pool, |v| !check(v));
                        assert!(
                            false,
                            "Predicate failed for argument {:?}",
                            self.gen.generate(&mut minpool.tap())
                        )
                    }
                }
                Err(DataError::SkipItem) => {
                    items_skipped += 1;
                    if items_skipped >= MAX_SKIPS {
                        panic!(
                            "Could not finish on {}/{} tests (have skipped {} times)",
                            tests_run,
                            NUM_TESTS,
                            items_skipped
                        );
                    }
                }
                Err(e) => {
                    debug!("{:?}", e);
                }
            }
        }
    }
}
