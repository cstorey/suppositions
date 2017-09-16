## 0.1

* [ ] Generators
  * [X] Integers & other Primitives
  * [X] Floats
  * [ ] Result
  * [ ] Weighted coin
  * [ ] FromIterator having things
    * [ ] Min/max/average lengths via reservoir-esque sampling
  * [ ] Argument emulation via tuple of generator is a generator of tuples
  * [ ] combinators
    * [X] filter
    * [ ] map
    * [X] filter_map
    * [ ] flat_map (ie: a thing that varies the generator based on a value)?)
  * [ ] One of a set of generators
  * [X] Const value
* [ ] Catch panics and extract panic message
* [ ] Skip data items
* [ ] Examples ("inspired" by hedgehog/hypothesis/etc)
  * [ ] https://begriffs.com/posts/2017-01-14-design-use-quickcheck.html?hn=1
  * [ ] http://matt.might.net/articles/quick-quickcheck/
  * [ ] https://github.com/BurntSushi/quickcheck
  * [ ] https://github.com/hedgehogqa/haskell-hedgehog/tree/master/hedgehog-example/test/Test/Example
* [ ] TestResult trait for unit/bool/Result.
* [ ] Configure number of runs/discards
* [ ] Confgigure pool size (still an arbitrarily specified value)
* [ ] Auto size pool (lazily create pool from rand source and cache into Vec)
* [ ]  Avoid re-testing the same value 
  * [ ] Keep previous value around; compare against it.
  * [ ] (eg: via scalable bloom/Cuckoo filters)
  * [ ] Make optional, to avoid extra Ord/Hash constraint
* [ ] Stats on runs/skips/fails on random/shrinkage
* [ ] Derive input based on trace of execution? (ie: lineage driven fault injection)
* [ ] CoArbitrary equivalent?
