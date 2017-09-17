// Generators
use std::marker::PhantomData;
use std::mem::size_of;

use data::*;

pub struct BoolGenerator;
pub struct IntGenerator<N>(PhantomData<N>);
pub struct FloatGenerator<N>(PhantomData<N>);
pub struct UniformFloatGenerator<N>(PhantomData<N>);
pub struct VecGenerator<G>(G);
pub struct InfoPoolGenerator(usize);
pub struct WeightedCoinGenerator(f32);

pub struct OneOfGenerator<T>(Vec<Box<Generator<Item=T>>>);

pub struct Filtered<G, F>(G, F);
pub struct FilterMapped<G, F>(G, F);
pub struct Mapped<G, F>(G, F);
pub struct Const<V>(V);

pub trait Generator {
    type Item;
    fn generate(&self, source: &mut InfoTap) -> Maybe<Self::Item>;
    fn generate_from(&self, source: &InfoPool) -> Maybe<Self::Item> {
        self.generate(&mut source.tap())
    }

    fn filter<F: Fn(&Self::Item) -> bool>(self, pred: F) -> Filtered<Self, F>
    where
        Self: Sized,
    {
        Filtered(self, pred)
    }

    fn filter_map<R, F: Fn(Self::Item) -> Maybe<R>>(self, pred: F) -> FilterMapped<Self, F>
    where
        Self: Sized,
    {
        FilterMapped(self, pred)
    }

    fn map<R, F: Fn(Self::Item) -> R>(self, fun: F) -> Mapped<Self, F>
    where
        Self: Sized,
    {
        Mapped(self, fun)
    }
}

pub fn booleans() -> BoolGenerator {
    BoolGenerator
}


pub fn vecs<G>(inner: G) -> VecGenerator<G> {
    VecGenerator(inner)
}

pub fn consts<V>(val: V) -> Const<V> {
    Const(val)
}

pub fn info_pools(len: usize) -> InfoPoolGenerator {
    InfoPoolGenerator(len)
}

pub fn weighted_coin(p: f32) -> WeightedCoinGenerator {
    WeightedCoinGenerator(p)
}

impl<G: Generator> Generator for VecGenerator<G> {
    type Item = Vec<G::Item>;
    fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
        let mut result = Vec::new();
        let bs = booleans();
        while bs.generate(src)? {
            let item = self.0.generate(src)?;
            result.push(item)
        }

        Ok(result)
    }
}

impl Generator for InfoPoolGenerator {
    type Item = InfoPool;
    fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
        let mut result = Vec::new();
        let vals = u8s();
        for _ in 0..self.0 {
            let item = vals.generate(src)?;
            result.push(item)
        }

        Ok(InfoPool::of_vec(result))
    }
}

impl Generator for BoolGenerator {
    type Item = bool;
    fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
        src.next_byte().map(|next| next >= 0x80)
    }
}

macro_rules! unsigned_integer_gen {
    ($name:ident, $ty:ty) => {
        pub fn $name() -> IntGenerator<$ty> {
            IntGenerator(PhantomData)
        }

        impl Generator for IntGenerator<$ty> {
            type Item = $ty;
            fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
                assert!(size_of::<u8>() == 1);
                let nbytes = size_of::<$ty>() / size_of::<u8>();
                let mut val: $ty = 0;
                for _ in 0..nbytes {
                    val = val.wrapping_shl(8) | src.next_byte()? as $ty;
                }
                // src.next_byte().map(|next| next >= 0x80)
                Ok(val)
            }
        }
    }
}

unsigned_integer_gen!(u8s, u8);
unsigned_integer_gen!(u16s, u16);
unsigned_integer_gen!(u32s, u32);
unsigned_integer_gen!(u64s, u64);
unsigned_integer_gen!(usizes, usize);

// We use the equivalent unsigned generator as an intermediate
macro_rules! signed_integer_gen {
    ($name:ident, $ugen:expr, $ty:ty) => {
        pub fn $name() -> IntGenerator<$ty> {
            IntGenerator(PhantomData)
        }

        impl Generator for IntGenerator<$ty> {
            type Item = $ty;
            fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
                let inner_g = $ugen;
                let uval = inner_g.generate(src)?;
                let is_neg = (uval & 1) == 0;
                let uval = uval >> 1;
                if is_neg {
                    Ok(-(uval as $ty))
                } else {
                    Ok(uval as $ty)
                }
            }
        }
    }
}

signed_integer_gen!(i8s, u8s(), i8);
signed_integer_gen!(i16s, u16s(), i16);
signed_integer_gen!(i32s, u32s(), i32);
signed_integer_gen!(i64s, u64s(), i64);
signed_integer_gen!(isizes, usizes(), isize);


