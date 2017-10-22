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

## ???
* Generators
  * Collections
    * [ ] Min lengths
    * [ ] Max
* [ ] Examples ("inspired" by hedgehog/hypothesis/etc)
  * [ ] https://begriffs.com/posts/2017-01-14-design-use-quickcheck.html?hn=1
  * [ ] http://matt.might.net/articles/quick-quickcheck/
  * [ ] https://github.com/BurntSushi/quickcheck
  * [ ] https://github.com/hedgehogqa/haskell-hedgehog/tree/master/hedgehog-example/test/Test/Example
* [X] Configure number of runs/discards
* [ ] Confgigure pool size (still an arbitrarily specified value)
* [ ] Auto size pool (lazily create pool from rand source and cache into Vec)
  * Return zeroes (like theft) at end of input?
    * This may need to vary between first-run (extend with random) and shrinking (zero fill)
* [ ]  Avoid re-testing the same value
  * [ ] Keep previous value around; compare against it.
  * [ ] (eg: via scalable bloom/Cuckoo filters)
  * [ ] Make optional, to avoid extra Ord/Hash constraint
* [ ] Stats on runs/skips/fails on random/shrinkage
* [ ] Derive input based on trace of execution? (ie: lineage driven fault injection)
* [ ] Track which bytes (regions) are used for which generators; use this in shrinking
* [ ]
  Use monad vs. applicative style interfaces to infer causal relations between
  regions; Means that iff B is causally dependent upon A (eg: `A = bools();
  B = someValue(); if A.generate(g) { out.emit(B.generate(g)) }`) We should
  avoid deleting A `xor` B.
* [ ] CoArbitrary equivalent?
