pub trait Query {
    type Elem;
    const IDENT: Self::Elem;
    fn query(&self, a: &Self::Elem, b: &Self::Elem) -> Self::Elem;
}

pub trait Commutative: Query {}

impl Query for () {
    type Elem = ();
    const IDENT: Self::Elem = ();
    fn query(&self, _: &Self::Elem, _: &Self::Elem) -> Self::Elem {}
}
