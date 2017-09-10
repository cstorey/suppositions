extern crate suppositions;

use suppositions::*;
use suppositions::generators::*;

#[test]
#[ignore]
fn some_approximation_of_usage() {
    property(vecs(booleans())).check(|l| {
        let rev = l.iter().cloned().rev().collect::<Vec<_>>();
        let rev2 = rev.into_iter().rev().collect::<Vec<_>>();
        return rev2 == l;
    })
}
