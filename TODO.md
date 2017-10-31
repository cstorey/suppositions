## 0.1.0

* Generators
  * [X] Integers & other Primitives
  * [X] Floats
  * [X] Result
  * [X] Weighted coin
  * Collections
    * [X] Average lengths via reservoir-esque sampling
  * [X] Argument emulation via tuple of generator is a generator of tuples
  * combinators
    * [X] filter
    * [X] map
    * [X] filter_map
    * [ ] flat_map (ie: a thing that varies the generator based on a value)?)
  * [X] One of a set of generators
  * [X] Const value
* [X] Skip data items
* [ ] TestResult trait for
  * [X] bool.
  * [X] Result.
  * [X] unit
* [X] Catch panics and extract panic message

## 0.1.1
* [X] Configure number of runs/discards
* [X] Auto size pool (lazily create pool from rand source and cache into Vec)
  * Return zeroes (like theft) at end of input?
    * This may need to vary between first-run (extend with random) and shrinking (zero fill)
## 0.1.2

* Shrink via removal on power-of two boundaries.

## 0.1.3

* [X] N-ary tuples
* [X] Avoid needing trait-objects in the main `Generator` trait, and create `GeneratorObject` to replace boxed usage.

## 0.1.4
* [X] `lazy` combinator to make recursion at least possible.

## ???
* [ ] Track which bytes (regions) are used for which generators; use this in shrinking

## Backlog

* Generators
  * Collections
    * [ ] Min lengths
    * [ ] Max
  * Recursion
    * Helpers to limit recursion depth
    * Or just weighted one-of?
* [ ] Examples ("inspired" by hedgehog/hypothesis/etc)
  * [ ] http://matt.might.net/articles/quick-quickcheck/
  * [ ] https://github.com/BurntSushi/quickcheck
  * [ ] https://github.com/hedgehogqa/haskell-hedgehog/tree/master/hedgehog-example/test/Test/Example
  * [ ] https://fsharpforfunandprofit.com/posts/property-based-testing-2/
* [ ]  Avoid re-testing the same value
  * [ ] Keep previous value around; compare against it.
  * [ ] (eg: via scalable bloom/Cuckoo filters)
  * [ ] Make optional, to avoid extra Ord/Hash constraint
* [ ] Stats on runs/skips/fails on random/shrinkage
* [ ] Derive input based on trace of execution? (ie: lineage driven fault injection)
* [ ]
  Use monad vs. applicative style interfaces to infer causal relations between
  regions; Means that iff B is causally dependent upon A (eg: `A = bools();
  B = someValue(); if A.generate(g) { out.emit(B.generate(g)) }`) We should
  avoid deleting A `xor` B.
* [ ] CoArbitrary equivalent?

# Implementation notes:
## `generators::one_of`
Rather than boxing values, maybe consider using a type-level thing? End with an impl that uses boxing (in `generators::boxed`) and a type-level induction based one (`generators::unboxed`) that sadly has horrible type errors.