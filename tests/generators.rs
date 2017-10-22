extern crate suppositions;
extern crate log;
extern crate env_logger;

use suppositions::*;
use suppositions::data::InfoPool;
use suppositions::generators::*;

fn _assert_is_generator<G: Generator>(_: &G) {}

fn info_pools(size: usize) -> Box<Generator<Item = InfoPool>> {
    let g = vecs(u8s()).mean_length(size).map(|v| InfoPool::of_vec(v));
    Box::new(g)
}

#[test]
fn i64s_should_generate_same_output_given_same_input() {
    let gen = i64s();
    property(info_pools(64).filter_map(|p| {
        let v0 = gen.generate_from(&p)?;
        let v1 = gen.generate_from(&p)?;
        Ok((v0, v1))
    })).check(|(v0, v1)| v0 == v1)
}

#[test]
fn i64s_should_partially_order_same_as_source() {
    env_logger::init().unwrap_or(());
    let gen = i64s();
    property(
        (info_pools(16), info_pools(16))
            .filter(|&(ref p0, ref p1)| p0.buffer() < p1.buffer())
            .filter_map(|(p0, p1)| {
                gen.generate(&mut p0.replay()).and_then(|v0| {
                    gen.generate(&mut p1.replay()).map(|v1| (v0, v1))
                })
            }),
    ).check(|(v0, v1)| v0.abs() <= v1.abs())
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
    ).check(|(v0, v1)| v0 == v1)
}

#[test]
fn f64s_should_partially_order_same_as_source() {
    env_logger::init().unwrap_or(());
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
    ).check(|(v0, v1)| v0.abs() <= v1.abs())
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
    ).check(|(v0, v1)| v0 == v1)
}

#[test]
fn uniform_f64s_should_partially_order_same_as_source() {
    env_logger::init().unwrap_or(());
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
    ).check(|(v0, v1)| v0.abs() <= v1.abs())
}

#[test]
fn weighted_coin_should_generate_same_output_given_same_input() {
    let gen = weighted_coin(1.0 / 7.0);
    property(info_pools(32).filter_map(|p| {
        let v0 = gen.generate_from(&p)?;
        let v1 = gen.generate_from(&p)?;
        Ok((v0, v1))
    })).check(|(v0, v1)| v0 == v1)
}

#[test]
fn weighted_coin_should_partially_order_same_as_source() {
    env_logger::init().unwrap_or(());
    let gen = weighted_coin(1.0 / 7.0);
    property(
        (info_pools(16), info_pools(16))
            .filter(|&(ref p0, ref p1)| p0.buffer() < p1.buffer())
            .filter_map(|(p0, p1)| {
                let v0 = gen.generate_from(&p0)?;
                let v1 = gen.generate_from(&p1)?;
                Ok((v0, v1))
            }),
    ).check(|(v0, v1)| v0 <= v1)
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
    env_logger::init().unwrap_or(());
    let gen = one_of(consts(1usize)).or(consts(2)).or(consts(3));
    property(
        (info_pools(16), info_pools(16))
            .filter(|&(ref p0, ref p1)| p0.buffer() < p1.buffer())
            .filter_map(|(p0, p1)| {
                let v0 = gen.generate_from(&p0)?;
                let v1 = gen.generate_from(&p1)?;
                Ok((p0, p1, v0, v1))
            }),
    ).check(|(_, _, v0, v1)| v0 <= v1)
}

#[test]
fn boxed_generator_yields_same_as_inner_value() {
    env_logger::init().unwrap_or(());
    let orig = u64s();
    let boxed = u64s().boxed();
    _assert_is_generator(&boxed);
    property(info_pools(16).filter_map(|p| {
        let v0 = orig.generate_from(&p)?;
        let v1 = boxed.generate_from(&p)?;
        Ok((v0, v1))
    })).check(|(v0, v1)| v0 == v1)
}
