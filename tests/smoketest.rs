extern crate suppositions;
#[macro_use]
extern crate log;
extern crate env_logger;

use suppositions::*;
use suppositions::generators::*;

#[test]
fn some_approximation_of_usage() {
    property(vecs(booleans())).check(|l| {
        let rev = l.iter().cloned().rev().collect::<Vec<_>>();
        let rev2 = rev.into_iter().rev().collect::<Vec<_>>();
        return rev2 == l;
    })
}

// In this case, we reverse the last three items.
#[test]
#[should_panic]
fn some_approximation_of_failing_example() {
    env_logger::init().unwrap_or(());
    property(vecs(booleans())).check(|l| {
        let rev = l.iter().cloned().rev().take(3).collect::<Vec<_>>();
        let rev2 = rev.into_iter().rev().collect::<Vec<_>>();
        info!("in:{:?}; out:{:?}; ok? {:?}", l, rev2, &rev2 == &l);
        return rev2 == l;
    })
}

#[test]
#[should_panic]
fn trivial_failure() {
    property((booleans())).check(|_| false)
}

#[test]
fn trivial_pass() {
    property((booleans())).check(|_| true)
}

#[test]
#[should_panic]
fn value_dependent() {
    property(vecs(booleans())).check(|v| {
        println!("Check: {:?}", v);
        !v.into_iter().any(|t| t)
    })
}
