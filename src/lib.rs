extern crate rand;
extern crate hex_slice;
#[macro_use]
extern crate log;

use std::fmt;

pub mod data;
pub mod generators;

use data::*;
use generators::*;

const NUM_TESTS: usize = 100;
const DEFAULT_POOL_SIZE: usize = 1024;

pub struct Property<G> {
    gen: G,
}

pub fn property<G>(gen: G) -> Property<G> {
    Property { gen }
}

impl<G: Generator> Property<G>
where
    G::Item: fmt::Debug,
{
    pub fn check<F: Fn(G::Item) -> bool>(self, check: F) {
        for _i in 0..NUM_TESTS {
            let pool = InfoPool::random_of_size(DEFAULT_POOL_SIZE);
            if let Ok(arg) = self.gen.generate(&mut pool.tap()) {
                let res = check(arg);
                assert!(
                    res,
                    "Predicate failed for argument {:?}",
                    self.gen.generate(&mut pool.tap())
                )
            } else {
                println!("Not enough pool")
            }
        }
    }
}
