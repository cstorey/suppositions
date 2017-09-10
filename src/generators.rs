// Generators

use data::*;

pub struct BoolGenerator;
pub struct VecGenerator<G>(G);

pub trait Generator {
    type Item;
    fn generate(&self, source: &mut InfoTap) -> Maybe<Self::Item>;
}

pub fn booleans() -> BoolGenerator {
    BoolGenerator
}
pub fn vecs<G>(inner: G) -> VecGenerator<G> {
    VecGenerator(inner)
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

impl BoolGenerator {}

#[cfg(test)]
mod tests {
    use rand::random;
    use std::iter;
    use super::*;
    use data::InfoPool;

    fn gen_random_vec(size: usize) -> Vec<u8> {
        (0..size).map(|_| random()).collect::<Vec<u8>>()
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

    #[test]
    fn bools_should_generate_same_output_given_same_input() {
        let gen = booleans();
        for (p0, p1, v0, v1) in iter::repeat(())
            .map(|_| gen_random_vec(16))
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

    // These really need to be proper statistical tests.
    #[test]
    fn bools_usually_generates_different_output_for_different_inputs() {
        let gen = booleans();
        let nitems = 100;
        let differing = iter::repeat(())
            .map(|_| (gen_random_vec(16), gen_random_vec(1024)))
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
    fn vecs_should_generate_same_output_given_same_input() {
        let gen = vecs(booleans());
        for (p0, p1, v0, v1) in iter::repeat(())
            .map(|_| gen_random_vec(16))
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

    #[test]
    fn vecs_usually_generates_different_output_for_different_inputs() {
        let gen = vecs(booleans());
        let nitems = 100;
        let differing = iter::repeat(())
            .map(|_| (gen_random_vec(16), gen_random_vec(1024)))
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
}
