use std::{marker::PhantomData, ops};

pub trait Operator {
    type Query;
    fn ident(&self) -> Self::Query;
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

macro_rules! impl_auto_trait_simple {
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

macro_rules! impl_auto_trait {
    ($($t:ident),*; $u:ty) => {
        impl<$($t),*> Clone for $u {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<$($t),*> Copy for $u {}

        impl<$($t),*> Default for $u {
            fn default() -> Self {
                Self(PhantomData)
            }
        }

        impl<$($t),*> PartialEq for $u {
            fn eq(&self, _: &Self) -> bool {
                true
            }
        }

        impl<$($t),*> Eq for $u {}

        impl<$($t),*> ::core::fmt::Debug for $u {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                write!(f, "{}", ::core::any::type_name::<$u>())
            }
        }
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
    fn ident(&self) -> Self::Query {
        T::ZERO
    }
    fn op(&self, a: &Self::Query, b: &Self::Query) -> Self::Query {
        a.clone() + b.clone()
    }
}

impl<T> Operator for Mul<T>
where
    T: ops::Mul<Output = T> + Clone + HasOne,
{
    type Query = T;
    fn ident(&self) -> Self::Query {
        T::ONE
    }
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
    fn ident(&self) -> Self::Query {
        T::MIN
    }
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
    fn ident(&self) -> Self::Query {
        T::MAX
    }
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
    fn ident(&self) -> Self::Query {
        T::ident(self)
    }
    fn op(&self, a: &Self::Query, b: &Self::Query) -> Self::Query {
        T::op(self, a, b)
    }
}

impl_auto_trait_simple!(T, Add<T>, Mul<T>, Max<T>, Min<T>);

pub trait Map {
    type OP: Operator;
    type Elem;
    const IDENT: Self::Elem;

    fn apply(
        &self,
        q: &<Self::OP as Operator>::Query,
        m: &Self::Elem,
    ) -> <Self::OP as Operator>::Query;
    fn apply_assign(&self, q: &mut <Self::OP as Operator>::Query, m: &Self::Elem) {
        let new_q = self.apply(q, m);
        *q = new_q;
    }
    fn composite(&self, a: &Self::Elem, b: &Self::Elem) -> Self::Elem;
    fn composite_assign(&self, a: &mut Self::Elem, b: &Self::Elem) {
        let new_a = self.composite(a, b);
        *a = new_a;
    }
}

impl<'a, T: Map> Map for &'a T {
    type OP = T::OP;
    type Elem = T::Elem;
    const IDENT: Self::Elem = T::IDENT;
    fn apply(
        &self,
        q: &<Self::OP as Operator>::Query,
        m: &Self::Elem,
    ) -> <Self::OP as Operator>::Query {
        T::apply(self, q, m)
    }
    fn apply_assign(&self, q: &mut <Self::OP as Operator>::Query, m: &Self::Elem) {
        T::apply_assign(self, q, m)
    }
    fn composite(&self, a: &Self::Elem, b: &Self::Elem) -> Self::Elem {
        T::composite(self, a, b)
    }
    fn composite_assign(&self, a: &mut Self::Elem, b: &Self::Elem) {
        T::composite_assign(self, a, b)
    }
}

pub struct Update<OP>(PhantomData<fn() -> OP>);

pub fn update<OP>() -> Update<OP> {
    Update(PhantomData)
}

impl<OP> Map for Update<OP>
where
    OP: Idempotent,
    <OP as Operator>::Query: Clone,
{
    type OP = OP;
    type Elem = Option<<OP as Operator>::Query>;
    const IDENT: Self::Elem = None;
    fn apply(
        &self,
        q: &<Self::OP as Operator>::Query,
        m: &Self::Elem,
    ) -> <Self::OP as Operator>::Query {
        match m {
            Some(m) => m.clone(),
            None => q.clone(),
        }
    }

    fn apply_assign(&self, q: &mut <Self::OP as Operator>::Query, m: &Self::Elem) {
        if let Some(m) = m {
            q.clone_from(m);
        }
    }

    fn composite(&self, a: &Self::Elem, b: &Self::Elem) -> Self::Elem {
        match b {
            Some(b) => Some(b.clone()),
            None => a.clone(),
        }
    }

    fn composite_assign(&self, a: &mut Self::Elem, b: &Self::Elem) {
        if b.is_some() {
            a.clone_from(b);
        }
    }
}

pub struct RangeAdd<F, OP>(PhantomData<fn() -> (F, OP)>);

pub fn range_add<F, OP>() -> RangeAdd<F, OP> {
    RangeAdd(PhantomData)
}

impl<T, F> Map for RangeAdd<F, Min<T>>
where
    Min<T>: Operator<Query = T>,
    T: ops::Add<F, Output = T> + Clone,
    F: HasZero + ops::Add<Output = F> + Clone,
{
    type OP = Min<T>;
    type Elem = F;
    const IDENT: Self::Elem = F::ZERO;

    fn apply(
        &self,
        q: &<Self::OP as Operator>::Query,
        m: &Self::Elem,
    ) -> <Self::OP as Operator>::Query {
        q.clone() + m.clone()
    }

    fn composite(&self, a: &Self::Elem, b: &Self::Elem) -> Self::Elem {
        a.clone() + b.clone()
    }
}

impl_auto_trait_simple!(OP, Update<OP>);
impl_auto_trait!(F, OP; RangeAdd<F, OP>);
