
use std::fmt;
use std::panic;

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
    pub fn check<R: CheckResult + fmt::Debug, F: Fn(G::Item) -> R>(self, subject: F) {
        let mut tests_run = 0usize;
        let mut items_skipped = 0usize;
        while tests_run < NUM_TESTS {
            let pool = InfoPool::random_of_size(DEFAULT_POOL_SIZE);
            trace!("Tests run: {}; skipped:{}", tests_run, items_skipped);
            match self.gen.generate(&mut pool.tap()) {
                Ok(arg) => {
                    let res = Self::attempt(&subject, arg);
                    trace!(
                        "Result: {:?} -> {:?}",
                        self.gen.generate(&mut pool.tap()),
                        res
                    );
                    tests_run += 1;
                    if res.is_failure() {
                        let minpool = find_minimal(
                            &self.gen,
                            pool,
                            |v| Self::attempt(&subject, v).is_failure(),
                        );
                        panic!(
                            "Predicate failed for argument {:?}; check returned {:?}",
                            self.gen.generate(&mut minpool.tap()),
                            res
                        )
                    }
                }
                Err(DataError::SkipItem) => {
                    trace!("Skip: {:?}", self.gen.generate(&mut pool.tap()));
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
                    trace!("Gen failure: {:?}", self.gen.generate(&mut pool.tap()));
                    debug!("{:?}", e);
                }
            }
        }
        trace!("Completing okay");
    }

    fn attempt<R: CheckResult, F: Fn(G::Item) -> R>(subject: F, arg: G::Item) -> Result<R, String> {
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| subject(arg)));
        match res {
            Ok(r) => Ok(r),
            Err(err) => {
                let msg = if let Some(s) = err.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = err.downcast_ref::<String>() {
                    s.to_string()
                } else {
                    format!("Unrecognised panic result: {:?}", err)
                };
                Err(msg)
            }
        }
    }
}

impl CheckResult for bool {
    fn is_failure(&self) -> bool {
        !self
    }
}

impl<O: CheckResult, E> CheckResult for Result<O, E> {
    fn is_failure(&self) -> bool {
        self.as_ref().map(|r| r.is_failure()).unwrap_or(true)
    }
}

impl CheckResult for () {
    fn is_failure(&self) -> bool {
        false
    }
}
