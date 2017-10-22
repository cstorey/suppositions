extern crate suppositions;
extern crate env_logger;
use std::cmp::min;
use suppositions::*;
use suppositions::generators::*;


// Run this with:
// ```
// cargo run --example die-hard
// ```
// You should see output of the form:
// ```
// thread 'main' panicked at 'Predicate failed for argument
// Ok([FillBigJug, BigToSmall, EmptySmallJug, BigToSmall, FillBigJug, BigToSmall]);
// check returned Ok(Err(State { big: 4, small: 3 }))', src/properties.rs:56:24
// ```


#[derive(Debug, Clone)]
pub enum Op {
    FillSmallJug,
    FillBigJug,
    EmptySmallJug,
    EmptyBigJug,
    SmallToBig,
    BigToSmall,
}

#[derive(Debug, Default, Clone)]
pub struct State {
    big: usize,
    small: usize,
}

impl State {
    fn apply(&mut self, op: &Op) {
        match op {
            &Op::FillSmallJug => {
                self.small = 3;
            }
            &Op::FillBigJug => self.big = 5,
            &Op::EmptySmallJug => self.small = 0,
            &Op::EmptyBigJug => self.big = 0,
            &Op::SmallToBig => {
                let old = self.clone();
                self.big = min(old.big + self.small, 5);
                self.small -= self.big - old.big
            }

            &Op::BigToSmall => {
                let old = self.clone();
                self.small = min(old.big + self.small, 3);
                self.big -= self.small - old.small
            }
        }
    }

    fn assert_invariants(&self) {
        assert!(self.big <= 5);
        assert!(self.small <= 3);
    }
    fn finished(&self) -> bool {
        self.big == 4
    }
}

fn ops() -> Box<Generator<Item = Op>> {
    let g = one_of(consts(Op::FillSmallJug))
        .or(consts(Op::FillBigJug))
        .or(consts(Op::EmptySmallJug))
        .or(consts(Op::EmptyBigJug))
        .or(consts(Op::SmallToBig))
        .or(consts(Op::BigToSmall));
    Box::new(g)
}

fn main() {
    env_logger::init().expect("env_logger::init");
    CheckConfig::default()
        .num_tests(10000)
        .property(vecs(ops()).mean_length(1000))
        .check(|xs| {
            let mut sts = Vec::new();
            let mut st = State::default();
            for o in xs.iter() {
                st.apply(o);
                sts.push((o.clone(), st.clone()));
                st.assert_invariants();
                if st.finished() {
                    return Err(st);
                }
            }
            return Ok(());
        });

    panic!("No solution found")
}
