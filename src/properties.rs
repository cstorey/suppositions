
use std::fmt;

use data::*;
use generators::*;

const NUM_TESTS: usize = 100;
const MAX_SKIPS: usize = NUM_TESTS * 10;
const DEFAULT_POOL_SIZE: usize = 1024;

/// This represents a configuration for a particular test, ie: a set of generators
/// and a (currently fixed) set of test parameters.
pub struct Property<G> {
    gen: G,
}

/// This represents something that a check can return.
pub trait CheckResult {
    /// Check whether this result witnesses a failure.
    fn is_failure(&self) -> bool;
}

/// This is the main entry point for users of the library.
pub fn property<G>(gen: G) -> Property<G> {
    Property { gen: gen }
}

impl<G: Generator> Property<G>
where
    G::Item: fmt::Debug,
{
    /// Use this function to sepecify the thing you wish to check.
    pub fn check<R: CheckResult, F: Fn(G::Item) -> R>(self, check: F) {
        let mut tests_run = 0usize;
        let mut items_skipped = 0usize;
        while tests_run < NUM_TESTS {
            let pool = InfoPool::random_of_size(DEFAULT_POOL_SIZE);
            match self.gen.generate(&mut pool.tap()) {
                Ok(arg) => {
                    let res = check(arg);
                    tests_run += 1;
                    if res.is_failure() {
                        let minpool = find_minimal(&self.gen, pool, |v| check(v).is_failure());
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

impl CheckResult for bool {
    fn is_failure(&self) -> bool {
        !self
    }
}

impl<O, E> CheckResult for Result<O, E> {
    fn is_failure(&self) -> bool {
        self.is_err()
    }
}
