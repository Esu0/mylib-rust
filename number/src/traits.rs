use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

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
    type BitWidthType;
    fn pow(self, exp: Self::BitWidthType) -> Self;
    fn div_euclid(self, rhs: Self) -> Self;
    fn rem_euclid(self, rhs: Self) -> Self;
}

pub trait BitArithmetic:
    Sized
    + Arithmetic
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Shl<Self::BitWidthType, Output = Self>
    + Shr<Self::BitWidthType, Output = Self>
    + BitAndAssign
    + BitOrAssign
    + BitXorAssign
    + ShlAssign<Self::BitWidthType>
    + ShrAssign<Self::BitWidthType>
{
}

macro_rules! impl_arithmetic_for_int {
    ($($t:ty),*) => {
        $(
            impl Arithmetic for $t {
                type BitWidthType = u32;
                fn pow(self, exp: Self::BitWidthType) -> Self {
                    self.pow(exp)
                }

                fn div_euclid(self, rhs: Self) -> Self {
                    self.div_euclid(rhs)
                }

                fn rem_euclid(self, rhs: Self) -> Self {
                    self.rem_euclid(rhs)
                }
            }

            impl BitArithmetic for $t {}
        )*
    };
}

impl_arithmetic_for_int!(i8, i16, i32, i64, i128, isize);
impl_arithmetic_for_int!(u8, u16, u32, u64, u128, usize);

pub trait Signed: Integer {}

pub trait Unsigned: Integer {}

pub trait FixedWidth: Integer {}

pub trait ExistsBiggerInt: Integer {
    type Twice: Integer;
    fn cast_to_twice(self) -> Self::Twice;
    fn cast_from_twice(twice: Self::Twice) -> Self;
}

macro_rules! impl_exists_bigger_int {
    ($($t1:ty: $t2:ty),*) => {
        $(
            impl ExistsBiggerInt for $t1 {
                type Twice = $t2;
                fn cast_to_twice(self) -> Self::Twice {
                    self as $t2
                }

                fn cast_from_twice(twice: Self::Twice) -> Self {
                    twice as $t1
                }
            }
        )*
    }
}

pub trait Integer: Arithmetic + Ord {
    fn zero() -> Self;
    fn one() -> Self;
    fn max_value() -> Self;
    fn min_value() -> Self;
}

macro_rules! impl_integer {
    ($($t:ty),*) => {
        $(
            impl Integer for $t {
                fn zero() -> Self { 0 }
                fn one() -> Self { 1 }
                fn max_value() -> Self { <$t>::MAX }
                fn min_value() -> Self { <$t>::MIN }
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

impl_integer!(i8, i16, i32, i64, i128, isize);
impl_integer!(u8, u16, u32, u64, u128, usize);
impl_marker_trait!(Signed, i8, i16, i32, i64, i128, isize);
impl_marker_trait!(Unsigned, u8, u16, u32, u64, u128, usize);
impl_marker_trait!(FixedWidth, i8, i16, i32, i64, i128, isize);
impl_marker_trait!(FixedWidth, u8, u16, u32, u64, u128, usize);
impl_exists_bigger_int!(i8: i16, i16: i32, i32: i64, i64: i128);
impl_exists_bigger_int!(u8: u16, u16: u32, u32: u64, u64: u128);

pub trait SaturatingOps: Arithmetic {
    fn saturating_add(self, rhs: Self) -> Self;
    fn saturating_sub(self, rhs: Self) -> Self;
    fn saturating_mul(self, rhs: Self) -> Self;
    fn saturating_div(self, rhs: Self) -> Self;
    fn saturating_pow(self, exp: Self::BitWidthType) -> Self;
}

pub trait WrappingOps: Arithmetic {
    fn wrapping_add(self, rhs: Self) -> Self;
    fn wrapping_sub(self, rhs: Self) -> Self;
    fn wrapping_mul(self, rhs: Self) -> Self;
    fn wrapping_div(self, rhs: Self) -> Self;
    fn wrapping_neg(self) -> Self;
    fn wrapping_rem(self, rhs: Self) -> Self;
    fn wrapping_pow(self, exp: Self::BitWidthType) -> Self;
}

pub trait OverflowingOps: Arithmetic {
    fn overflowing_add(self, rhs: Self) -> (Self, bool);
    fn overflowing_sub(self, rhs: Self) -> (Self, bool);
    fn overflowing_mul(self, rhs: Self) -> (Self, bool);
    fn overflowing_div(self, rhs: Self) -> (Self, bool);
    fn overflowing_neg(self) -> (Self, bool);
    fn overflowing_rem(self, rhs: Self) -> (Self, bool);
    fn overflowing_pow(self, exp: Self::BitWidthType) -> (Self, bool);
}

pub trait CheckedOps: Arithmetic {
    fn checked_add(self, rhs: Self) -> Option<Self>;
    fn checked_sub(self, rhs: Self) -> Option<Self>;
    fn checked_mul(self, rhs: Self) -> Option<Self>;
    fn checked_div(self, rhs: Self) -> Option<Self>;
    fn checked_neg(self) -> Option<Self>;
    fn checked_rem(self, rhs: Self) -> Option<Self>;
    fn checked_pow(self, exp: Self::BitWidthType) -> Option<Self>;
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
                fn saturating_pow(self, exp: Self::BitWidthType) -> Self {
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
                fn wrapping_pow(self, exp: Self::BitWidthType) -> Self {
                    self.wrapping_pow(exp)
                }
            }

            impl OverflowingOps for $t {
                fn overflowing_add(self, rhs: Self) -> (Self, bool) {
                    self.overflowing_add(rhs)
                }

                fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
                    self.overflowing_sub(rhs)
                }

                fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
                    self.overflowing_mul(rhs)
                }

                fn overflowing_div(self, rhs: Self) -> (Self, bool) {
                    self.overflowing_div(rhs)
                }

                fn overflowing_neg(self) -> (Self, bool) {
                    self.overflowing_neg()
                }

                fn overflowing_rem(self, rhs: Self) -> (Self, bool) {
                    self.overflowing_rem(rhs)
                }

                fn overflowing_pow(self, exp: Self::BitWidthType) -> (Self, bool) {
                    self.overflowing_pow(exp)
                }
            }

            impl CheckedOps for $t {
                fn checked_add(self, rhs: Self) -> Option<Self> {
                    self.checked_add(rhs)
                }

                fn checked_sub(self, rhs: Self) -> Option<Self> {
                    self.checked_sub(rhs)
                }

                fn checked_mul(self, rhs: Self) -> Option<Self> {
                    self.checked_mul(rhs)
                }

                fn checked_div(self, rhs: Self) -> Option<Self> {
                    self.checked_div(rhs)
                }

                fn checked_neg(self) -> Option<Self> {
                    self.checked_neg()
                }

                fn checked_rem(self, rhs: Self) -> Option<Self> {
                    self.checked_rem(rhs)
                }

                fn checked_pow(self, exp: Self::BitWidthType) -> Option<Self> {
                    self.checked_pow(exp)
                }
            }
        )*
    };
}

