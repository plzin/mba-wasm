//! [Rings](https://en.wikipedia.org/wiki/Ring_(mathematics)) with additional structure.

use std::ops::{
    IndexMut, Index, BitAnd, BitOr, BitXor, Not,
    Shl, ShlAssign, Add, Sub, Mul, Div, Rem,
    AddAssign, DivAssign, RemAssign, MulAssign, SubAssign
};
use std::fmt::{self, Formatter, Display};
use num_traits::{Num, NumAssign, Unsigned, Signed, Zero, One};

/// The integers mod n.
/// Representatives in the range 0..n are stored.
pub trait UnsignedInt: NumAssign + Copy + Ord + Unsigned + Display {
    /// Should the number be printed as a negative number.
    fn print_negative(self) -> bool {
        false
    }

    /// Fast conversion from u8.
    fn from_u8(v: u8) -> Self;
}

/// Parses an integer in base ten from the iterator.
pub(crate) fn int_from_it<T: UnsignedInt>(
    it: &mut std::iter::Peekable<std::str::Chars>
) -> Option<T> {
    let neg = *it.peek()? == '-';
    if neg {
        it.next();
    }

    // Is the character an ascii digit?
    if !it.peek().map_or(false, |c| c.is_ascii_digit()) {
        return None;
    }

    let ten = T::from_u8(10);

    // Parse the number.
    let mut n = T::zero();
    loop {
        // Is this still a digit?
        let Some(d) = it.peek().and_then(|c| c.to_digit(10)) else {
            break
        };

        n *= ten;
        n += T::from_u8(d as u8);
        it.next();
    }

    if neg {
        n = T::zero() - n;        
    }

    Some(n)
}

/// N-bit integers basically.
pub trait UniformNum: UnsignedInt
    + BitAnd<Self, Output = Self>
    + BitOr<Self, Output = Self>
    + BitXor<Self, Output = Self>
    + Shl<usize>
    + ShlAssign<usize>
    + Not<Output = Self> {}

impl UniformNum for std::num::Wrapping<u8> {}
impl UniformNum for std::num::Wrapping<u16> {}
impl UniformNum for std::num::Wrapping<u32> {}
impl UniformNum for std::num::Wrapping<u64> {}
impl UniformNum for std::num::Wrapping<u128> {}

macro_rules! impl_uint {
    ($impl_ty:ty) => {
        impl UnsignedInt for std::num::Wrapping<$impl_ty> {
            fn print_negative(self) -> bool {
                self.0 > (1 << (<$impl_ty>::BITS - 1))
            }

            fn from_u8(v: u8) -> Self {
                std::num::Wrapping(v as $impl_ty)
            }
        }
    }
}

impl_uint!(u8);
impl_uint!(u16);
impl_uint!(u32);
impl_uint!(u64);
impl_uint!(u128);
impl_uint!(usize);