// As with signed types, use the equivalent unsigned generator as an intermediate
macro_rules! float_gen {
    ($name:ident, $ugen:expr, $ty:ident) => {
        pub fn $name() -> FloatGenerator<$ty> {
            FloatGenerator(PhantomData)
        }

        impl Generator for FloatGenerator<$ty> {
            type Item = $ty;
            fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
                let inner_g = $ugen;
                let uval = inner_g.generate(src)?;

                let is_neg = (uval & 1) == 0;
                let fval = $ty::from_bits(uval >> 1);
                if is_neg {
                    Ok(-(fval as $ty))
                } else {
                    Ok(fval as $ty)
                }
            }
        }
    }
}

float_gen!(f32s, u32s(), f32);
float_gen!(f64s, u64s(), f64);

// As with signed types, use the equivalent unsigned generator as an intermediate
macro_rules! uniform_float_gen {
    ($name:ident, $ugen:expr, $inty:ident, $ty:ident) => {
        pub fn $name() -> UniformFloatGenerator<$ty> {
            UniformFloatGenerator(PhantomData)
        }

        impl Generator for UniformFloatGenerator<$ty> {
            type Item = $ty;
            fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
                let inner_g = $ugen;
                let uval = inner_g.generate(src)?;
                return Ok(uval as $ty / $inty::max_value() as $ty);
            }
        }
    }
}

uniform_float_gen!(uniform_f32s, u32s(), u32, f32);
uniform_float_gen!(uniform_f64s, u64s(), u64, f64);

impl Generator for WeightedCoinGenerator {
    type Item = bool;
    fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
        let &WeightedCoinGenerator(p) = self;
        let v = uniform_f32s().generate(src)?;
        let res = v > (1.0-p);
        Ok(res)
    }
}

impl<G: Generator, F: Fn(&G::Item) -> bool> Generator for Filtered<G, F> {
    type Item = G::Item;
    fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
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
    fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
        let &FilterMapped(ref gen, ref f) = self;
        let val = gen.generate(src)?;
        let out = f(val)?;
        Ok(out)
    }
}

impl<G: Generator, R, F: Fn(G::Item) -> R> Generator for Mapped<G, F> {
    type Item = R;
    fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
        let &Mapped(ref gen, ref f) = self;
        let val = gen.generate(src)?;
        let out = f(val);
        Ok(out)
    }
}

impl<V: Clone> Generator for Const<V> {
    type Item = V;
    fn generate(&self, _: &mut InfoTap) -> Maybe<Self::Item> {
        Ok(self.0.clone())
    }
}

impl<G: Generator, H: Generator> Generator for (G, H) {
    type Item = (G::Item, H::Item);
    fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
        let v0 = self.0.generate(src)?;
        let v1 = self.1.generate(src)?;
        Ok((v0, v1))
    }
}

pub fn one_of<G:Generator + 'static>(inner: G) -> OneOfGenerator<G::Item> {
    let inners = vec![Box::new(inner) as Box<Generator<Item=G::Item>>];
    OneOfGenerator(inners)
}

impl<T> OneOfGenerator<T> {
    pub fn or<G: Generator<Item=T> + 'static>(mut self, other: G) -> Self {
        self.0.push(Box::new(other));
        self
    }
}

impl<T>  Generator for  OneOfGenerator<T> {
    type Item = T;
    fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
        let v = u32s().generate(src)?;
        let it = (v as usize * self.0.len()) >> 32;
        self.0[it].generate(src)
    }
}

pub fn find_minimal<G: Generator, F: Fn(G::Item) -> bool>(
    gen: &G,
    pool: InfoPool,
    check: F,
) -> InfoPool {
    minimize(&pool, &|mut t| {
        gen.generate(&mut t).map(|v| check(v)).unwrap_or(false)
    }).unwrap_or(pool)
}

#[cfg(test)]
mod tests {
    extern crate env_logger;
    use rand::random;
    use std::iter;
    use std::fmt;
    use super::*;
    use data::InfoPool;
    const SHORT_VEC_SIZE: usize = 256;

    fn gen_random_vec() -> Vec<u8> {
        (0..SHORT_VEC_SIZE).map(|_| random()).collect::<Vec<u8>>()
    }

