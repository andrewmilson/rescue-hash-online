//! Traits and implementation of field elements (copied from a personal
//! project).

use core::iter::Sum;
use num_traits::One;
use num_traits::Zero;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::DivAssign;
use std::ops::Mul;
use std::ops::MulAssign;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::SubAssign;

pub mod fp_u128;

/// Trait for field elements that can be represented in 128 bits.
pub trait Felt:
    Copy
    + Clone
    + Debug
    + Display
    + Zero
    + One
    + Eq
    + Hash
    + Neg<Output = Self>
    + Sized
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + AddAssign<Self>
    + SubAssign<Self>
    + MulAssign<Self>
    + DivAssign<Self>
    + for<'a> Add<&'a Self, Output = Self>
    + for<'a> Sub<&'a Self, Output = Self>
    + for<'a> Mul<&'a Self, Output = Self>
    + for<'a> Div<&'a Self, Output = Self>
    + for<'a> AddAssign<&'a Self>
    + for<'a> SubAssign<&'a Self>
    + for<'a> MulAssign<&'a Self>
    + for<'a> DivAssign<&'a Self>
    + Sum<Self>
    + for<'a> Sum<&'a Self>
    + From<u128>
    + From<u64>
    + From<u32>
    + From<u16>
    + From<u8>
{
    /// A multiplicative generator of the entire field except 0.
    const GENERATOR: Self;

    /// Returns `self * self`.
    #[must_use]
    fn square(&self) -> Self {
        *self * self
    }

    /// Squares `self` in place.
    fn square_in_place(&mut self) -> &mut Self {
        *self *= *self;
        self
    }

    /// Returns `self + self`.
    #[must_use]
    fn double(&self) -> Self {
        *self + self
    }

    /// Doubles `self` in place.
    fn double_in_place(&mut self) -> &mut Self {
        *self += *self;
        self
    }

    /// Computes the multiplicative inverse of itself if it exists.
    ///
    /// It should exist if `self` is non-zero.
    #[must_use]
    fn inverse(&self) -> Option<Self>;

    /// Sets, and returns, `self` to its inverse if it exists. Returns `None`
    /// otherwise.
    fn inverse_in_place(&mut self) -> Option<&mut Self>;

    // TODO: remove this method
    /// Montgomery batch inversion.
    ///
    /// Implementation reference:
    /// - https://books.google.com.au/books?id=kGu4lTznRdgC&pg=PA54 (Sec. 5.3)
    /// - https://vitalik.ca/general/2018/07/21/starks_part_3.html
    fn batch_inverse(values: &[Self]) -> Vec<Option<Self>> {
        if values.is_empty() {
            return vec![];
        }

        // compute running multiple of values
        // a, ab, ..., abc...xyz
        let mut accumulator = Self::one();
        let mut partials = vec![Some(accumulator)];
        for value in values.iter() {
            if value.is_zero() {
                partials.push(None);
            } else {
                accumulator *= value;
                partials.push(Some(accumulator));
            }
        }

        // With Montgomery method we only need to calculate one inverse
        // 1/abc...xyz
        accumulator.inverse_in_place();

        // Calculate output values (and update the inverse as we go):
        //   - 1/z = abc...xy * 1/abx...xyz
        //   - 1/y = abc...x * 1/abx...xy
        //   - ...
        //   - 1/b = a * 1/ab
        //   - 1/a = 1 * 1/a
        let mut output = vec![None; values.len()];
        for (i, value) in values.iter().enumerate().rev() {
            if !value.is_zero() {
                if let Some(partial) = partials[i] {
                    output[i] = Some(partial * accumulator);
                    accumulator *= value;
                }
            }
        }

        output
    }

    fn pow(self, power: u128) -> Self {
        let mut res = Self::one();

        if power.is_zero() {
            return Self::one();
        } else if self.is_zero() {
            return Self::zero();
        }

        let mut power = power;
        let mut accumulator = self;

        while !power.is_zero() {
            if (power & 1).is_one() {
                res *= accumulator;
            }
            power >>= 1;
            accumulator.square_in_place();
        }

        res
    }

    /// Returns a canonical integer representation of this field element.
    fn as_integer(&self) -> u128;
}

/// Prime field element.
pub trait PrimeFelt: Felt {
    /// Prime modulus of the field.
    const MODULUS: u128;

    /// Bits needed to represent the modulus
    const BITS: u32;
}
