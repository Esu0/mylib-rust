use std::{marker::PhantomData, ops};

pub trait Query {
    type ValT;
    type QValT;
    const IDENT: Self::QValT;
    fn op_left(&self, a: &Self::QValT, b: &Self::ValT) -> Self::QValT;
    fn op_right(&self, a: &Self::ValT, b: &Self::QValT) -> Self::QValT;
    fn op(&self, a: &Self::QValT, b: &Self::QValT) -> Self::QValT;
    fn val_to_query(&self, val: &Self::ValT) -> Self::QValT {
        self.op_left(&Self::IDENT, val)
    }
}

pub trait Commutative: Query {}

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

impl<T: ops::Add<Output = T> + HasZero + Clone> Query for Add<T> {
    type ValT = T;
    type QValT = T;
    const IDENT: Self::QValT = T::ZERO;
    fn op_left(&self, a: &Self::QValT, b: &Self::ValT) -> Self::QValT {
        a.clone() + b.clone()
    }
    fn op_right(&self, a: &Self::ValT, b: &Self::QValT) -> Self::QValT {
        a.clone() + b.clone()
    }
    fn op(&self, a: &Self::QValT, b: &Self::QValT) -> Self::QValT {
        a.clone() + b.clone()
    }
}

impl<T> Commutative for Add<T> where Add<T>: Query {}

pub trait PathOperator {
    type V;
    type E;
    type Path;

    fn val_to_path(&self, val: &Self::V) -> Self::Path;
    fn connect_path(&self, p1: &Self::Path, edge: &Self::E, p2: &Self::Path) -> Self::Path;
    fn to_reversable(self) -> Reversable<Self>
    where
        Self: Sized,
    {
        Reversable(self)
    }
}

pub trait ReversablePathOperator: PathOperator {
    fn reverse_path(&self, path: &mut Self::Path);
}

pub struct Reversable<O>(O);

impl<P: Clone, O: PathOperator<Path = P>> PathOperator for Reversable<O> {
    type V = O::V;
    type E = O::E;
    type Path = (P, P);

    fn val_to_path(&self, val: &Self::V) -> Self::Path {
        let p = self.0.val_to_path(val);
        (p.clone(), p)
    }

    fn connect_path(&self, p1: &Self::Path, edge: &Self::E, p2: &Self::Path) -> Self::Path {
        let p = self.0.connect_path(&p1.0, edge, &p2.0);
        let p_reverse = self.0.connect_path(&p2.1, edge, &p1.1);
        (p, p_reverse)
    }
}

impl<P: Clone, O: PathOperator<Path = P>> ReversablePathOperator for Reversable<O> {
    fn reverse_path(&self, path: &mut Self::Path) {
        std::mem::swap(&mut path.0, &mut path.1);
    }
}

impl<'a, O: PathOperator> PathOperator for &'a O {
    type V = O::V;
    type E = O::E;
    type Path = O::Path;

    fn val_to_path(&self, val: &Self::V) -> Self::Path {
        O::val_to_path(self, val)
    }

    fn connect_path(&self, p1: &Self::Path, edge: &Self::E, p2: &Self::Path) -> Self::Path {
        O::connect_path(self, p1, edge, p2)
    }
}

impl<'a, O: ReversablePathOperator> ReversablePathOperator for &'a O {
    fn reverse_path(&self, path: &mut Self::Path) {
        O::reverse_path(self, path)
    }
}

pub fn noop<V, E>() -> Noop<V, E> {
    Noop(PhantomData)
}

pub struct Noop<V, E>(PhantomData<fn() -> (V, E)>);

impl<V, E> Default for Noop<V, E> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<V, E> Clone for Noop<V, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V, E> Copy for Noop<V, E> {}

impl<V, E> PathOperator for Noop<V, E> {
    type V = V;
    type E = E;
    type Path = ();
    fn val_to_path(&self, _: &Self::V) -> Self::Path {}
    fn connect_path(&self, _: &Self::Path, _: &Self::E, _: &Self::Path) -> Self::Path {}
}

impl<V, E> ReversablePathOperator for Noop<V, E> {
    fn reverse_path(&self, _: &mut Self::Path) {}
}
