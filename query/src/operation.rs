use std::{
    marker::PhantomData,
    ops::{self, Add, AddAssign, BitXorAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::{AbelianGroup, Group, Monoid, Ring};

pub trait Number: Sized {
    const ZERO: Self;
    const ONE: Self;
}

pub trait HasAddIdent {
    const ADD_IDENT: Self;
}

impl<T: Number> HasAddIdent for T {
    const ADD_IDENT: Self = T::ZERO;
}

pub trait HasMulIdent {
    const MUL_IDENT: Self;
}

impl<T: Number> HasMulIdent for T {
    const MUL_IDENT: Self = T::ONE;
}

pub trait HasBitXorIdent {
    const BIT_XOR_IDENT: Self;
}

impl<T: Number> HasBitXorIdent for T {
    const BIT_XOR_IDENT: Self = T::ZERO;
}

macro_rules! impl_number {
    ($t:ty, $zero:expr, $one:expr) => {
        impl Number for $t {
            const ZERO: Self = $zero;
            const ONE: Self = $one;
        }
    };
}

impl_number!(i8, 0, 1);
impl_number!(i16, 0, 1);
impl_number!(i32, 0, 1);
impl_number!(i64, 0, 1);
impl_number!(i128, 0, 1);
impl_number!(u8, 0, 1);
impl_number!(u16, 0, 1);
impl_number!(u32, 0, 1);
impl_number!(u64, 0, 1);
impl_number!(u128, 0, 1);

#[derive(Clone, Copy, Debug)]
pub struct Sum<T>(PhantomData<fn() -> T>);

impl<T> Default for Sum<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> Monoid for Sum<T>
where
    T: Clone + Add<Output = T> + AddAssign + HasAddIdent,
{
    type Element = T;
    const IDENTITY: Self::Element = T::ADD_IDENT;

    fn op(&self, a: &Self::Element, b: &Self::Element) -> Self::Element {
        a.clone() + b.clone()
    }

    fn op_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        *a += b.clone();
    }
}

impl<T> Group for Sum<T>
where
    Sum<T>: Monoid<Element = T>,
    T: Clone + Neg<Output = T> + Sub<Output = T> + SubAssign,
{
    fn inv(&self, a: &Self::Element) -> Self::Element {
        Self::IDENTITY.clone() - a.clone()
    }

    fn op_inv(&self, a: &Self::Element, b: &Self::Element) -> Self::Element {
        b.clone() - a.clone()
    }

    fn op_inv_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        *a -= b.clone();
    }
}

impl<T> AbelianGroup for Sum<T>
where
    Sum<T>: Group<Element = T>,
    T: Number,
{}

#[derive(Clone, Copy, Debug)]
pub struct Product<T>(PhantomData<fn() -> T>);

impl<T> Default for Product<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> Monoid for Product<T>
where
    T: Clone + Mul<Output = T> + MulAssign + HasMulIdent,
{
    type Element = T;
    const IDENTITY: Self::Element = T::MUL_IDENT;
    fn op(&self, a: &Self::Element, b: &Self::Element) -> Self::Element {
        a.clone() * b.clone()
    }

    fn op_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        *a *= b.clone();
    }
}

pub struct BitXor<T>(PhantomData<fn() -> T>);

impl<T> Default for BitXor<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> Monoid for BitXor<T>
where
    T: Clone + ops::BitXor<Output = T> + BitXorAssign + HasBitXorIdent,
{
    type Element = T;
    const IDENTITY: Self::Element = T::BIT_XOR_IDENT;

    fn op(&self, a: &Self::Element, b: &Self::Element) -> Self::Element {
        a.clone() ^ b.clone()
    }

    fn op_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        *a ^= b.clone();
    }
}

impl<T> Group for BitXor<T>
where
    BitXor<T>: Monoid<Element = T>,
    T: Clone,
{
    fn inv(&self, a: &Self::Element) -> Self::Element {
        a.clone()
    }

    fn op_inv(&self, a: &Self::Element, b: &Self::Element) -> Self::Element {
        self.op(a, b)
    }

    fn op_inv_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        self.op_assign(a, b);
    }
}

impl<T> AbelianGroup for BitXor<T>
where
    BitXor<T>: Group<Element = T>,
    T: Number,
{}

impl<T> Ring for (Sum<T>, Product<T>)
where
    Sum<T>: AbelianGroup<Element = T>,
    Product<T>: Monoid<Element = T>,
    T: Clone,
{
    type Element = T;
    const ADD_IDENTITY: Self::Element = Sum::<T>::IDENTITY;
    const MUL_IDENTITY: Self::Element = Product::<T>::IDENTITY;
    fn add(&self, a: &Self::Element, b: &Self::Element) -> Self::Element {
        self.0.op(a, b)
    }
    fn mul(&self, a: &Self::Element, b: &Self::Element) -> Self::Element {
        self.1.op(a, b)
    }
    fn neg(&self, a: &Self::Element) -> Self::Element {
        self.0.inv(a)
    }
    fn sub(&self, a: &Self::Element, b: &Self::Element) -> Self::Element {
        self.0.op_inv(a, b)
    }

    fn add_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        self.0.op_assign(a, b);
    }
    fn mul_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        self.1.op_assign(a, b);
    }
    fn sub_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        self.0.op_inv_assign(a, b);
    }
}