    fn should_generate_same_output_given_same_input<G: Generator>(gen: G)
    where
        G::Item: fmt::Debug + PartialEq,
    {
        for (p0, p1, v0, v1) in iter::repeat(())
            .map(|_| gen_random_vec())
            .map(|v0| (InfoPool::of_vec(v0.clone()), InfoPool::of_vec(v0)))
            .flat_map(|(p0, p1)| {
                gen.generate(&mut p0.tap()).and_then(|v0| {
                    gen.generate(&mut p1.tap()).map(|v1| (p0, p1, v0, v1))
                })
            })
            .take(100)
        {
            assert!(v0 == v1, "({:?} == {:?}) -> ({:?} == {:?})", p0, p1, v0, v1);
        }
    }

    fn usually_generates_different_output_for_different_inputs<G: Generator>(gen: G)
    where
        G::Item: PartialEq,
    {
        let nitems = 100;
        let differing = iter::repeat(())
            .map(|_| (gen_random_vec(), gen_random_vec()))
            .filter(|&(ref v0, ref v1)| v0 != v1)
            .map(|(v0, v1)| (InfoPool::of_vec(v0), InfoPool::of_vec(v1)))
            .flat_map(|(p0, p1)| {
                gen.generate(&mut p0.tap()).and_then(|v0| {
                    gen.generate(&mut p1.tap()).map(|v1| (v0, v1))
                })
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
    fn should_partially_order_same_as_source<G: Generator>(gen: G)
    where
        G::Item: PartialOrd + fmt::Debug,
    {
        let nitems = 100;
        for (p0, p1, v0, v1) in iter::repeat(())
            .map(|_| (gen_random_vec(), gen_random_vec()))
            .filter(|&(ref v0, ref v1)| v0 < v1)
            .map(|(v0, v1)| (InfoPool::of_vec(v0), InfoPool::of_vec(v1)))
            .flat_map(|(p0, p1)| {
                gen.generate(&mut p0.tap()).and_then(|v0| {
                    gen.generate(&mut p1.tap()).map(|v1| (p0, p1, v0, v1))
                })
            })
            .take(nitems)
        {
            assert!(v0 <= v1, "({:?} < {:?}) -> ({:?} <= {:?})", p0, p1, v0, v1);
        }
    }

    fn should_partially_order_same_as_source_by<G: Generator, K: PartialOrd, F: Fn(&G::Item) -> K>(
        gen: G,
        key: F,
    ) where
        G::Item: fmt::Debug,
    {
        let nitems = 100;
        for (p0, p1, v0, v1) in iter::repeat(())
            .map(|_| (gen_random_vec(), gen_random_vec()))
            .filter(|&(ref v0, ref v1)| v0 < v1)
            .map(|(v0, v1)| (InfoPool::of_vec(v0), InfoPool::of_vec(v1)))
            .flat_map(|(p0, p1)| {
                gen.generate(&mut p0.tap()).and_then(|v0| {
                    gen.generate(&mut p1.tap()).map(|v1| (p0, p1, v0, v1))
                })
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
            gen.generate(&mut InfoPool::of_vec(v1).tap()),
            Ok("fourty two")
        );
    }

    // If only I had some kind of property testing library.
    #[test]
    fn bools_should_generate_false_booleans_from_zeros() {
        let v1 = vec![0];
        let bools = booleans();
        assert_eq!(bools.generate(&mut InfoPool::of_vec(v1).tap()), Ok(false));
    }

    #[test]
    fn bools_should_generate_true_booleans_from_saturated_values() {
        let v1 = vec![0xff];
        let bools = booleans();
        assert_eq!(bools.generate(&mut InfoPool::of_vec(v1).tap()), Ok(true));
    }

    fn should_minimize_to<G: Generator>(gen: G, expected: G::Item)
    where
        G::Item: fmt::Debug + PartialEq,
    {
        let mut p;
        loop {
            p = InfoPool::random_of_size(SHORT_VEC_SIZE);
            match gen.generate(&mut p.tap()) {
                Ok(_) => break,
                Err(DataError::SkipItem) => (),
                Err(DataError::PoolExhausted) => panic!("Not enough pool to generate data"),
            }
        }
        debug!("Before: {:?}", p);
        let p = find_minimal(&gen, p, |_| true);
        debug!("After: {:?}", p);

        let val = gen.generate(&mut p.tap()).expect("generated value");
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
    fn vecs_should_generate_same_output_given_same_input() {
        should_generate_same_output_given_same_input(vecs(booleans()));
    }

    #[test]
    fn vecs_usually_generates_different_output_for_different_inputs() {
        usually_generates_different_output_for_different_inputs(vecs(booleans()))
    }

    #[test]
    fn vec_bools_minimize_to_empty() {
        env_logger::init().unwrap_or(());
        should_minimize_to(vecs(booleans()), vec![])
    }

    #[test]
    fn vec_bools_can_minimise_with_predicate() {
        env_logger::init().unwrap_or(());
        should_minimize_to(
            vecs(booleans()).filter(|v| v.len() > 2),
            vec![false, false, false],
        );
    }

    #[test]
    fn info_pools_should_generate_same_output_given_same_input() {
        should_generate_same_output_given_same_input(info_pools(8))
    }

    #[test]
    fn info_pools_usually_generates_different_output_for_different_inputs() {
        usually_generates_different_output_for_different_inputs(info_pools(8))
    }

    #[test]
    fn info_pools_minimize_to_empty() {
        env_logger::init().unwrap_or(());
        // We force the generator to output a fixed length.
        // This is perhaps not the best idea ever; but it'll do for now.
        should_minimize_to(info_pools(8), InfoPool::of_vec(vec![0; 8]))
    }

    #[test]
    fn u8s_should_generate_same_output_given_same_input() {
        should_generate_same_output_given_same_input(u8s())
    }

    // These really need to be proper statistical tests.
    #[test]
    fn u8s_usually_generates_different_output_for_different_inputs() {
        usually_generates_different_output_for_different_inputs(u8s());
    }

    #[test]
    fn u8s_minimize_to_zero() {
        should_minimize_to(u8s(), 0);
    }

    #[test]
    fn u8s_should_partially_order_same_as_source() {
        should_partially_order_same_as_source(u8s());
    }

    #[test]
    fn u64s_should_generate_same_output_given_same_input() {
        should_generate_same_output_given_same_input(u64s())
    }

    // These really need to be proper statistical tests.
    #[test]
    fn u64s_usually_generates_different_output_for_different_inputs() {
        usually_generates_different_output_for_different_inputs(u64s());
    }

    #[test]
    fn u64s_minimize_to_zero() {
        should_minimize_to(u64s(), 0);
    }

    #[test]
    fn u64s_should_partially_order_same_as_source() {
        should_partially_order_same_as_source(u64s());
    }

    #[test]
    fn tuple_u8s_u8s_should_generate_same_output_given_same_input() {
        should_generate_same_output_given_same_input((u8s(), u8s()))
    }


    #[test]
    fn tuple_u8s_u8s_usually_generates_different_output_for_different_inputs() {
        usually_generates_different_output_for_different_inputs((u8s(), u8s()));
    }

    #[test]
    fn tuple_u8s_u8s_minimize_to_zero() {
        should_minimize_to((u8s(), u8s()), (0, 0));
    }

    #[test]
    fn tuple_u8s_u8s_should_partially_order_same_as_source() {
        should_partially_order_same_as_source((u8s(), u8s()));
    }

    #[test]
    fn i64s_should_generate_same_output_given_same_input() {
        should_generate_same_output_given_same_input(i64s())
    }

    #[test]
    fn i64s_usually_generates_different_output_for_different_inputs() {
        usually_generates_different_output_for_different_inputs(i64s());
    }

    #[test]
    fn i64s_minimize_to_zero() {
        should_minimize_to(i64s(), 0);
    }

    #[test]
    fn i64s_should_partially_order_same_as_source() {
        should_partially_order_same_as_source_by(i64s(), |&v| v.abs());
    }


    #[test]
    fn filter_should_pass_through_when_true() {
        let gen = consts(()).filter(|&_| true);
        let p = InfoPool::random_of_size(4);
        assert_eq!(gen.generate(&mut p.tap()), Ok(()));
    }

    #[test]
    fn filter_should_skip_when_false() {
        let gen = consts(()).filter(|&_| false);
        let p = InfoPool::random_of_size(4);
        assert_eq!(gen.generate(&mut p.tap()), Err(DataError::SkipItem));
    }

    #[test]
    fn biased_coin() {
        let mut rng = ::rand::XorShiftRng::new_unseeded();
        let p = InfoPool::from_random_of_size(&mut rng, 1024);
        let gen = weighted_coin(1.0 / 3.0);
        let trials = 256;
        let expected = trials / 3;
        let allowed_error = trials / 32;
        let mut heads = 0;
        let mut t = p.tap();
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
        let _: &Generator<Item = usize> = &gen;
        let p = InfoPool::random_of_size(4);
        assert_eq!(gen.generate_from(&p), Ok(42));
    }

    #[test]
    fn filter_map_should_skip_when_err() {
        let gen = consts(()).filter_map(|()| Err(DataError::SkipItem));
        let _: &Generator<Item = usize> = &gen;
        let p = InfoPool::random_of_size(4);
        assert_eq!(gen.generate_from(&p), Err(DataError::SkipItem));
    }
}
