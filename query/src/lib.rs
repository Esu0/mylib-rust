pub mod impls;
pub use impls::Noop;

use std::{hash::Hash, marker::PhantomData, ops};

pub trait BinaryOperation {
    type ArgType1;
    type ArgType2;
    type OutputType;

    fn op(&self, a: &Self::ArgType1, b: &Self::ArgType2) -> Self::OutputType;
}

impl<F: BinaryOperation> BinaryOperation for &F {
    type ArgType1 = F::ArgType1;
    type ArgType2 = F::ArgType2;
    type OutputType = F::OutputType;

    fn op(&self, a: &Self::ArgType1, b: &Self::ArgType2) -> Self::OutputType {
        F::op(self, a, b)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct BinaryOp<A, B, C, F> {
    op: F,
    _marker: PhantomData<fn(&A, &B) -> C>,
}

impl<A, B, C, F: Clone> Clone for BinaryOp<A, B, C, F> {
    fn clone(&self) -> Self {
        Self {
            op: self.op.clone(),
            _marker: PhantomData,
        }
    }
}

impl<A, B, C, F: Copy> Copy for BinaryOp<A, B, C, F> {}

impl<A, B, C, F: Default> Default for BinaryOp<A, B, C, F> {
    fn default() -> Self {
        Self {
            op: F::default(),
            _marker: PhantomData,
        }
    }
}

impl<A, B, C, F: PartialEq> PartialEq for BinaryOp<A, B, C, F> {
    fn eq(&self, other: &Self) -> bool {
        self.op == other.op
    }
}
impl<A, B, C, F: Eq> Eq for BinaryOp<A, B, C, F> {}

impl<A, B, C, F: PartialOrd> PartialOrd for BinaryOp<A, B, C, F> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.op.partial_cmp(&other.op)
    }
}

impl<A, B, C, F: Ord> Ord for BinaryOp<A, B, C, F> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.op.cmp(&other.op)
    }
}

impl<A, B, C, F: Hash> Hash for BinaryOp<A, B, C, F> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.op.hash(state);
    }

    fn hash_slice<H: std::hash::Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized, {
        let data = unsafe {
            &*(data as *const [Self] as *const [F])
        };
        data.hash(state);
    }
}

impl<A, B, C, F: Fn(&A, &B) -> C> BinaryOp<A, B, C, F> {
    pub fn new(op: F) -> Self {
        Self {
            op,
            _marker: PhantomData,
        }
    }
}

impl<A, B, C> BinaryOp<A, B, C, fn(&A, &B) -> C> {
    pub fn from_fn(op: fn(&A, &B) -> C) -> Self {
        Self::new(op)
    }
}

impl<T: Clone, F: Fn(&T, &T) -> T> BinaryOp<T, T, T, F> {
    pub fn into_monoid(self, identity: T) -> MonoidOp<T, F> {
        MonoidOp::new(identity, self.op)
    }
}

impl<A, B, C, F: Fn(&A, &B) -> C> BinaryOperation for BinaryOp<A, B, C, F> {
    type ArgType1 = A;
    type ArgType2 = B;
    type OutputType = C;

    fn op(&self, a: &A, b: &B) -> C {
        (self.op)(a, b)
    }
}

/// モノイドの性質を満たす二項演算
///
/// # モノイドであるための条件
/// * 任意の元`a,b,c`に対して、`(a * b) * c = a * (b * c)`が成り立つ。(結合則)
/// * ある元`e`が存在して、任意の元`a`に対して、`e * a = a * e = a`が成り立つ。(単位元の存在)
pub trait Monoid: BinaryOperation<ArgType1 = Self::Element, ArgType2 = Self::Element, OutputType = Self::Element> {
    type Element: Clone;
    fn identity(&self) -> Self::Element;
    fn op_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        *a = self.op(a, b);
    }

    fn assume_idempotent(self) -> IdempotentOp<Self>
    where
        Self: Sized,
    {
        IdempotentOp(self)
    }

    fn assume_commutative(self) -> CommutativeOp<Self>
    where
        Self: Sized,
    {
        CommutativeOp(self)
    }
}

impl<M: Monoid> Monoid for &M {
    type Element = M::Element;

    fn identity(&self) -> Self::Element {
        (*self).identity()
    }

    fn op_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        (*self).op_assign(a, b);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonoidOp<T, F> {
    identity: T,
    op: F,
}

impl<T: Clone, F: Fn(&T, &T) -> T> MonoidOp<T, F> {
    pub fn new(identity: T, op: F) -> Self {
        Self { identity, op }
    }
}

impl<T: Clone, F: Fn(&T, &T) -> T> BinaryOperation for MonoidOp<T, F> {
    type ArgType1 = T;
    type ArgType2 = T;
    type OutputType = T;

    fn op(&self, a: &T, b: &T) -> T {
        (self.op)(a, b)
    }
}

impl<T: Clone, F: Fn(&T, &T) -> T> Monoid for MonoidOp<T, F> {
    type Element = T;

    fn identity(&self) -> T {
        self.identity.clone()
    }
}

/// 群の性質を満たす二項演算
///
/// # 群であるための条件
/// * 二項演算はモノイドである。
/// * 任意の元`a`に対して、ある元`b`が存在して、`a * b = b * a = e`が成り立つ。(逆元の存在)
pub trait Group: Monoid {
    /// `self.op(&a, &self.inv(&a)) == Self::IDENTITY`を満たす必要がある。
    fn inv(&self, a: &Self::Element) -> Self::Element;

