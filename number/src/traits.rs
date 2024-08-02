use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

pub trait Arithmetic:
    Sized
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + RemAssign
{
}

impl<T> Arithmetic for T where
    T: Sized
        + Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + Rem<Output = Self>
        + AddAssign
        + SubAssign
        + MulAssign
        + DivAssign
        + RemAssign
{
}

pub trait Signed: Integer {}

pub trait Unsigned: Integer {}

pub trait Integer: Copy + Arithmetic + Ord {
    const ZERO: Self;
    const ONE: Self;
    const MAX: Self;
    const MIN: Self;
}

macro_rules! impl_integer {
    ($($t:ty),*) => {
        $(
            impl Integer for $t {
                const ZERO: Self = 0;
                const ONE: Self = 1;
                const MAX: Self = <$t>::MAX;
                const MIN: Self = <$t>::MIN;
            }
        )*
    };
}

macro_rules! impl_marker_trait {
    ($trait:ty, $($t:ty),*) => {
        $(
            impl $trait for $t {}
        )*
    };
}

impl_integer!(i8, i16, i32, i64, i128);
impl_integer!(u8, u16, u32, u64, u128);
impl_marker_trait!(Signed, i8, i16, i32, i64, i128);
impl_marker_trait!(Unsigned, u8, u16, u32, u64, u128);
