use crate::{BinaryOperation, Commutative, Group, Idempotent, Monoid};

pub struct Noop;

impl BinaryOperation for Noop {
    type ArgType1 = ();
    type ArgType2 = ();
    type OutputType = ();

    fn op(&self, _: &(), _: &()) {}
}

impl Monoid for Noop {
    type Element = ();

    fn identity(&self) {}

    fn op_assign(&self, _: &mut (), _: &()) {}
}

impl Idempotent for Noop {}
impl Commutative for Noop {}

impl Group for Noop {
    fn inv(&self, _: &()) {}

    fn op_inv(&self, _: &(), _: &()) {}

    fn op_inv_assign(&self, _: &mut (), _: &()) {}
}
