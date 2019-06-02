extern crate env_logger;
extern crate log;
extern crate suppositions;

use std::fmt;
use suppositions::data::*;
use suppositions::generators::*;
use suppositions::*;

fn _assert_is_generator<G: Generator>(_: &G) {}

#[test]
fn i64s_should_generate_same_output_given_same_input() {
    let gen = i64s();
    property(info_pools(64).filter_map(|p| {
        let v0 = gen.generate_from(&p)?;
        let v1 = gen.generate_from(&p)?;
        Ok((v0, v1))
    }))
    .check(|(v0, v1)| v0 == v1)
}

#[test]
fn i64s_should_partially_order_same_as_source() {
    env_logger::try_init().unwrap_or_default();
    let gen = i64s();
    property(
        (info_pools(16), info_pools(16))
            .filter(|&(ref p0, ref p1)| p0.buffer() < p1.buffer())
            .filter_map(|(p0, p1)| {
                gen.generate(&mut p0.replay())
                    .and_then(|v0| gen.generate(&mut p1.replay()).map(|v1| (v0, v1)))
            }),
    )
    .check(|(v0, v1)| v0.abs() <= v1.abs())
}

#[test]
fn f64s_should_generate_same_output_given_same_input() {
    let gen = f64s();
    property(
        info_pools(32)
            .filter_map(|p| {
                let v0 = gen.generate_from(&p)?;
                let v1 = gen.generate_from(&p)?;
                Ok((v0, v1))
            })
            .filter(|&(v0, v1)| !(v0.is_nan() || v1.is_nan())),
    )
    .check(|(v0, v1)| v0 == v1)
}

#[test]
fn f64s_should_partially_order_same_as_source() {
    env_logger::try_init().unwrap_or_default();
    let gen = f64s();
    property(
        (info_pools(16), info_pools(16))
            .filter(|&(ref p0, ref p1)| p0.buffer() < p1.buffer())
            .filter_map(|(p0, p1)| {
                let v0 = gen.generate_from(&p0)?;
                let v1 = gen.generate_from(&p1)?;
                Ok((v0, v1))
            })
            .filter(|&(v0, v1)| !(v0.is_nan() || v1.is_nan())),
    )
    .check(|(v0, v1)| v0.abs() <= v1.abs())
}

#[test]
fn uniform_f64s_should_generate_same_output_given_same_input() {
    let gen = uniform_f64s();
    property(
        info_pools(32)
            .filter_map(|p| {
                let v0 = gen.generate_from(&p)?;
                let v1 = gen.generate_from(&p)?;
                Ok((v0, v1))
            })
            .filter(|&(v0, v1)| !(v0.is_nan() || v1.is_nan())),
    )
    .check(|(v0, v1)| v0 == v1)
}

#[test]
fn uniform_f64s_should_partially_order_same_as_source() {
    env_logger::try_init().unwrap_or_default();
    let gen = uniform_f64s();
    property(
        (info_pools(16), info_pools(16))
            .filter(|&(ref p0, ref p1)| p0.buffer() < p1.buffer())
            .filter_map(|(p0, p1)| {
                let v0 = gen.generate_from(&p0)?;
                let v1 = gen.generate_from(&p1)?;
                Ok((v0, v1))
            })
            .filter(|&(v0, v1)| !(v0.is_nan() || v1.is_nan())),
    )
    .check(|(v0, v1)| v0.abs() <= v1.abs())
}

#[test]
fn weighted_coin_should_generate_same_output_given_same_input() {
    let gen = weighted_coin(1.0 / 7.0);
    property(info_pools(32).filter_map(|p| {
        let v0 = gen.generate_from(&p)?;
        let v1 = gen.generate_from(&p)?;
        Ok((v0, v1))
    }))
    .check(|(v0, v1)| v0 == v1)
}

#[test]
fn weighted_coin_should_partially_order_same_as_source() {
    env_logger::try_init().unwrap_or_default();
    let gen = weighted_coin(1.0 / 7.0);
    property(
        (info_pools(16), info_pools(16))
            .filter(|&(ref p0, ref p1)| p0.buffer() < p1.buffer())
            .filter_map(|(p0, p1)| {
                let v0 = gen.generate_from(&p0)?;
                let v1 = gen.generate_from(&p1)?;
                Ok((v0, v1))
            }),
    )
    .check(|(v0, v1)| v0 <= v1)
}

