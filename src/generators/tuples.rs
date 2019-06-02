use data::*;
use generators::core::*;

macro_rules! tuple_generator_impl {
    ($gen_a:ident: $var_a:ident: $type_a:ident
        $(, $gen_n: ident: $var_n:ident: $type_n:ident)*) => (
        impl<$type_a: Generator, $($type_n: Generator),*> Generator
                for ($type_a, $($type_n),*) {
                    type Item = ($type_a::Item, $($type_n::Item),*);
                    fn generate<In: InfoSource>(&self, src: &mut In) -> Maybe<Self::Item> {
                        // Gens
                        let &(ref $gen_a, $(ref $gen_n),*) = self;
                        let $var_a = $gen_a.generate(src)?;
                        $(let $var_n = $gen_n.generate(src)?;)*
                        Ok(($var_a, $($var_n),*))
                    }
                }
    );
}

tuple_generator_impl!(ga: a: A);
tuple_generator_impl!(ga: a: A, gb: b: B);
tuple_generator_impl!(ga: a: A, gb: b: B, gc: c: C);
tuple_generator_impl!(ga: a: A, gb: b: B, gc: c: C, gd: d: D);

tuple_generator_impl!(ga: a: A, gb: b: B, gc: c: C, gd: d: D, ge: e: E);
tuple_generator_impl!(ga: a: A, gb: b: B, gc: c: C, gd: d: D, ge: e: E, gf: f: F);
tuple_generator_impl!(
    ga: a: A,
    gb: b: B,
    gc: c: C,
    gd: d: D,
    ge: e: E,
    gf: f: F,
    gg: g: G
);
tuple_generator_impl!(
    ga: a: A,
    gb: b: B,
    gc: c: C,
    gd: d: D,
    ge: e: E,
    gf: f: F,
    gg: g: G,
    gh: h: H
);

tuple_generator_impl!(
    ga: a: A,
    gb: b: B,
    gc: c: C,
    gd: d: D,
    ge: e: E,
    gf: f: F,
    gg: g: G,
    gh: h: H,
    gi: i: I
);
tuple_generator_impl!(
    ga: a: A,
    gb: b: B,
    gc: c: C,
    gd: d: D,
    ge: e: E,
    gf: f: F,
    gg: g: G,
    gh: h: H,
    gi: i: I,
    gj: j: J
);
tuple_generator_impl!(
    ga: a: A,
    gb: b: B,
    gc: c: C,
    gd: d: D,
    ge: e: E,
    gf: f: F,
    gg: g: G,
    gh: h: H,
    gi: i: I,
    gj: j: J,
    gk: k: K
);
tuple_generator_impl!(
    ga: a: A,
    gb: b: B,
    gc: c: C,
    gd: d: D,
    ge: e: E,
    gf: f: F,
    gg: g: G,
    gh: h: H,
    gi: i: I,
    gj: j: J,
    gk: k: K,
    gl: l: L
);
