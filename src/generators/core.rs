use data::*;

use super::numbers::{u32s, uniform_f32s};

/// A convenience alias for generators that use the pool.
pub type Maybe<T> = Result<T, DataError>;

/// See [`booleans`](fn.booleans.html).
#[derive(Debug, Clone)]
pub struct BoolGenerator;
/// See [`weighted_coin`](fn.weighted_coin.html)
#[derive(Debug, Clone)]
pub struct WeightedCoinGenerator(f32);
/// See [`optional`](fn.optional.html)
#[derive(Debug, Clone)]
pub struct OptionalGenerator<B, G>(B, G);
/// See [`result`](fn.result.html)
#[derive(Debug, Clone)]
pub struct ResultGenerator<G, H>(G, H);

/// See [`one_of`](fn.one_of.html)
#[derive(Debug, Clone)]
pub struct OneOfGenerator<GS>(GS);
/// See [`lazy`](fn.lazy.html)
#[derive(Debug, Clone)]
pub struct LazyGenerator<F>(F);

/// Internal implementation for [`one_of`](fn.one_of.html). Defines the
/// operations supported by an choice in a `one_of`.
pub trait OneOfItem {
    /// The generator type.
    type Item;
    /// The number of cases reachable from this one.
    fn len(&self) -> usize;
    /// Depending on the case selected in the
    /// [`Generator`](trait.Generator.html) ipmlementation for
    /// (`OneOfGenerator`)[struct.OneOfGenerator.html], we either call our
    /// generator directly, or delegate to the next in the chain.
    fn generate_or_delegate<I: InfoSource>(&self, depth: usize, tap: &mut I) -> Maybe<Self::Item>;
}

/// Internal implementation for [`one_of`](fn.one_of.html). Forms the
/// terminating case of the induction.
#[derive(Debug, Clone)]
pub struct OneOfTerm<G> {
    gen: G,
}
/// Internal implementation for [`one_of`](fn.one_of.html). Forms a
/// left-associated chain of generators.
#[derive(Debug, Clone)]
pub struct OneOfSnoc<G, R> {
    rest: R,
    gen: G,
}

/// See [`Generator::filter`](trait.Generator.html#method.filter)
#[derive(Debug, Clone)]
pub struct Filtered<G, F>(G, F);
/// See [`Generator::filter_map`](trait.Generator.html#method.filter_map)
#[derive(Debug, Clone)]
pub struct FilterMapped<G, F>(G, F);
/// See [`Generator::map`](trait.Generator.html#method.map)
#[derive(Debug, Clone)]
pub struct Mapped<G, F>(G, F);
/// See [`Generator::flat_map`](trait.Generator.html#method.flat_map)
#[derive(Debug, Clone)]
pub struct FlatMapped<G, F>(G, F);
/// See [`consts`](fn.consts.html)
#[derive(Debug, Clone)]
pub struct Const<V>(V);

/// An object that can generate test data from an underlying data source.
///
/// In order for shrinking to work correctly; we need to ensure that the
/// output values get smaller as the input bytes get smaller (usually by input value).

pub trait Generator {
    /// The type of values that we can generate.
    type Item;
    /// This consumes a stream of bytes given by `source`, and generates a
    /// value of type `Self::Item`.
    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item>;
    /// A convenience function to generate a value directly from an `InfoPool`.
    fn generate_from(&self, src: &InfoPool) -> Maybe<Self::Item> {
        self.generate(&mut src.replay())
    }

    /// Returns a generator that will skip values generated by `self` when
    /// the predicate `pred` returns `false`.
    fn filter<F: Fn(&Self::Item) -> bool>(self, pred: F) -> Filtered<Self, F>
    where
        Self: Sized,
    {
        Filtered(self, pred)
    }

    /// A generator that takes the values generated by `self`; then allows
    /// `fun` to either Skip them (by returning `Err(DataError::SkipItem)` or
    /// transform (by returning `Ok(val)`).

    fn filter_map<R, F: Fn(Self::Item) -> Maybe<R>>(self, fun: F) -> FilterMapped<Self, F>
    where
        Self: Sized,
    {
        FilterMapped(self, fun)
    }

    /// A generator that takes the generated values of `self` and pipes them
    /// through `fun`.
    fn map<R, F: Fn(Self::Item) -> R>(self, fun: F) -> Mapped<Self, F>
    where
        Self: Sized,
    {
        Mapped(self, fun)
    }

    /// A generator that allows creating a new generator based on the results
    /// of a previous generator.
    fn flat_map<H: Generator, F: Fn(Self::Item) -> H>(self, fun: F) -> FlatMapped<Self, F>
    where
        Self: Sized,
    {
        FlatMapped(self, fun)
    }
}

