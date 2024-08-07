use std::{ops, marker::PhantomData};

pub trait Operator {
    type Query;
    const IDENT: Self::Query;
    fn op(&self, a: &Self::Query, b: &Self::Query) -> Self::Query;
}

trait HasZero {
    const ZERO: Self;
}

trait HasOne {
    const ONE: Self;
}

trait HasMax {
    const MAX: Self;
}

trait HasMin {
    const MIN: Self;
}

macro_rules! impl_trait_integer {
    ($($t:ty),*) => {
        $(
            impl HasZero for $t {
                const ZERO: Self = 0;
            }
            impl HasOne for $t {
                const ONE: Self = 1;
            }
            impl HasMax for $t {
                const MAX: Self = <$t>::MAX;
            }
            impl HasMin for $t {
                const MIN: Self = <$t>::MIN;
            }
        )*
    };
}

impl_trait_integer!(i8, i16, i32, i64, i128, isize);
impl_trait_integer!(u8, u16, u32, u64, u128, usize);

pub struct Add<T>(PhantomData<fn() -> T>);
pub struct Mul<T>(PhantomData<fn() -> T>);

impl<T> Operator for Add<T>
where
    T: ops::Add<Output = T> + Clone + HasZero,
{
    type Query = T;
    const IDENT: Self::Query = T::ZERO;
    fn op(&self, a: &Self::Query, b: &Self::Query) -> Self::Query {
        a.clone() + b.clone()
    }
}

impl<T> Operator for Mul<T>
where
    T: ops::Mul<Output = T> + Clone + HasOne,
{
    type Query = T;
    const IDENT: Self::Query = T::ONE;
    fn op(&self, a: &Self::Query, b: &Self::Query) -> Self::Query {
        a.clone() * b.clone()
    }
}

pub struct Max<T>(PhantomData<fn() -> T>);
pub struct Min<T>(PhantomData<fn() -> T>);

impl<T> Operator for Max<T>
where
    T: Ord + Clone + HasMin,
{
    type Query = T;
    const IDENT: Self::Query = T::MIN;
    fn op(&self, a: &Self::Query, b: &Self::Query) -> Self::Query {
        if a > b {
            a.clone()
        } else {
            b.clone()
        }
    }
}

impl<T> Operator for Min<T>
where
    T: Ord + Clone + HasMax,
{
    type Query = T;
    const IDENT: Self::Query = T::MAX;
    fn op(&self, a: &Self::Query, b: &Self::Query) -> Self::Query {
        if a < b {
            a.clone()
        } else {
            b.clone()
        }
    }
}