#[test]
fn uniform_f64s_should_generate_values_between_0_and_1() {
    let gen = uniform_f64s();
    property(info_pools(32).filter_map(|p| gen.generate_from(&p))).check(|v| v >= 0.0 && v < 1.0)
}

#[test]
fn generator_map_should_trivially_preserve_invariants() {
    property(u8s().map(|v| (v as u16) * 2)).check(|v| v % 2 == 0)
}

#[test]
fn one_of_should_pick_a_single_sample() {
    let g = one_of(consts(1usize)).or(consts(2)).or(consts(3));
    property(g).check(|v| v >= 1 && v <= 3)
}

#[test]
fn one_of_should_partially_order_same_as_source() {
    env_logger::try_init().unwrap_or_default();
    let gen = one_of(consts(1usize)).or(consts(2)).or(consts(3));
    property(
        (info_pools(16), info_pools(16))
            .filter(|&(ref p0, ref p1)| p0.buffer() < p1.buffer())
            .filter_map(|(p0, p1)| {
                let v0 = gen.generate_from(&p0)?;
                let v1 = gen.generate_from(&p1)?;
                Ok((p0, p1, v0, v1))
            }),
    )
    .check(|(_, _, v0, v1)| v0 <= v1)
}

#[test]
fn boxed_generator_yields_same_as_inner_value() {
    env_logger::try_init().unwrap_or_default();
    let orig = u64s();
    let boxed = u64s().boxed();
    _assert_is_generator(&boxed);
    property(info_pools(16).filter_map(|p| {
        let v0 = orig.generate_from(&p)?;
        let v1 = boxed.generate_from(&p)?;
        Ok((v0, v1))
    }))
    .check(|(v0, v1)| v0 == v1)
}

#[test]
fn generator_of_2_tuple() {
    let g = (u64s(), u32s());
    _assert_is_generator(&g);
}

#[test]
fn generator_of_3_tuple() {
    let g = (u64s(), u32s(), u8s());
    _assert_is_generator(&g);
}

#[test]
fn lazy_generator_yields_same_as_inner_value() {
    env_logger::try_init().unwrap_or_default();
    let orig = u64s();
    let lazy = lazy(u64s);
    _assert_is_generator(&lazy);
    property(info_pools(16).filter_map(|p| {
        let v0 = orig.generate_from(&p)?;
        let v1 = lazy.generate_from(&p)?;
        Ok((v0, v1))
    }))
    .check(|(v0, v1)| v0 == v1)
}

fn uptos_never_generates_greater_than_limit<G: Generator + Clone>(g: G)
where
    G::Item: ScaleInt + Copy + fmt::Debug + PartialOrd,
{
    env_logger::try_init().unwrap_or_default();
    property(g.clone().flat_map(|max| {
        let h = uptos(g.clone(), max);
        _assert_is_generator(&h);
        h.map(move |n| (n, max))
    }))
    .check(|(n, max)| n <= max);
}

#[test]
fn uptos_u8_is_gen() {
    _assert_is_generator(&uptos(u8s(), ::std::u8::MAX))
}

#[test]
fn uptos_u8_never_generates_greater_than_limit() {
    uptos_never_generates_greater_than_limit(u8s())
}

#[test]
fn uptos_u16_never_generates_greater_than_limit() {
    uptos_never_generates_greater_than_limit(u16s())
}

#[test]
fn uptos_u32_never_generates_greater_than_limit() {
    uptos_never_generates_greater_than_limit(u32s())
}

#[test]
fn uptos_u64_never_generates_greater_than_limit() {
    uptos_never_generates_greater_than_limit(u64s())
}

struct RegionCounter<S> {
    src: S,
    cnt: usize,
}

impl<G: InfoSource> InfoSource for RegionCounter<G> {
    fn draw_u8(&mut self) -> u8 {
        self.src.draw_u8()
    }
    fn draw<S: InfoSink>(&mut self, sink: S) -> S::Out
    where
        Self: Sized,
    {
        self.cnt += 1;
        self.src.draw(sink)
    }
}

#[test]
fn optional_u64s_should_have_one_region_for_none() {
    let g = optional(u64s());

    property(
        info_pools(32)
            .filter_map(|p| {
                let mut ctr = RegionCounter {
                    src: &mut p.replay(),
                    cnt: 0,
                };
                g.generate(&mut ctr).map(|val| (ctr.cnt, val))
            })
            .filter(|&(_, ref val)| val.is_none()),
    )
    .check(|(cnt, _)| assert_eq!(1, cnt));
}