impl<'a, G: Generator> InfoSink for &'a G {
    type Out = Maybe<G::Item>;
    fn sink<I: InfoSource>(&mut self, src: &mut I) -> Self::Out {
        self.generate(src)
    }
}

/// Like [`Generator`](trait.Generator.html), but allows use as a trait object.
pub trait GeneratorObject {
    /// The type of values that we can generate.
    type Item;
    /// This consumes a stream of bytes given by `source`, and generates a
    /// value of type `Self::Item`.
    fn generate_obj(&self, src: &mut InfoSource) -> Maybe<Self::Item>;
}

/// An extension trait that allows use of methods that assume Self has a known
/// size, like `#boxed`.
pub trait GeneratorSized {
    /// See [`Generator::Item`](trait.Generator.html#associatedtype.Item)
    type Item;
    /// Returns a boxed trait object. Useful for returning a series of chained
    /// combinators without having to declare the full type.
    fn boxed(self) -> Box<GeneratorObject<Item = Self::Item>>;
}

impl<G> GeneratorSized for G
where
    G: Generator + 'static,
{
    type Item = G::Item;
    fn boxed(self) -> Box<GeneratorObject<Item = Self::Item>> {
        Box::new(self)
    }
}

impl<G: Generator> GeneratorObject for G {
    type Item = G::Item;
    fn generate_obj(&self, mut src: &mut InfoSource) -> Maybe<Self::Item> {
        (*self).generate(&mut src)
    }
}

impl<T> Generator for Box<GeneratorObject<Item = T>> {
    type Item = T;
    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
        (**self).generate_obj(src as &mut InfoSource)
    }
}

/// Generates boolean value with a 50% chance of being true.
pub fn booleans() -> BoolGenerator {
    BoolGenerator
}

/// Always generates a clone of the given value.
pub fn consts<V: Clone>(val: V) -> Const<V> {
    Const(val)
}

/// Generates a boolean with the specified probability (0.0 <= p <= 1.0) of being true.
pub fn weighted_coin(p: f32) -> WeightedCoinGenerator {
    WeightedCoinGenerator(p)
}

/// Generates an Optional<_> value with a 50% chance of `Some(_)` from the
/// `inner` generator, otherwise None.
pub fn optional<G>(inner: G) -> OptionalGenerator<BoolGenerator, G> {
    OptionalGenerator(booleans(), inner)
}

/// Generates an Optional<_> value using `bools` to decide whether to choose
/// `Some(_)` from the `inner` generator, otherwise None.
pub fn optional_by<B, G>(bools: B, inner: G) -> OptionalGenerator<B, G> {
    OptionalGenerator(bools, inner)
}

/// Generates either an okay value from `ok`; or an error from `err`, with 50% chance of each.
pub fn result<G: Generator, H: Generator>(ok: G, err: H) -> ResultGenerator<G, H> {
    ResultGenerator(ok, err)
}

/// Returns a lazily evaluated generator. The `thunk` should be pure.
/// Mostly used to allow recursive generators.
pub fn lazy<F: Fn() -> G, G: Generator>(thunk: F) -> LazyGenerator<F> {
    LazyGenerator(thunk)
}

impl<'a, G: Generator> Generator for &'a G {
    type Item = G::Item;
    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
        (**self).generate(src)
    }
}

impl Generator for BoolGenerator {
    type Item = bool;
    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
        debug!("-> BoolGenerator::generate");
        let res = src.draw_u8() >= 0x80;
        debug!("<- BoolGenerator::generate");
        Ok(res)
    }
}

impl<B: Generator<Item = bool>, G: Generator> Generator for OptionalGenerator<B, G> {
    type Item = Option<G::Item>;
    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
        debug!("-> OptionalGenerator::generate");
        let &OptionalGenerator(ref bools, ref gen) = self;
        let result = if src.draw(bools)? {
            Some(src.draw(gen)?)
        } else {
            None
        };

        debug!("<- OptionalGenerator::generate");
        Ok(result)
    }
}

impl<G: Generator, H: Generator> Generator for ResultGenerator<G, H> {
    type Item = Result<G::Item, H::Item>;
    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
        let &ResultGenerator(ref ok, ref err) = self;
        let bs = booleans();
        let result = if bs.generate(src)? {
            Err(err.generate(src)?)
        } else {
            Ok(ok.generate(src)?)
        };

        Ok(result)
    }
}

