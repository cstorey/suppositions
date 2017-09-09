extern crate rand;

pub struct InfoPool;

pub trait Generator {
    type Item;
    // Result?
    fn generate(&self, source: &mut InfoPool) -> Self::Item;
}

pub struct Property<G> {
    gen: G,
}

pub fn property<G>(gen: G) -> Property<G> {
    Property { gen }
}

impl<G: Generator> Property<G> {
    pub fn check<F: Fn(G::Item) -> bool>(self, check: F) {
        let mut pool = InfoPool;

        for _i in 0..100 {
            let arg = self.gen.generate(&mut pool);
            let _res = check(arg);
            // Something something
            unimplemented!()
        }
    }
}

// Generators

pub struct IntGenerator;
pub struct VecGenerator<G>(G);

pub fn integers() -> IntGenerator {
    IntGenerator
}
pub fn vecs<G>(inner: G) -> VecGenerator<G> {
    VecGenerator(inner)
}

impl<G: Generator> Generator for VecGenerator<G> {
    type Item = Vec<G::Item>;
    fn generate(&self, _src: &mut InfoPool) -> Self::Item {
        unimplemented!()
    }
}
impl Generator for IntGenerator {
    type Item = u8;
    fn generate(&self, _src: &mut InfoPool) -> Self::Item {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {}
