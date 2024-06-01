use std::borrow::Borrow;

use node::Root;

pub struct BTree<K, V> {
    root: Option<Root<K, V>>,
}

impl<K, V> BTree<K, V> {
    pub const fn new() -> Self {
        Self { root: None }
    }
}

#[allow(unused_variables)]
impl<K: Ord, V> BTree<K, V> {
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        todo!()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        todo!()
    }
}

impl<K, V> Default for BTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

mod mem;
mod node;
pub mod persistent;
mod alloc;