impl<G: Generator, F: Fn(&G::Item) -> bool> Generator for Filtered<G, F> {
    type Item = G::Item;
    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
        let &Filtered(ref gen, ref pred) = self;
        let val = gen.generate(src)?;
        if pred(&val) {
            Ok(val)
        } else {
            Err(DataError::SkipItem)
        }
    }
}

impl<G: Generator, R, F: Fn(G::Item) -> Maybe<R>> Generator for FilterMapped<G, F> {
    type Item = R;
    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
        let &FilterMapped(ref gen, ref f) = self;
        let val = gen.generate(src)?;
        let out = f(val)?;
        Ok(out)
    }
}

impl<G: Generator, R, F: Fn(G::Item) -> R> Generator for Mapped<G, F> {
    type Item = R;
    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
        let &Mapped(ref gen, ref f) = self;
        let val = gen.generate(src)?;
        let out = f(val);
        Ok(out)
    }
}

impl<G: Generator, H: Generator, F: Fn(G::Item) -> H> Generator for FlatMapped<G, F> {
    type Item = H::Item;
    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
        let &FlatMapped(ref gen, ref f) = self;
        let gen2 = gen.generate(src)?;
        let out = f(gen2).generate(src)?;
        Ok(out)
    }
}

impl<V: Clone> Generator for Const<V> {
    type Item = V;
    fn generate<I: InfoSource>(&self, _: &mut I) -> Maybe<Self::Item> {
        Ok(self.0.clone())
    }
}

impl Generator for WeightedCoinGenerator {
    type Item = bool;
    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
        debug!("-> WeightedCoinGenerator::generate");
        let &WeightedCoinGenerator(p) = self;
        let v = uniform_f32s().generate(src)?;
        let res = v > (1.0 - p);
        debug!("<- WeightedCoinGenerator::generate");
        Ok(res)
    }
}

/// Allows the user to use one of a set of alternative generators.
/// Often useful when you need to generate elements of an enum.
///
/// ```
/// use suppositions::generators::*;
/// fn option_of_u8() {
///     let g = one_of(consts(None)).or(u8s().map(Some));
///     //
/// }
/// ```

pub fn one_of<G: Generator + 'static>(inner: G) -> OneOfGenerator<OneOfTerm<G>> {
    OneOfGenerator(OneOfTerm { gen: inner })
}

impl<GS: OneOfItem> OneOfGenerator<GS> {
    /// Specifies an alternative data generator. See [generators::one_of](fn.one_of.html) for details.
    pub fn or<G: Generator<Item = GS::Item> + 'static>(
        self,
        other: G,
    ) -> OneOfGenerator<OneOfSnoc<G, GS>> {
        let OneOfGenerator(gs) = self;
        let rs = OneOfSnoc {
            gen: other,
            rest: gs,
        };
        OneOfGenerator(rs)
    }
}

impl<F: Fn() -> G, G: Generator> Generator for LazyGenerator<F> {
    type Item = G::Item;
    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
        let g = self.0();
        g.generate(src)
    }
}

impl<G: Generator, R: OneOfItem<Item = G::Item>> OneOfItem for OneOfSnoc<G, R> {
    type Item = G::Item;
    fn len(&self) -> usize {
        self.rest.len() + 1
    }
    fn generate_or_delegate<I: InfoSource>(&self, depth: usize, tap: &mut I) -> Maybe<Self::Item> {
        if depth == 0 {
            self.gen.generate(tap)
        } else {
            self.rest.generate_or_delegate(depth - 1, tap)
        }
    }
}

impl<G: Generator> OneOfItem for OneOfTerm<G> {
    type Item = G::Item;
    fn len(&self) -> usize {
        1
    }
    fn generate_or_delegate<I: InfoSource>(&self, depth: usize, tap: &mut I) -> Maybe<Self::Item> {
        debug_assert_eq!(depth, 0);
        self.gen.generate(tap)
    }
}

impl<GS: OneOfItem> Generator for OneOfGenerator<GS> {
    type Item = GS::Item;
    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
        let v = !u32s().generate(src)?;
        let it = (v as usize * self.0.len()) >> 32;
        self.0.generate_or_delegate(it, src)
    }
}

/// Find the smalltest `InfoPool` such that the function `check` succeeds
/// when applied to the generated arguments Mostly a convenience wrapper
/// around the shrinking functions in the `data` module.
pub fn find_minimal<G: Generator, F: Fn(G::Item) -> bool>(
    gen: &G,
    pool: InfoPool,
    check: F,
) -> InfoPool {
    minimize(&pool, &|mut t| {
        t.draw(&gen).map(|v| check(v)).unwrap_or(false)
    }).unwrap_or(pool)
}