impl_ops!(i8, i16, i32, i64, i128, isize);
impl_ops!(u8, u16, u32, u64, u128, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Saturating<T>(pub T);

impl<T: SaturatingOps> Saturating<T> {
    pub fn pow(self, exp: T::BitWidthType) -> Self {
        Self(self.0.saturating_pow(exp))
    }
}

impl<T> Add for Saturating<T>
where
    T: SaturatingOps,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Saturating(self.0.saturating_add(rhs.0))
    }
}

impl<T> Add<T> for Saturating<T>
where
    T: SaturatingOps,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Saturating(self.0.saturating_add(rhs))
    }
}

impl<T> AddAssign for Saturating<T>
where
    T: SaturatingOps + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        let tmp = self.clone().add(rhs);
        *self = tmp;
    }
}

impl<T> AddAssign<T> for Saturating<T>
where
    T: SaturatingOps + Clone,
{
    fn add_assign(&mut self, rhs: T) {
        let tmp = self.clone().add(rhs);
        *self = tmp;
    }
}

impl<T> Sub for Saturating<T>
where
    T: SaturatingOps,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Saturating(self.0.saturating_sub(rhs.0))
    }
}

impl<T> Sub<T> for Saturating<T>
where
    T: SaturatingOps,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Saturating(self.0.saturating_sub(rhs))
    }
}

impl<T> SubAssign for Saturating<T>
where
    T: SaturatingOps + Clone,
{
    fn sub_assign(&mut self, rhs: Self) {
        let tmp = self.clone().sub(rhs);
        *self = tmp;
    }
}

impl<T> SubAssign<T> for Saturating<T>
where
    T: SaturatingOps + Clone,
{
    fn sub_assign(&mut self, rhs: T) {
        let tmp = self.clone().sub(rhs);
        *self = tmp;
    }
}

impl<T> Mul for Saturating<T>
where
    T: SaturatingOps,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Saturating(self.0.saturating_mul(rhs.0))
    }
}

impl<T> Mul<T> for Saturating<T>
where
    T: SaturatingOps,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Saturating(self.0.saturating_mul(rhs))
    }
}

impl<T> MulAssign for Saturating<T>
where
    T: SaturatingOps + Clone,
{
    fn mul_assign(&mut self, rhs: Self) {
        let tmp = self.clone().mul(rhs);
        *self = tmp;
    }
}

impl<T> MulAssign<T> for Saturating<T>
where
    T: SaturatingOps + Clone,
{
    fn mul_assign(&mut self, rhs: T) {
        let tmp = self.clone().mul(rhs);
        *self = tmp;
    }
}

impl<T> Div for Saturating<T>
where
    T: SaturatingOps,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Saturating(self.0.saturating_div(rhs.0))
    }
}

impl<T> Div<T> for Saturating<T>
where
    T: SaturatingOps,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Saturating(self.0.saturating_div(rhs))
    }
}

impl<T> DivAssign for Saturating<T>
where
    T: SaturatingOps + Clone,
{
    fn div_assign(&mut self, rhs: Self) {
        let tmp = self.clone().div(rhs);
        *self = tmp;
    }
}

