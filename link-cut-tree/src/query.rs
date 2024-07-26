use std::marker::PhantomData;

pub trait Query {
    type ValT;
    type QValT;
    const IDENT: Self::QValT;
    fn op_left(&self, a: &Self::QValT, b: &Self::ValT) -> Self::QValT;
    fn op_right(&self, a: &Self::ValT, b: &Self::QValT) -> Self::QValT;
    fn op(&self, a: &Self::QValT, b: &Self::QValT) -> Self::QValT;
    fn from_val(&self, val: &Self::ValT) -> Self::QValT {
        self.op_left(&Self::IDENT, val)
    }
}

pub trait Commutative: Query {}

pub struct Noop<T>(PhantomData<fn() -> T>);

impl<T> Query for Noop<T> {
    type ValT = T;
    type QValT = ();
    const IDENT: Self::QValT = ();
    fn op_left(&self, _: &Self::QValT, _: &Self::ValT) -> Self::QValT {}
    fn op_right(&self, _: &Self::ValT, _: &Self::QValT) -> Self::QValT {}
    fn op(&self, _: &Self::QValT, _: &Self::QValT) -> Self::QValT {}
}
