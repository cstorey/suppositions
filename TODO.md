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

## Shrinking tracking.
This then leaves us with the problem of ... how to keep track of usage on subsequent shrinks. Create region tracker as a wrapper?

```rust
enum Extent {
  Leaf(usize), // Leaf Size
  Branch(Vec<Extent>),
}
struct ExtentTracker<I> {
  src: I,
}

// Top level consumer
fn stuff() {
  let mut tracker = ExtendTracker::new(pool.replay());
  self.gen.generate(&mut tracker);
  let root_extent = tracker.root();
}

// Consumer
impl<G: Generator> Generator for Foo<G> {
  fn generate<I: Iterator<Item = u8>>(&self, src: &mut ExtentTracker<I>) -> Maybe<Self::Item> {
    while self.new_items()? {
      let v = self.inner.generate(&mut src.child());
      self.chunk(v);
    };
    Ok(...);
  }
}
```

This organisation has the nice property that 

For an example with:

```rust
enum Foo {
  Bar,
  Quux(usize)
}
```

A `Vec<Foo>` would have the following layout:

```
 * Vec<Foo> -- Derived from a sequence of Option<Foo>
   * next/weighted coin:true
   * Foo
     * is_quux:true
     * Quux(u64)
       * 42usize
   * next/weighted coin:true
   * Foo
     * is_quux:false
     * Bar
   * next/weighted coin: false
```

Which ends up being represented as: 
```
Extent::Branch(/*Vec<Foo>*/ vec![
  Leaf(/*weighted coin*/ 1),
  Branch(/*Foo*/ vec![
    Leaf(/* is quux */, 1), Leaf(/* 42usize */ 8)]), 
  Leaf(/*weighted coin*/ 1),
  Branch(/*Foo*/ vec![
    Leaf(/* is quux */, 1), Leaf(/* 42usize */ 0)]), 
  Leaf(/*weighted coin*/ 1)])
```

it'd be possible to encode a type id to encompass such notions as "replace expression with subexpression of same type" and similar. Questions:

We want to be able to replay each level at a time. So If we shrink from a vector of 2 items to one, then we still want to generate the same information for items following the Vec, rather than generating something completely different. Otherwise, this would make something of a mockery of our "shrinking".

Put another way, when you stop consuming the sequence of (weighted_coin >> Foo), any Widget generators afterwards should get the same input. If we somehow end up consuming _more_ from the current branch, that branch should just return zeros once exhausted.

Turn the minimizers into iterators?