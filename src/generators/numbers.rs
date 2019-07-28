use data::*;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops;

use super::core::*;

/// See [`u64s`](fn.u64s.html), [`i64s`](fn.i64s.html), etc.
#[derive(Debug, Clone)]
pub struct IntGenerator<N>(PhantomData<N>);
/// See [`f32s`](fn.f32s.html)
/// or [`f64s`](fn.f64s.html)
#[derive(Debug, Clone)]
pub struct FloatGenerator<N>(PhantomData<N>);
/// See [`uniform_f32s`](fn.uniform_f32s.html)
/// or [`uniform_f64s`](fn.uniform_f64s.html)
#[derive(Debug, Clone)]
pub struct UniformFloatGenerator<N>(PhantomData<N>);

/// See [`uptos`](fn.uptos.html)
#[derive(Debug, Clone)]
pub struct UptoGenerator<G: Generator>(G, G::Item);

impl<T, N: Copy + ScaleInt> IntGenerator<T>
where
    IntGenerator<T>: Generator<Item = N>,
{
    /// Scales the output of g generating an unsigned integer up to max. See also [`uptos`](fn.uptos.html)
    pub fn upto(self, max: N) -> impl Generator<Item = N> {
        uptos(self, max)
    }
}

impl<T, N: Copy + ScaleInt + ops::Sub<N, Output = N> + ops::Add<N, Output = N>> IntGenerator<T>
where
    IntGenerator<T>: Generator<Item = N>,
{
    /// Yields a value beween min and max, inclusive of both.
    pub fn between(self, min: N, max: N) -> impl Generator<Item = N> {
        let diff = max - min;
        uptos(self, diff).map(move |x| x + min)
    }
}

macro_rules! unsigned_integer_gen {
    ($name:ident, $ty:ty) => {
        /// A generator that generates integers of the specified type.
        pub fn $name() -> IntGenerator<$ty> {
            IntGenerator(PhantomData)
        }

        impl Generator for IntGenerator<$ty> {
            type Item = $ty;
            fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
                assert!(size_of::<u8>() == 1);
                let nbytes = size_of::<$ty>() / size_of::<u8>();
                let mut val: $ty = 0;
                for _ in 0..nbytes {
                    val = val.wrapping_shl(8) | src.draw_u8() as $ty;
                }
                Ok(val)
            }
        }
    };
}

unsigned_integer_gen!(u8s, u8);
unsigned_integer_gen!(u16s, u16);
unsigned_integer_gen!(u32s, u32);
unsigned_integer_gen!(u64s, u64);
unsigned_integer_gen!(usizes, usize);

/// Scales the output of g generating an unsigned integer upto max.
pub fn uptos<G: Generator>(g: G, max: G::Item) -> UptoGenerator<G> {
    UptoGenerator(g, max)
}

#[doc(hidden)]
pub trait ScaleInt {
    fn scale(self, max: Self) -> Self;
}

impl<G: Generator> Generator for UptoGenerator<G>
where
    G::Item: ScaleInt + Copy,
{
    type Item = G::Item;

    fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
        let &UptoGenerator(ref gen, ref limit) = self;

        let v = gen.generate(src)?;

        Ok(v.scale(limit.clone()))
    }
}

macro_rules! scale_int_impl {
    ($ty: ident, $next_size: ident) => {
        impl ScaleInt for $ty {
            fn scale(self, max: Self) -> Self {
                let shift = ::std::mem::size_of::<$ty>() * 8;
                let res = (self as $next_size * max as $next_size) >> shift;
                res as Self
            }
        }
    };
}

// this is a macro because:
// * I wnat to be able to test that this is correct against smaller than u64 values, and
// * rust std numerics aren't so useful for doing that generically.
macro_rules! multiply_limbs_impl {
    ($name: ident, $ty: ident) => {
        fn $name(a: $ty, b: $ty) -> ($ty, $ty) {
            let shift_bits = ::std::mem::size_of::<$ty>() * 8;
            let half_bits = shift_bits / 2;
            let mask = (1 << half_bits) - 1;

            let al = a & mask;
            let ah = a >> half_bits;
            let bl = b & mask;
            let bh = b >> half_bits;

            let ll = al * bl;
            let lh = al * bh;
            let hl = ah * bl;
            let hh = ah * bh;

            // println!("{:#02x} * {:#02x}; [[{:#02x}, {:#02x}], [{:#02x}, {:#02x}]]", a, b, hh, hl, lh, ll);

            let (lower, lov) = ll.overflowing_add(lh << half_bits);
            let (lower, lov2) = lower.overflowing_add(hl << half_bits);
            let upper = hh
                + if lov { 1 } else { 0 }
                + if lov2 { 1 } else { 0 }
                + (lh >> half_bits)
                + (hl >> half_bits);

            (upper, lower)
        }
    };
}

