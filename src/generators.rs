// Generators
use std::marker::PhantomData;
use std::mem::size_of;

use data::*;

pub struct BoolGenerator;
pub struct IntGenerator<N>(PhantomData<N>);
pub struct VecGenerator<G>(G);

pub struct Filtered<G, F>(G, F);
pub struct Const<V>(V);

pub trait Generator {
    type Item;
    fn generate(&self, source: &mut InfoTap) -> Maybe<Self::Item>;

    fn filter<F: Fn(&Self::Item) -> bool>(self, pred: F) -> Filtered<Self, F>
    where
        Self: Sized,
    {
        Filtered(self, pred)
    }
}

pub fn booleans() -> BoolGenerator {
    BoolGenerator
}
pub fn u8s() -> IntGenerator<u8> {
    IntGenerator(PhantomData)
}
pub fn vecs<G>(inner: G) -> VecGenerator<G> {
    VecGenerator(inner)
}

pub fn consts<V>(val: V) -> Const<V> {
    Const(val)
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

impl Generator for BoolGenerator {
    type Item = bool;
    fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
        src.next_byte().map(|next| next >= 0x80)
    }
}

impl Generator for IntGenerator<u8> {
    type Item = u8;
    fn generate(&self, src: &mut InfoTap) -> Maybe<Self::Item> {
        assert!(size_of::<u8>() == 1);
        let nbytes = size_of::<u8>() / size_of::<u8>();
        let mut val: u8 = 0;
        for _ in 0..nbytes {
            val = val.wrapping_shl(8) | src.next_byte()?;
        }
        // src.next_byte().map(|next| next >= 0x80)
        Ok(val)
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

impl<V: Clone> Generator for Const<V> {
    type Item = V;
    fn generate(&self, _: &mut InfoTap) -> Maybe<Self::Item> {
        Ok(self.0.clone())
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

}
