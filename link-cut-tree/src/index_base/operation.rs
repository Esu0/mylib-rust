pub trait Operator {
    type ValT;
    type QValT;
    const IDENT: Self::QValT;
    fn val_to_query(&self, val: &Self::ValT) -> Self::QValT;
    fn operate(&self, a: &Self::QValT, b: &Self::QValT) -> Self::QValT;
}

pub trait Commutative: Operator {}

pub trait Reversable: Operator {
    fn reverse(&self, a: &Self::QValT) -> Self::QValT;
}

impl<O> Reversable for O
where
    O: Operator + Commutative,
    O::QValT: Clone,
{
    fn reverse(&self, a: &Self::QValT) -> Self::QValT {
        a.clone()
    }
}
