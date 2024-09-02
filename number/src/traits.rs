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

pub trait SaturatingOps: Arithmetic {
    fn saturating_add(self, rhs: Self) -> Self;
    fn saturating_sub(self, rhs: Self) -> Self;
    fn saturating_mul(self, rhs: Self) -> Self;
    fn saturating_div(self, rhs: Self) -> Self;
    fn saturating_pow(self, exp: u32) -> Self;
}

pub trait WrappingOps: Arithmetic {
    fn wrapping_add(self, rhs: Self) -> Self;
    fn wrapping_sub(self, rhs: Self) -> Self;
    fn wrapping_mul(self, rhs: Self) -> Self;
    fn wrapping_div(self, rhs: Self) -> Self;
    fn wrapping_neg(self) -> Self;
    fn wrapping_rem(self, rhs: Self) -> Self;
    fn wrapping_pow(self, exp: u32) -> Self;
}

macro_rules! impl_ops {
    ($($t:ty),* $(,)?) => {
        $(
            impl SaturatingOps for $t {
                fn saturating_add(self, rhs: Self) -> Self {
                    self.saturating_add(rhs)
                }
                fn saturating_sub(self, rhs: Self) -> Self {
                    self.saturating_sub(rhs)
                }
                fn saturating_mul(self, rhs: Self) -> Self {
                    self.saturating_mul(rhs)
                }
                fn saturating_div(self, rhs: Self) -> Self {
                    self.saturating_div(rhs)
                }
                fn saturating_pow(self, exp: u32) -> Self {
                    self.saturating_pow(exp)
                }
            }

            impl WrappingOps for $t {
                fn wrapping_add(self, rhs: Self) -> Self {
                    self.wrapping_add(rhs)
                }
                fn wrapping_sub(self, rhs: Self) -> Self {
                    self.wrapping_sub(rhs)
                }
                fn wrapping_mul(self, rhs: Self) -> Self {
                    self.wrapping_mul(rhs)
                }
                fn wrapping_div(self, rhs: Self) -> Self {
                    self.wrapping_div(rhs)
                }
                fn wrapping_neg(self) -> Self {
                    self.wrapping_neg()
                }
                fn wrapping_rem(self, rhs: Self) -> Self {
                    self.wrapping_rem(rhs)
                }
                fn wrapping_pow(self, exp: u32) -> Self {
                    self.wrapping_pow(exp)
                }
            }
        )*
    };
}

impl_ops!(i8, i16, i32, i64, i128, isize);
impl_ops!(u8, u16, u32, u64, u128, usize);

#[cfg(test)]
mod tests {
    #[test]
    fn check() {
        
    }
}