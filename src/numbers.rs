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
    // Should the number be printed as a negative number.
    fn print_negative(self) -> bool {
        false
    }
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
        }
    }
}

impl_uint!(u8);
impl_uint!(u16);
impl_uint!(u32);
impl_uint!(u64);
impl_uint!(u128);
impl_uint!(usize);

/// N-Bit integers. N has to be less than or equal to the width of usize.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Bits<const N: usize>(pub std::num::Wrapping<u64>);

impl<const N: usize> Bits<N> {
    const MASK: u64 = (1 << N) - 1;

    /// Apply the mask.
    fn m(self) -> Self {
        Self(std::num::Wrapping(self.0.0 & Self::MASK))
    }
}

impl<const N: usize> Num for Bits<N> {
    type FromStrRadixErr = std::num::ParseIntError;
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self(std::num::Wrapping(u64::from_str_radix(str, radix)?)).m())
    }
}

impl<const N: usize> NumAssign for Bits<N> {}

impl<const N: usize> UnsignedInt for Bits<N> {
    fn print_negative(self) -> bool {
        self.0.0 > (1 << (N - 1))
    }
}

impl<const N: usize> UniformNum for Bits<N> {}

impl<const N: usize> Zero for Bits<N> {
    fn zero() -> Self {
        Self(std::num::Wrapping(0))
    }

    fn is_zero(&self) -> bool {
        self.0.0 == 0
    }
}

impl<const N: usize> One for Bits<N> {
    fn one() -> Self {
        Self(std::num::Wrapping(1))
    }
}

impl<const N: usize> Unsigned for Bits<N> {}

impl<const N: usize> Add for Bits<N> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0).m()
    }
}

impl<const N: usize> AddAssign for Bits<N> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.0 &= Self::MASK;
    }
}

impl<const N: usize> Sub for Bits<N> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0).m()
    }
}

impl<const N: usize> SubAssign for Bits<N> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
        self.0 &= Self::MASK;
    }
}

impl<const N: usize> Mul for Bits<N> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0).m()
    }
}

impl<const N: usize> MulAssign for Bits<N> {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
        self.0 &= Self::MASK;
    }
}

impl<const N: usize> Div for Bits<N> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0).m()
    }
}

impl<const N: usize> DivAssign for Bits<N> {
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
        self.0 &= Self::MASK;
    }
}

impl<const N: usize> Rem for Bits<N> {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0).m()
    }
}

impl<const N: usize> RemAssign for Bits<N> {
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0;
        self.0 &= Self::MASK;
    }
}

impl<const N: usize> BitAnd for Bits<N> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl<const N: usize> BitOr for Bits<N> {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl<const N: usize> BitXor for Bits<N> {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl<const N: usize> Shl<usize> for Bits<N> {
    type Output = Self;
    fn shl(self, rhs: usize) -> Self::Output {
        Self(self.0 << rhs).m()
    }
}

impl<const N: usize> ShlAssign<usize> for Bits<N> {
    fn shl_assign(&mut self, rhs: usize) {
        self.0 <<= rhs;
        self.0 &= Self::MASK;
    }
}

impl<const N: usize> Not for Bits<N> {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0).m()
    }
}

impl<const N: usize> Display for Bits<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

macro_rules! uints {
    () => {};
    ($name:tt $bits:tt $($o:tt)*) => {
        #[allow(non_camel_case_types)]
        pub type $name = Bits<$bits>;
        uints!($($o)*);
    };
}

uints!(
    u1 1 u2 2 u3 3 u4 4 u5 5 u6 6 u7 7 u9 9 u10 10 u11 11 u12 12 u13 13
    u14 14 u15 15 u17 17 u18 18 u19 19 u20 20 u21 21 u22 22 u23 23 u24 24
    u25 25 u26 26 u27 27 u28 28 u29 29 u30 30 u31 31 u33 33 u34 34 u35 35
    u36 36 u37 37 u38 38 u39 39 u40 40 u41 41 u42 42 u43 43 u44 44 u45 45
    u46 46 u47 47 u48 48 u49 49 u50 50 u51 51 u52 52 u53 53 u54 54 u55 55
    u56 56 u57 57 u58 58 u59 59 u60 60 u61 61 u62 62 u63 63
);