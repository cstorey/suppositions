extern crate suppositions;

use suppositions::*;

#[test]
#[ignore]
fn some_approximation_of_usage() {
    property(vecs(integers())).check(|l| {
        let rev = l.iter().cloned().rev().collect::<Vec<_>>();
        let rev2 = rev.into_iter().rev().collect::<Vec<_>>();
        return rev2 == l;
    })
}