#[cfg(test)]
pub mod tests {
    use rand::{random, Rng};
    use env_logger;
    use std::iter;
    use std::fmt;
    use std::collections::BTreeMap;
    use super::*;
    use data::InfoPool;
    use generators::numbers::*;

    const SHORT_VEC_SIZE: usize = 256;

    fn gen_random_vec() -> Vec<u8> {
        (0..SHORT_VEC_SIZE).map(|_| random()).collect::<Vec<u8>>()
    }

    /// Create an `InfoPool` with a `size` length vector of random bytes
    /// using the generator `rng`. (Mostly used for testing).
    pub fn unseeded_of_size(size: usize) -> InfoPool {
        let mut rng = ::rand::XorShiftRng::new_unseeded();
        InfoPool::of_vec((0..size).map(|_| rng.gen::<u8>()).collect::<Vec<u8>>())
    }

    pub fn should_generate_same_output_given_same_input<G: Generator>(gen: G)
    where
        G::Item: fmt::Debug + PartialEq,
    {
        for (p0, p1, v0, v1) in iter::repeat(())
            .map(|_| gen_random_vec())
            .map(|v0| (InfoPool::of_vec(v0.clone()), InfoPool::of_vec(v0)))
            .flat_map(|(p0, p1)| {
                gen.generate(&mut p0.replay())
                    .and_then(|v0| gen.generate(&mut p1.replay()).map(|v1| (p0, p1, v0, v1)))
            })
            .take(100)
        {
            assert!(v0 == v1, "({:?} == {:?}) -> ({:?} == {:?})", p0, p1, v0, v1);
        }
    }

    pub fn usually_generates_different_output_for_different_inputs<G: Generator>(gen: G)
    where
        G::Item: PartialEq,
    {
        let nitems = 100;
        let differing = iter::repeat(())
            .map(|_| (gen_random_vec(), gen_random_vec()))
            .filter(|&(ref v0, ref v1)| v0 != v1)
            .map(|(v0, v1)| (InfoPool::of_vec(v0), InfoPool::of_vec(v1)))
            .flat_map(|(p0, p1)| {
                gen.generate(&mut p0.replay())
                    .and_then(|v0| gen.generate(&mut p1.replay()).map(|v1| (v0, v1)))
            })
            .take(nitems)
            .filter(|&(ref v0, ref v1)| v0 != v1)
            .count();
        assert!(differing > 0, "Differing items:{} > 0", differing);
    }

    // Mostly only useful for scalar quantities. For collections, we basically say:
    // `while booleans() { elemnt() }`
    // So because booleans are imprecisly generated (ie: a pool of [0xff] ~ [0x80]),
    // the source and result can have a differing ordering.
    pub fn should_partially_order_same_as_source<G: Generator>(gen: G)
    where
        G::Item: PartialOrd + fmt::Debug + Clone,
    {
        should_partially_order_same_as_source_by(gen, |v| v.clone())
    }

    pub fn should_partially_order_same_as_source_by<
        G: Generator,
        K: PartialOrd,
        F: Fn(&G::Item) -> K,
    >(
        gen: G,
        key: F,
    ) where
        G::Item: fmt::Debug + PartialEq,
    {
        let nitems = 100;
        for (p0, p1, v0, v1) in iter::repeat(())
            .map(|_| (gen_random_vec(), gen_random_vec()))
            .filter(|&(ref v0, ref v1)| v0 < v1)
            .map(|(v0, v1)| (InfoPool::of_vec(v0), InfoPool::of_vec(v1)))
            .flat_map(|(p0, p1)| {
                gen.generate(&mut p0.replay())
                    .and_then(|v0| gen.generate(&mut p1.replay()).map(|v1| (p0, p1, v0, v1)))
            })
            .take(nitems)
        {
            assert!(
                key(&v0) <= key(&v1),
                "({:?} < {:?}) -> ({:?} <= {:?})",
                p0,
                p1,
                v0,
                v1
            );
        }
    }

    #[test]
    fn consts_should_generate_same_values() {
        let v1 = gen_random_vec();
        let gen = consts("fourty two");
        assert_eq!(
            gen.generate(&mut InfoPool::of_vec(v1).replay()),
            Ok("fourty two")
        );
    }

    // If only I had some kind of property testing library.
    #[test]
    fn bools_should_generate_false_booleans_from_zeros() {
        let v1 = vec![0];
        let bools = booleans();
        assert_eq!(
            bools.generate(&mut InfoPool::of_vec(v1).replay()),
            Ok(false)
        );
    }

