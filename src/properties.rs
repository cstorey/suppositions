
use std::fmt;
use std::panic;

use data::*;
use generators::*;

/// Configuration that allows the user to override how many tests, skipped-tests etc.
/// are permitted.
#[derive(Debug, Clone)]
pub struct CheckConfig {
    num_tests: usize,
    max_skips: usize,
}

impl Default for CheckConfig {
    fn default() -> Self {
        let num_tests = 100;
        CheckConfig {
            num_tests: num_tests,
            max_skips: num_tests * 10,
        }
    }
}
impl CheckConfig {
    /// Overrides how many tests (either failing or successful) are executed.
    pub fn num_tests(&self, num_tests: usize) -> Self {
        CheckConfig {
            num_tests,
            ..self.clone()
        }
    }
    /// Overrides how many times the generators can skip generation before we
    /// abort the test run.
    pub fn max_skips(&self, max_skips: usize) -> Self {
        CheckConfig {
            max_skips,
            ..self.clone()
        }
    }
    /// This is the main entry point for users of the library.
    pub fn property<G: Generator>(&self, gen: G) -> Property<G> {
        Property {
            config: self.clone(),
            gen: gen,
        }
    }
}

/// This represents a configuration for a particular test, ie: a set of generators
/// and a (currently fixed) set of test parameters.
pub struct Property<G> {
    config: CheckConfig,
    gen: G,
}

/// This represents something that a check can return.
pub trait CheckResult {
    /// Check whether this result witnesses a failure.
    fn is_failure(&self) -> bool;
}

/// See [`CheckConfig::property`](struct.CheckConfig.html#method.property)
/// Initiates a test with default configuration.
pub fn property<G: Generator>(gen: G) -> Property<G> {
    CheckConfig::default().property(gen)
}

#[derive(Debug, Clone, Default)]
struct Stats {
    tests_run: usize,
    items_skipped: usize,
}

impl<G: Generator> Property<G>
where
    G::Item: fmt::Debug,
{
    /// Use this function to sepecify the thing you wish to check. Because we include the
    /// debug representation of the input and the output within the
    pub fn check<R: CheckResult + fmt::Debug, F: Fn(G::Item) -> R>(self, subject: F) {
        let mut stats = Stats::default();
        while stats.tests_run < self.config.num_tests {
            trace!("Tests run: {}; skipped:{}", stats.tests_run, stats.items_skipped);
            self.try_one(&mut stats, &subject)
        }
        trace!("Completing okay");
    }

    fn try_one<R: CheckResult + fmt::Debug, F: Fn(G::Item) -> R>(
            &self, stats: &mut Stats, subject: &F) {
        let mut src = RngSource::new();
        let mut pool = InfoRecorder::new(&mut src);
        let result = self.gen.generate(&mut pool);
        trace!("Pool: {:?}", pool);
        let pool = pool.into_pool();
        match result {
            Ok(arg) => {
                stats.tests_run += 1;
                self.try_example(subject, pool, arg)
            }
            Err(DataError::SkipItem) => {
                stats.items_skipped += 1;
                trace!("Skip");

                if stats.items_skipped >= self.config.max_skips {
                    panic!(
                        "Could not finish on {}/{} tests (have skipped {} times)",
                        stats.tests_run,
                        self.config.num_tests,
                        stats.items_skipped
                    );
                }
            }
            Err(e) => {
                debug!("Data generation failure: {:?}", e);
            }
        }
    }

    fn try_example<R: CheckResult + fmt::Debug, F: Fn(G::Item) -> R>(&self, subject: &F, pool: InfoPool, arg: G::Item) {
        let res = Self::attempt(&subject, arg);
        trace!(
            "Result: {:?} -> {:?}",
            self.gen.generate(&mut pool.replay()),
            res
        );
        if res.is_failure() {
            let minpool = find_minimal(
                &self.gen,
                pool,
                |v| {
                    trace!("Shrink attempt: {:?}", v);
                    let res = Self::attempt(&subject, v);
                    trace!("Shrink attempt -> {:?}", res);
                    res.is_failure()
                    },
            );
            trace!("Minpool: {:?}", minpool);
            trace!("Values: {:?}", self.gen.generate(&mut minpool.replay()));
            panic!(
                "Predicate failed for argument {:?}; check returned {:?}",
                self.gen.generate(&mut minpool.replay()),
                res
            )
        }

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
