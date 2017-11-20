use std::marker::PhantomData;
use std::mem::size_of;
use data::*;

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

macro_rules! unsigned_integer_gen {
    ($name:ident, $ty:ty) => {
        /// A generator that generates integers of the specified type.
        pub fn $name() -> IntGenerator<$ty> {
            IntGenerator(PhantomData)
        }

        impl Generator for IntGenerator<$ty> {
            type Item = $ty;
            fn generate<I: Iterator<Item = u8>>(&self, src: &mut I) -> Maybe<Self::Item> {
                assert!(size_of::<u8>() == 1);
                let nbytes = size_of::<$ty>() / size_of::<u8>();
                let mut val: $ty = 0;
                for _ in 0..nbytes {
                    val = val.wrapping_shl(8) | src.next().ok_or(DataError::PoolExhausted)?
 as $ty;
                }
                Ok(val)
            }
        }
    }
}

unsigned_integer_gen!(u8s, u8);
unsigned_integer_gen!(u16s, u16);
unsigned_integer_gen!(u32s, u32);
unsigned_integer_gen!(u64s, u64);
unsigned_integer_gen!(usizes, usize);

// We use the equivalent unsigned generator as an intermediate
macro_rules! signed_integer_gen {
    ($name:ident, $ugen:expr, $ty:ty) => {
        /// A generator that generates the full range of the specified type.
        pub fn $name() -> IntGenerator<$ty> {
            IntGenerator(PhantomData)
        }

        impl Generator for IntGenerator<$ty> {
            type Item = $ty;
            fn generate<I: Iterator<Item = u8>>(&self, src: &mut I) -> Maybe<Self::Item> {
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
    }
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
            fn generate<I: Iterator<Item = u8>>(&self, src: &mut I) -> Maybe<Self::Item> {
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
    }
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
            fn generate<I: Iterator<Item = u8>>(&self, src: &mut I) -> Maybe<Self::Item> {
                let inner_g = $ugen;
                let uval = inner_g.generate(src)?;
                return Ok(uval as $ty / $inty::max_value() as $ty);
            }
        }
    }
}

uniform_float_gen!(uniform_f32s, u32s(), u32, f32);
uniform_float_gen!(uniform_f64s, u64s(), u64, f64);

#[cfg(test)]
mod tests {
    use generators::numbers::*;
    use generators::core::tests::*;

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
}
