extern crate suppositions;
extern crate log;
extern crate env_logger;

use suppositions::*;
use suppositions::generators::*;


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
                gen.generate(&mut p0.tap()).and_then(|v0| {
                    gen.generate(&mut p1.tap()).map(|v1| (v0, v1))
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
fn uniform_f64s_should_generate_values_between_0_and_1() {
    let gen = uniform_f64s();
    property(
        info_pools(32)
            .filter_map(|p| gen.generate_from(&p))
    ).check(|v| v >= 0.0 && v < 1.0)
}