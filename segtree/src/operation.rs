use std::{marker::PhantomData, ops};

pub trait Operator {
    type Query;
    const IDENT: Self::Query;
    fn op(&self, a: &Self::Query, b: &Self::Query) -> Self::Query;
    fn op_assign_left(&self, a: &mut Self::Query, b: &Self::Query) {
        *a = self.op(a, b);
    }
    fn op_assign_right(&self, a: &Self::Query, b: &mut Self::Query) {
        *b = self.op(a, b);
    }
}

/// An operator is idempotent if `op(a, a) = a` for all `a`.
pub trait Idempotent: Operator {}

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

macro_rules! impl_auto_trait_for_marker {
    ($t:ident, $($u:ty),*) => {
        $(
            impl<$t> Clone for $u {
                fn clone(&self) -> Self {
                    *self
                }
            }

            impl<$t> Copy for $u {}

            impl<$t> Default for $u {
                fn default() -> Self {
                    Self(PhantomData)
                }
            }

            impl<$t> PartialEq for $u {
                fn eq(&self, _: &Self) -> bool {
                    true
                }
            }

            impl<$t> Eq for $u {}

            impl<$t> ::core::fmt::Debug for $u {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    write!(f, "{}", ::core::any::type_name::<$u>())
                }
            }
        )*
    }
}

// #[derive(Debug)]
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

pub const fn max<T>() -> Max<T> {
    Max(PhantomData)
}
pub const fn min<T>() -> Min<T> {
    Min(PhantomData)
}

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

impl<T> Idempotent for Max<T> where Max<T>: Operator {}

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

impl<T> Idempotent for Min<T> where Min<T>: Operator {}

impl<'a, T: Operator> Operator for &'a T {
    type Query = T::Query;
    const IDENT: Self::Query = T::IDENT;
    fn op(&self, a: &Self::Query, b: &Self::Query) -> Self::Query {
        T::op(self, a, b)
    }
}

impl_auto_trait_for_marker!(T, Add<T>, Mul<T>, Max<T>, Min<T>);