#[test]
fn optional_u64s_should_have_two_regions_for_some() {
    let g = optional(u64s());

    property(
        info_pools(32)
            .filter_map(|p| {
                let mut ctr = RegionCounter {
                    src: &mut p.replay(),
                    cnt: 0,
                };
                g.generate(&mut ctr).map(|val| (ctr.cnt, val))
            })
            .filter(|&(_, ref val)| val.is_some()),
    )
    .check(|(cnt, _)| assert_eq!(2, cnt));
}

#[test]
fn choice_should_always_draw_from_inputs() {
    property((vecs(u64s()), info_pools(32)).filter(|&(ref its, _)| its.len() > 0)).check(
        |(its, p)| {
            let choice = choice(its.clone())
                .generate_from(&p)
                .expect("generate_from");
            assert!(its.contains(&choice));
        },
    );
}

#[test]
fn choice_should_skip_on_empty_choices() {
    property(info_pools(32)).check(|p| {
        let result = choice(vec![0u8; 0]).generate_from(&p);
        assert_eq!(result, Err(DataError::SkipItem));
    });
}

#[test]
fn map_should_yield_generated_value_but_with_f_applied_for_usize() {
    fn add_one(i: usize) -> usize {
        i.wrapping_add(i)
    }
    let f = add_one;
    let g = usizes();
    map_should_yield_generated_value_but_with_f_applied(&g, &f);
}

fn map_should_yield_generated_value_but_with_f_applied<
    G: Generator,
    R: fmt::Debug + PartialEq,
    F: Fn(G::Item) -> R,
>(
    g: &G,
    f: &F,
) {
    // Check that g.map(f).generate(...) == f(g.generate(...))
    // (Modulo generator failure)
    property(info_pools(32)).check(move |p| {
        assert_eq!(
            g.generate_from(&p).map(f),
            g.clone().map(f).generate_from(&p)
        )
    });
}

// These are inspired by https://hackage.haskell.org/package/checkers-0.4.9.5/docs/src/Test-QuickCheck-Classes.html#monad

#[test]
fn flat_map_should_have_left_identity_over_usize() {
    let f = |i: usize| consts(i.wrapping_add(1));
    let gen = usizes();
    flat_map_should_have_left_identity(&gen, &f);
}

fn flat_map_should_have_left_identity<G: Generator, H: Generator, F: Fn(G::Item) -> H>(g: &G, f: &F)
where
    G::Item: fmt::Debug + Clone,
    H::Item: fmt::Debug + PartialEq,
{
    property(info_pools(32).filter_map(|p| {
        let a = g.generate_from(&p)?;
        Ok((p, a))
    }))
    .check(|(p, a)| -> Result<(), DataError> {
        let lhs = consts(a.clone()).flat_map(f);
        let rhs = f(a);

        assert_eq!(lhs.generate_from(&p), rhs.generate_from(&p));

        Ok(())
    });
}

#[test]
fn flat_map_should_have_right_identity_over_usize() {
    flat_map_should_have_right_identity(&usizes());
}

fn flat_map_should_have_right_identity<G: Generator>(g: &G)
where
    G::Item: fmt::Debug + PartialEq + Clone,
{
    property(info_pools(32)).check(|p| -> Result<(), DataError> {
        let lhs = g.clone().flat_map(consts);
        let rhs = g;

        assert_eq!(lhs.generate_from(&p), rhs.generate_from(&p));

        Ok(())
    });
}

#[test]
fn flat_map_should_be_associative_over_canary_types() {
    // I can't think of a better example than this rather contrived thing.
    flat_map_should_be_associative(&usizes(), &|l| vecs(u8s()).mean_length(l), &|vs| choice(vs));
}

fn flat_map_should_be_associative<
    G: Generator,
    H: Generator,
    I: Generator,
    E: Fn(G::Item) -> H,
    F: Fn(H::Item) -> I,
>(
    gen: &G,
    e: &E,
    f: &F,
) where
    I::Item: fmt::Debug + PartialEq,
{
    // Check that g.map(f).generate(...) == f(g.generate(...))
    // (return a >>= f)  =-= f a
    // (Modulo generator failure)
    property(info_pools(32)).check(|p| -> Result<(), DataError> {
        let lhs = gen.flat_map(e).flat_map(f);
        let rhs = gen.flat_map(|x| e(x).flat_map(f));

        assert_eq!(lhs.generate_from(&p), rhs.generate_from(&p));

        Ok(())
    });
}
