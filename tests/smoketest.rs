extern crate suppositions;

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
    property((booleans())).check(|test| test)
}