    #[test]
    fn bools_should_generate_true_booleans_from_saturated_values() {
        let v1 = vec![0xff];
        let bools = booleans();
        assert_eq!(bools.generate(&mut InfoPool::of_vec(v1).replay()), Ok(true));
    }

    pub fn should_minimize_to<G: Generator>(gen: G, expected: G::Item)
    where
        G::Item: fmt::Debug + PartialEq,
    {
        let mut p;
        loop {
            p = InfoRecorder::new(RngSource::new());
            match gen.generate(&mut p) {
                Ok(_) => break,
                Err(DataError::SkipItem) => (),
                Err(DataError::PoolExhausted) => panic!("Not enough pool to generate data"),
            }
        }
        let p = p.into_pool();
        debug!("Before: {:?}", p);
        let p = find_minimal(&gen, p, |_| true);
        debug!("After: {:?}", p);

        let val = gen.generate(&mut p.replay()).expect("generated value");
        assert_eq!(val, expected);
    }

    #[test]
    fn bools_should_generate_same_output_given_same_input() {
        should_generate_same_output_given_same_input(booleans())
    }

    // These really need to be proper statistical tests.
    #[test]
    fn bools_usually_generates_different_output_for_different_inputs() {
        usually_generates_different_output_for_different_inputs(booleans())
    }

    #[test]
    fn bools_minimize_to_false() {
        should_minimize_to(booleans(), false)
    }

    #[test]
    fn bools_should_partially_order_same_as_source() {
        should_partially_order_same_as_source(booleans())
    }

    #[test]
    fn optional_u64s_minimize_to_none() {
        should_minimize_to(optional(u64s()), None);
    }

    #[test]
    fn optional_by_coin_of_u64s_minimize_to_none() {
        let gen = optional_by(weighted_coin(3.0f32 / 4.0), u64s());
        should_minimize_to(gen, None);
    }

    #[test]
    fn result_u8_u64s_minimize_to_ok() {
        should_minimize_to(result(u8s(), u64s()), Ok(0));
    }

    #[test]
    fn filter_should_pass_through_when_true() {
        let gen = consts(()).filter(|&_| true);
        let p = InfoPool::new();
        assert_eq!(gen.generate(&mut p.replay()), Ok(()));
    }

    #[test]
    fn filter_should_skip_when_false() {
        let gen = consts(()).filter(|&_| false);
        let p = InfoPool::new();
        assert_eq!(gen.generate(&mut p.replay()), Err(DataError::SkipItem));
    }

    #[test]
    fn biased_coin() {
        let p = unseeded_of_size(1024);
        let gen = weighted_coin(1.0 / 3.0);
        let trials = 256;
        let expected = trials / 3;
        let allowed_error = trials / 32;
        let mut heads = 0;
        let mut t = p.replay();
        for _ in 0..trials {
            if gen.generate(&mut t).expect("a trial") {
                heads += 1;
            }
        }

        assert!(
            heads >= (expected - allowed_error) && heads <= (expected + allowed_error),
            "Expected 33% of {} trials ({}+/-{}); got {}",
            trials,
            expected,
            allowed_error,
            heads
        );
    }

    #[test]
    fn filter_map_should_pass_through_when_ok() {
        let gen = consts(()).filter_map(|()| Ok(42usize));
        let p = InfoPool::new();
        assert_eq!(gen.generate_from(&p), Ok(42));
    }

    #[test]
    fn filter_map_should_skip_when_err() {
        let gen = consts(()).filter_map(|()| -> Result<(), DataError> { Err(DataError::SkipItem) });
        let p = InfoPool::new();
        assert_eq!(gen.generate_from(&p), Err(DataError::SkipItem));
    }

    #[test]
    fn one_of_should_pick_choices_relativey_evenly() {
        env_logger::init().unwrap_or(());
        let gen = one_of(consts(1usize)).or(consts(2)).or(consts(3));
        let trials = 1024usize;
        let expected = trials / 3;
        let allowed_error = expected / 10;
        let mut samples = BTreeMap::new();
        let p = unseeded_of_size(1 << 18);
        let mut t = p.replay();
        for _ in 0..trials {
            let val = gen.generate(&mut t).expect("a trial");
            *samples.entry(val).or_insert(0) += 1;
        }

        println!("Histogram: {:?}", samples);
        assert!(
            samples
                .values()
                .all(|&val| val >= (expected - allowed_error) && val <= (expected + allowed_error)),
            "Sample counts from {} trials are all ({}+/-{}); got {:?}",
            trials,
            expected,
            allowed_error,
            samples,
        );
    }
}
