use std::{marker::PhantomData, ops};

pub trait Operator {
    type ValT;
    type QValT;
    const IDENT: Self::QValT;
    fn op(&self, a: &Self::QValT, b: &Self::QValT) -> Self::QValT;
    fn val_to_query(&self, val: &Self::ValT) -> Self::QValT;
}

pub trait Commutative: Operator {}

pub struct Noop<T>(PhantomData<fn() -> T>);

impl<T> Operator for Noop<T> {
    type ValT = T;
    type QValT = ();
    const IDENT: Self::QValT = ();
    fn op(&self, _: &Self::QValT, _: &Self::QValT) -> Self::QValT {}
    fn val_to_query(&self, _: &Self::ValT) -> Self::QValT {
        
    }
}

pub struct Add<T>(PhantomData<fn() -> T>);

impl<T> Add<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for Add<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait HasZero {
    const ZERO: Self;
}

macro_rules! impl_has_zero {
    ($zero:expr, $($t:ty),*) => {
        $(
            impl HasZero for $t {
                const ZERO: Self = $zero;
            }
        )*
    };
}

impl_has_zero!(0, i8, i16, i32, i64, i128);
impl_has_zero!(0, u8, u16, u32, u64, u128);
impl_has_zero!(0., f32, f64);

impl<T: ops::Add<Output = T> + HasZero + Clone> Operator for Add<T> {
    type ValT = T;
    type QValT = T;
    const IDENT: Self::QValT = T::ZERO;
    fn op(&self, a: &Self::QValT, b: &Self::QValT) -> Self::QValT {
        a.clone() + b.clone()
    }
    fn val_to_query(&self, val: &Self::ValT) -> Self::QValT {
        val.clone()
    }
}

impl<T> Commutative for Add<T> where Add<T>: Operator {}