    /// `self.op(&a, &x) == b`を満たす`x`を返す。
    fn op_inv(&self, a: &Self::Element, b: &Self::Element) -> Self::Element {
        let b_inv = self.inv(b);
        self.op(a, &b_inv)
    }

    fn op_inv_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        *a = self.op_inv(a, b);
    }
}

impl<G: Group> Group for &G {
    fn inv(&self, a: &Self::Element) -> Self::Element {
        (*self).inv(a)
    }

    fn op_inv(&self, a: &Self::Element, b: &Self::Element) -> Self::Element {
        (*self).op_inv(a, b)
    }

    fn op_inv_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        (*self).op_inv_assign(a, b);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupOp<M, IF> {
    monoid: M,
    inv: IF,
}

impl<M: Monoid, IF: Fn(&M::Element) -> M::Element> GroupOp<M, IF> {
    pub fn new(monoid: M, inv: IF) -> Self {
        Self { monoid, inv }
    }
}

impl<M: Monoid, IF: Fn(&M::Element) -> M::Element> BinaryOperation for GroupOp<M, IF> {
    type ArgType1 = M::Element;
    type ArgType2 = M::Element;
    type OutputType = M::Element;

    fn op(&self, a: &M::Element, b: &M::Element) -> M::Element {
        self.monoid.op(a, b)
    }
}

impl<M: Monoid, IF: Fn(&M::Element) -> M::Element> Monoid for GroupOp<M, IF> {
    type Element = M::Element;

    fn identity(&self) -> M::Element {
        self.monoid.identity()
    }

    fn op_assign(&self, a: &mut M::Element, b: &M::Element) {
        self.monoid.op_assign(a, b);
    }
}

impl<M: Monoid, IF: Fn(&M::Element) -> M::Element> Group for GroupOp<M, IF> {
    fn inv(&self, a: &M::Element) -> M::Element {
        (self.inv)(a)
    }
}

/// アーベル群の性質を満たす二項演算
///
/// # アーベル群であるための条件
/// * 群である。
/// * 任意の元`a,b`に対して、`a * b = b * a`が成り立つ。(交換則)
pub trait AbelianGroup: Group + Commutative {}

impl<G: Group + Commutative> AbelianGroup for G {}

/// 冪等性を持つ二項演算
pub trait Idempotent: Monoid {}

impl<M: Idempotent> Idempotent for &M {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct IdempotentOp<M>(M);

impl<M> ops::Deref for IdempotentOp<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<M> ops::DerefMut for IdempotentOp<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<M> IdempotentOp<M> {
    pub fn into_inner(self) -> M {
        self.0
    }
}

impl<M: Monoid> BinaryOperation for IdempotentOp<M> {
    type ArgType1 = M::Element;
    type ArgType2 = M::Element;
    type OutputType = M::Element;

    fn op(&self, a: &M::Element, b: &M::Element) -> M::Element {
        self.0.op(a, b)
    }
}

impl<M: Monoid> Monoid for IdempotentOp<M> {
    type Element = M::Element;

    fn identity(&self) -> M::Element {
        self.0.identity()
    }

    fn op_assign(&self, a: &mut M::Element, b: &M::Element) {
        self.0.op_assign(a, b);
    }
}

impl<M: Monoid> Idempotent for IdempotentOp<M> {}

impl<M: Group> Group for IdempotentOp<M> {
    fn inv(&self, a: &M::Element) -> M::Element {
        self.0.inv(a)
    }

    fn op_inv(&self, a: &M::Element, b: &M::Element) -> M::Element {
        self.0.op_inv(a, b)
    }

    fn op_inv_assign(&self, a: &mut M::Element, b: &M::Element) {
        self.0.op_inv_assign(a, b);
    }
}

impl<M: Commutative> Commutative for IdempotentOp<M> {}

pub trait Commutative: Monoid {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CommutativeOp<M>(M);

impl<M> ops::Deref for CommutativeOp<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<M> ops::DerefMut for CommutativeOp<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<M> CommutativeOp<M> {
    pub fn into_inner(self) -> M {
        self.0
    }
}

impl<M: Monoid> BinaryOperation for CommutativeOp<M> {
    type ArgType1 = M::Element;
    type ArgType2 = M::Element;
    type OutputType = M::Element;

    fn op(&self, a: &M::Element, b: &M::Element) -> M::Element {
        self.0.op(a, b)
    }
}

impl<M: Monoid> Monoid for CommutativeOp<M> {
    type Element = M::Element;

    fn identity(&self) -> M::Element {
        self.0.identity()
    }

    fn op_assign(&self, a: &mut M::Element, b: &M::Element) {
        self.0.op_assign(a, b);
    }
}

impl<M: Monoid> Commutative for CommutativeOp<M> {}

impl<M: Group> Group for CommutativeOp<M> {
    fn inv(&self, a: &M::Element) -> M::Element {
        self.0.inv(a)
    }

    fn op_inv(&self, a: &M::Element, b: &M::Element) -> M::Element {
        self.0.op_inv(a, b)
    }

    fn op_inv_assign(&self, a: &mut M::Element, b: &M::Element) {
        self.0.op_inv_assign(a, b);
    }
}

impl<M: Idempotent> Idempotent for CommutativeOp<M> {}
