use std::iter;
use std::marker::PhantomData;
use data::*;
use generators::core::*;
use generators::numbers::*;

/// See [`vecs`](fn.vecs.html)
#[derive(Debug, Clone)]
pub struct VecGenerator<G> {
    inner: G,
    mean_length: usize,
}

/// See [`info_pools`](fn.info_pools.html)
#[derive(Debug, Clone)]
pub struct InfoPoolGenerator(usize);
/// See [`collections`](fn.collections.html)
#[derive(Debug, Clone)]
pub struct CollectionGenerator<C, G> {
    witness: PhantomData<C>,
    inner: G,
    mean_length: usize,
}

/// Generates vectors with items given by `inner`.
pub fn vecs<G>(inner: G) -> VecGenerator<G> {
    VecGenerator {
        inner: inner,
        mean_length: 10,
    }
}
/// Randomly generates an info-pool (mostly used for testing generators).
pub fn info_pools(len: usize) -> InfoPoolGenerator {
    InfoPoolGenerator(len)
}

/// Generates a collection of the given type, populated with elements from the
/// item generator.
///
/// To generate values of BTreeSet<u8>:
///
/// ```
/// use std::collections::BTreeSet;
/// use suppositions::generators::*;
/// let gen = collections::<BTreeSet<_>, _>(u8s());
/// ```
pub fn collections<C, G: Generator>(item: G) -> CollectionGenerator<C, G>
where
    C: Extend<G::Item>,
{
    CollectionGenerator {
        witness: PhantomData,
        inner: item,
        mean_length: 16,
    }
}

impl<G> VecGenerator<G> {
    /// Specify the mean length of the vector.
    pub fn mean_length(mut self, mean: usize) -> Self {
        self.mean_length = mean;
        self
    }
}

impl<G: Generator> Generator for VecGenerator<G> {
    type Item = Vec<G::Item>;
    fn generate<I: Iterator<Item = u8>>(&self, src: &mut I) -> Maybe<Self::Item> {
        let mut result = Vec::new();
        let p_is_final = 1.0 / (1.0 + self.mean_length as f32);
        let bs = weighted_coin(1.0 - p_is_final);
        while bs.generate(src)? {
            let item = self.inner.generate(src)?;
            result.push(item)
        }

        Ok(result)
    }
}

impl Generator for InfoPoolGenerator {
    type Item = InfoPool;
    fn generate<I: Iterator<Item = u8>>(&self, src: &mut I) -> Maybe<Self::Item> {
        let mut result = Vec::new();
        let vals = u8s();
        for _ in 0..self.0 {
            let item = vals.generate(src)?;
            result.push(item)
        }

        Ok(InfoPool::of_vec(result))
    }
}
impl<G, C> CollectionGenerator<C, G> {
    /// Specify the mean number of _generated_ items. For collections with
    /// set semantics, this many not be the same as the mean size of the
    /// collection.
    pub fn mean_length(mut self, mean: usize) -> Self {
        self.mean_length = mean;
        self
    }
}
impl<G: Generator, C: Default + Extend<G::Item>> Generator for CollectionGenerator<C, G> {
    type Item = C;
    fn generate<I: Iterator<Item = u8>>(&self, src: &mut I) -> Maybe<Self::Item> {
        let mut coll: C = Default::default();
        let p_is_final = 1.0 / (1.0 + self.mean_length as f32);
        let bs = weighted_coin(1.0 - p_is_final);
        while bs.generate(src)? {
            let item = self.inner.generate(src)?;
            coll.extend(iter::once(item));
        }

        Ok(coll)
    }
}

#[cfg(test)]
mod tests {
    use env_logger;
    use generators::core::*;
    use generators::collections::*;
    use generators::core::tests::*;

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
    fn collections_u64s_minimize_to_empty() {
        use std::collections::BTreeSet;
        should_minimize_to(collections::<BTreeSet<_>, _>(u8s()), BTreeSet::new());
    }

    mod vector_lengths {
        use env_logger;
        use generators::core::tests::*;
        use generators::collections::*;
        use std::collections::BTreeMap;

        #[test]
        fn mean_length_can_be_set_as_10() {
            mean_length_can_be_set_as(10);
        }

        #[test]
        fn mean_length_can_be_set_as_3() {
            mean_length_can_be_set_as(3);
        }

        #[test]
        fn mean_length_can_be_set_as_5() {
            mean_length_can_be_set_as(5);
        }

        #[test]
        fn mean_length_can_be_set_as_7() {
            mean_length_can_be_set_as(7);
        }

        #[test]
        fn mean_length_can_be_set_as_23() {
            mean_length_can_be_set_as(23);
        }
        fn mean_length_can_be_set_as(len: usize) {
            env_logger::init().unwrap_or(());
            let gen = vecs(u8s()).mean_length(len);
            let trials = 1024usize;
            let expected = len as f64;
            let allowed_error = expected * 0.1;
            let mut lengths = BTreeMap::new();
            let p = unseeded_of_size(1 << 18);
            let mut t = p.replay();
            for _ in 0..trials {
                let val = gen.generate(&mut t).expect("a trial");
                *lengths.entry(val.len()).or_insert(0) += 1;
            }

            println!("Histogram: {:?}", lengths);
            let mean: f64 = lengths
                .iter()
                .map(|(&l, &n)| (l * n) as f64 / trials as f64)
                .sum();
            assert!(
                mean >= (expected - allowed_error) && mean <= (expected + allowed_error),
                "Expected mean of {} trials ({}+/-{}); got {}",
                trials,
                expected,
                allowed_error,
                mean
            );
        }
    }
    mod collection_lengths {
        use env_logger;
        use generators::collections::*;
        use generators::core::tests::*;
        use std::collections::{BTreeMap, LinkedList};

        #[test]
        fn mean_length_can_be_set_as_10() {
            mean_length_can_be_set_as(10);
        }

        #[test]
        fn mean_length_can_be_set_as_3() {
            mean_length_can_be_set_as(3);
        }

        #[test]
        fn mean_length_can_be_set_as_5() {
            mean_length_can_be_set_as(5);
        }

        #[test]
        fn mean_length_can_be_set_as_7() {
            mean_length_can_be_set_as(7);
        }

        #[test]
        fn mean_length_can_be_set_as_23() {
            mean_length_can_be_set_as(23);
        }
        fn mean_length_can_be_set_as(len: usize) {
            env_logger::init().unwrap_or(());
            let gen = collections::<LinkedList<_>, _>(u8s()).mean_length(len);
            let trials = 1024usize;
            let expected = len as f64;
            let allowed_error = expected * 0.1;
            let mut lengths = BTreeMap::new();
            let p = unseeded_of_size(1 << 18);
            let mut t = p.replay();
            for _ in 0..trials {
                let val: LinkedList<u8> = gen.generate(&mut t).expect("a trial");
                *lengths.entry(val.len()).or_insert(0) += 1;
            }

            println!("Histogram: {:?}", lengths);
            let mean: f64 = lengths
                .iter()
                .map(|(&l, &n)| (l * n) as f64 / trials as f64)
                .sum();
            assert!(
                mean >= (expected - allowed_error) && mean <= (expected + allowed_error),
                "Expected mean of {} trials ({}+/-{}); got {}",
                trials,
                expected,
                allowed_error,
                mean
            );
        }
    }
}
