extern crate env_logger;
#[macro_use]
extern crate log;
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

// In this case, we reverse the last three items.
#[test]
#[should_panic(expected = "Predicate failed for argument ")]
fn some_approximation_of_failing_example() {
    env_logger::init().unwrap_or(());
    property(vecs(booleans())).check(|l| {
        let rev = l.iter().cloned().rev().take(3).collect::<Vec<_>>();
        let rev2 = rev.into_iter().rev().collect::<Vec<_>>();
        info!("in:{:?}; out:{:?}; ok? {:?}", l, rev2, &rev2 == &l);
        return rev2 == l;
    })
}

// http://matt.might.net/articles/quick-quickcheck/
#[test]
#[should_panic(expected = "Predicate failed for argument ")]
fn mersenne_conjecture() {
    env_logger::init().unwrap_or(());
    fn is_prime(n: u64) -> bool {
        match n {
            0 | 1 => false,
            2 => true,
            n => !(2..n - 1).any(|q| (n % q) == 0),
        }
    }

    // Only check small primes.
    property(u8s().filter(|&n: &u8| n < 16).filter(|&n: &u8| {
        debug!("mersenne_conjecture n: {}", n);
        let primep = is_prime(n as u64);
        debug!("mersenne_conjecture n: {}; prime? {}", n, primep);
        n < 64 && primep
    })).check(|n| is_prime((1u64 << n) - 1))
}

#[test]
#[should_panic(expected = "Predicate failed for argument ")]
fn trivial_failure() {
    env_logger::init().unwrap_or(());
    property((booleans())).check(|_| false)
}

#[test]
fn trivial_pass() {
    property((booleans())).check(|_| true)
}

#[test]
#[should_panic(expected = "Predicate failed for argument ")]
fn value_dependent() {
    property(vecs(booleans())).check(|v| {
        println!("Check: {:?}", v);
        !v.into_iter().any(|t| t)
    })
}

#[test]
#[should_panic(expected = "Predicate failed for argument ")]
fn trivial_result_failure() {
    property((booleans())).check(|_| -> Result<(), ()> { Err(()) })
}

#[test]
#[should_panic(expected = "horrible failure")]
fn trivial_result_includes_failing_result() {
    property((booleans())).check(|_| -> Result<(), &'static str> { Err("horrible failure") })
}

#[test]
fn trivial_result_pass() {
    property((booleans())).check(|_| -> Result<(), ()> { Ok(()) })
}

#[test]
#[should_panic(expected = "Predicate failed for argument ")]
fn trivial_panic_failure() {
    property((booleans())).check(|_| -> () { panic!("Big bad boom") })
}

#[test]
#[should_panic(expected = "Big bad boom")]
fn panic_includes_failure_message() {
    property((booleans())).check(|_| -> () { panic!("Big bad boom") })
}

/*
Currently fails with:
```
    ... 'Predicate failed for argument Ok(72057594037927936); check returned Ok(false)'...
    note: Panic did not include expected string '123457890'
```

This occurs because:
 * We shrink by removal before we shrink by value
 * We backfill empty chunks from the iterator with zeroes
 * So, this fails on 72057594037927936, or 0x100000000000000 with a single-byte pool.
*/
#[ignore]
#[test]
#[should_panic(expected = "12345")]
fn panic_includes_minimal_example_padding_error() {
    env_logger::init().unwrap_or(());
    property(u64s()).check(|n| n < 12345);
}

/*
Currently fails with:
```
---- panic_includes_minimal_example_rounding_errors stdout ----
    ... 'Predicate failed for argument Ok(1245187); check returned Ok(false)'...
    note: Panic did not include expected string '1234567'
```

This occurrs because:
 * We shrink each individual pool byte in turn,
 * So 0x101 may fail, but 0x001 won't, even if 0x0f1 would.
*/

#[ignore]
#[test]
#[should_panic(expected = "1234567")]
fn panic_includes_minimal_example_rounding_errors() {
    env_logger::init().unwrap_or(());
    property(u64s()).check(|n| !((n & 1 == 1) && n >= 1234567));
}