impl<T> DivAssign<T> for Saturating<T>
where
    T: SaturatingOps + Clone,
{
    fn div_assign(&mut self, rhs: T) {
        let tmp = self.clone().div(rhs);
        *self = tmp;
    }
}

impl<T> Rem for Saturating<T>
where
    T: SaturatingOps,
{
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Saturating(self.0.rem(rhs.0))
    }
}

impl<T> Rem<T> for Saturating<T>
where
    T: SaturatingOps,
{
    type Output = Self;

    fn rem(self, rhs: T) -> Self::Output {
        Saturating(self.0.rem(rhs))
    }
}

impl<T> RemAssign for Saturating<T>
where
    T: SaturatingOps,
{
    fn rem_assign(&mut self, rhs: Self) {
        self.0.rem_assign(rhs.0);
    }
}

impl<T> RemAssign<T> for Saturating<T>
where
    T: SaturatingOps,
{
    fn rem_assign(&mut self, rhs: T) {
        self.0.rem_assign(rhs);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Wrapping<T>(pub T);

impl<T: WrappingOps> Wrapping<T> {
    pub fn pow(self, exp: T::BitWidthType) -> Self {
        Self(self.0.wrapping_pow(exp))
    }
}

impl<T> Add for Wrapping<T>
where
    T: WrappingOps,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Wrapping(self.0.wrapping_add(rhs.0))
    }
}

impl<T> Add<T> for Wrapping<T>
where
    T: WrappingOps,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Wrapping(self.0.wrapping_add(rhs))
    }
}

impl<T> AddAssign for Wrapping<T>
where
    T: WrappingOps + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        let tmp = self.clone().add(rhs);
        *self = tmp;
    }
}

impl<T> AddAssign<T> for Wrapping<T>
where
    T: WrappingOps + Clone,
{
    fn add_assign(&mut self, rhs: T) {
        let tmp = self.clone().add(rhs);
        *self = tmp;
    }
}

impl<T> Sub for Wrapping<T>
where
    T: WrappingOps,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Wrapping(self.0.wrapping_sub(rhs.0))
    }
}

impl<T> Sub<T> for Wrapping<T>
where
    T: WrappingOps,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Wrapping(self.0.wrapping_sub(rhs))
    }
}

impl<T> SubAssign for Wrapping<T>
where
    T: WrappingOps + Clone,
{
    fn sub_assign(&mut self, rhs: Self) {
        let tmp = self.clone().sub(rhs);
        *self = tmp;
    }
}

impl<T> SubAssign<T> for Wrapping<T>
where
    T: WrappingOps + Clone,
{
    fn sub_assign(&mut self, rhs: T) {
        let tmp = self.clone().sub(rhs);
        *self = tmp;
    }
}

impl<T> Mul for Wrapping<T>
where
    T: WrappingOps,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Wrapping(self.0.wrapping_mul(rhs.0))
    }
}

impl<T> Mul<T> for Wrapping<T>
where
    T: WrappingOps,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Wrapping(self.0.wrapping_mul(rhs))
    }
}

impl<T> MulAssign for Wrapping<T>
where
    T: WrappingOps + Clone,
{
    fn mul_assign(&mut self, rhs: Self) {
        let tmp = self.clone().mul(rhs);
        *self = tmp;
    }
}

impl<T> MulAssign<T> for Wrapping<T>
where
    T: WrappingOps + Clone,
{
    fn mul_assign(&mut self, rhs: T) {
        let tmp = self.clone().mul(rhs);
        *self = tmp;
    }
}

impl<T> Div for Wrapping<T>
where
    T: WrappingOps,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Wrapping(self.0.wrapping_div(rhs.0))
    }
}

impl<T> Div<T> for Wrapping<T>
where
    T: WrappingOps,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Wrapping(self.0.wrapping_div(rhs))
    }
}

impl<T> DivAssign for Wrapping<T>
where
    T: WrappingOps + Clone,
{
    fn div_assign(&mut self, rhs: Self) {
        let tmp = self.clone().div(rhs);
        *self = tmp;
    }
}

impl<T> DivAssign<T> for Wrapping<T>
where
    T: WrappingOps + Clone,
{
    fn div_assign(&mut self, rhs: T) {
        let tmp = self.clone().div(rhs);
        *self = tmp;
    }
}

impl<T> Rem for Wrapping<T>
where
    T: WrappingOps,
{
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Wrapping(self.0.wrapping_rem(rhs.0))
    }
}

impl<T> Rem<T> for Wrapping<T>
where
    T: WrappingOps,
{
    type Output = Self;

    fn rem(self, rhs: T) -> Self::Output {
        Wrapping(self.0.wrapping_rem(rhs))
    }
}

impl<T> RemAssign for Wrapping<T>
where
    T: WrappingOps + Clone,
{
    fn rem_assign(&mut self, rhs: Self) {
        let tmp = self.clone().rem(rhs);
        *self = tmp;
    }
}

impl<T> RemAssign<T> for Wrapping<T>
where
    T: WrappingOps + Clone,
{
    fn rem_assign(&mut self, rhs: T) {
        let tmp = self.clone().rem(rhs);
        *self = tmp;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn check() {}
}
