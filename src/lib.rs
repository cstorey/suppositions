extern crate rand;
extern crate hex_slice;

pub mod data;
pub mod generators;

use data::*;
use generators::*;


pub struct Property<G> {
    gen: G,
}

pub fn property<G>(gen: G) -> Property<G> {
    Property { gen }
}

impl<G: Generator> Property<G> {
    pub fn check<F: Fn(G::Item) -> bool>(self, check: F) {
        let mut pool = InfoPool::default();

        for _i in 0..100 {
            let arg = self.gen.generate(&mut pool).expect("???");
            let _res = check(arg);
            // Something something
            unimplemented!()
        }
    }
}
