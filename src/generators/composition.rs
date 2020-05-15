use super::core::*;
use data::*;

struct GeneratorFn<F>(F);

/// Makes it slightly easier to implement generators, by allowing the user
/// to specify a function, rather than needing to build it from either
/// combinators, or create a new Generator instance.
pub fn generator_fn<T>(f: impl Fn(&mut dyn InfoSource) -> Maybe<T>) -> impl Generator<Item = T> {
    GeneratorFn(f)
}

impl<T, F: Fn(&mut dyn InfoSource) -> Maybe<T>> Generator for GeneratorFn<F> {
    type Item = T;

    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
        (self.0)(src)
    }
}
