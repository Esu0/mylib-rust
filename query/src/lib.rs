pub mod operation;

pub trait Monoid {
    type Element: Clone;
    const IDENTITY: Self::Element;

    fn op(&self, a: &Self::Element, b: &Self::Element) -> Self::Element;

    fn op_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        *a = self.op(a, b);
    }
}

pub trait Group: Monoid {
    /// `self.op(&a, &self.inv(&a)) == Self::IDENTITY`を満たす必要がある。
    fn inv(&self, a: &Self::Element) -> Self::Element;

    /// `self.op(&a, &x) == b`を満たす`x`を返す。
    fn op_inv(&self, a: &Self::Element, b: &Self::Element) -> Self::Element {
        self.op(a, &self.inv(b))
    }

    fn op_inv_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        *a = self.op_inv(a, b);
    }
}

/// self.op(&a, &b) == self.op(&b, &a)を満たす必要がある。
pub trait AbelianGroup: Group {}

pub trait Ring {
    type Element: Clone;
    const ADD_IDENTITY: Self::Element;
    const MUL_IDENTITY: Self::Element;

    fn add(&self, a: &Self::Element, b: &Self::Element) -> Self::Element;
    fn mul(&self, a: &Self::Element, b: &Self::Element) -> Self::Element;
    fn neg(&self, a: &Self::Element) -> Self::Element;
    fn sub(&self, a: &Self::Element, b: &Self::Element) -> Self::Element {
        self.add(a, &self.neg(b))
    }

    fn add_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        *a = self.add(a, b);
    }
    fn mul_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        *a = self.mul(a, b);
    }
    fn sub_assign(&self, a: &mut Self::Element, b: &Self::Element) {
        *a = self.sub(a, b);
    }
}