scale_int_impl!(u8, u16);
scale_int_impl!(u16, u32);
scale_int_impl!(u32, u64);

impl ScaleInt for u64 {
    fn scale(self, max: Self) -> Self {
        multiply_limbs_impl!(mul_u64s, u64);
        let (h, _) = mul_u64s(self, max);
        h
    }
}

impl ScaleInt for usize {
    fn scale(self, max: Self) -> Self {
        assert_eq!(::std::mem::size_of::<usize>(), ::std::mem::size_of::<u64>());
        u64::scale(self as u64, max as u64) as usize
    }
}

trait One {
    fn one() -> Self;
}

// We use the equivalent unsigned generator as an intermediate
macro_rules! signed_integer_gen {
    ($name:ident, $ugen:expr, $ty:ty) => {
        /// A generator that generates the full range of the specified type.
        pub fn $name() -> IntGenerator<$ty> {
            IntGenerator(PhantomData)
        }

        impl Generator for IntGenerator<$ty> {
            type Item = $ty;
            fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
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
    };
}

signed_integer_gen!(i8s, u8s(), i8);
signed_integer_gen!(i16s, u16s(), i16);
signed_integer_gen!(i32s, u32s(), i32);
signed_integer_gen!(i64s, u64s(), i64);
signed_integer_gen!(isizes, usizes(), isize);

// As with signed types, use the equivalent unsigned generator as an intermediate
macro_rules! float_gen {
    ($name:ident, $ugen:expr, $ty:ident) => {
        /// Generates values that encompass all possible float values
        /// (positive and negative), including NaN, and sub-normal values.
        pub fn $name() -> FloatGenerator<$ty> {
            FloatGenerator(PhantomData)
        }

        impl Generator for FloatGenerator<$ty> {
            type Item = $ty;
            fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
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
    };
}

float_gen!(f32s, u32s(), f32);
float_gen!(f64s, u64s(), f64);

// As with signed types, use the equivalent unsigned generator as an intermediate
macro_rules! uniform_float_gen {
    ($name:ident, $ugen:expr, $inty:ident, $ty:ident) => {
        /// Generates values that are uniformly distributed, such that the
        /// output value x satisifes 0.0 <= x < 1.0
        pub fn $name() -> UniformFloatGenerator<$ty> {
            UniformFloatGenerator(PhantomData)
        }

        impl Generator for UniformFloatGenerator<$ty> {
            type Item = $ty;
            fn generate<I: InfoSource>(&self, src: &mut I) -> Maybe<Self::Item> {
                let inner_g = $ugen;
                let uval = inner_g.generate(src)?;
                return Ok(uval as $ty / $inty::max_value() as $ty);
            }
        }
    };
}

uniform_float_gen!(uniform_f32s, u32s(), u32, f32);
uniform_float_gen!(uniform_f64s, u64s(), u64, f64);

#[cfg(test)]
mod tests {
    use generators::core::tests::*;
    use generators::numbers::*;

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
    fn upto_u8s_should_generate_same_output_given_same_input() {
        should_generate_same_output_given_same_input(uptos(u8s(), 7))
    }

    #[test]
    fn upto_u8s_usually_generates_different_output_for_different_inputs() {
        usually_generates_different_output_for_different_inputs(uptos(u8s(), 7));
    }

    #[test]
    fn upto_u8s_minimize_to_minimum_value() {
        should_minimize_to(uptos(u8s(), 7), 0)
    }

    #[test]
    fn upto_u8s_should_partially_order_same_as_source() {
        should_partially_order_same_as_source(uptos(u8s(), 7));
    }

    multiply_limbs_impl!(mul_u8, u8);

    #[test]
    fn multiply_with_carries_works() {
        let a = 123;
        let b = 113;
        // 123 * 113 = 13899 or 0x364b
        let expected = (0x36, 0x4b);

        assert_eq!(mul_u8(a, b), expected);
    }

    #[test]
    fn prop_multiply_with_carries_works() {
        use generators::*;
        use *;

        property((u8s(), u8s())).check(|(a, b)| {
            let product = (a as u16) * (b as u16);
            let (res_h, res_l) = mul_u8(a, b);
            let res = ((res_h as u16) << 8) | res_l as u16;
            println!(
                "a:{:#02x} * b:{:#02x} =? {:#04x} (expected {:#04x})",
                a, b, res, product
            );
            assert_eq!(res, product)
        });
    }
}
