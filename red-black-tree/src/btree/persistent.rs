#![allow(unused_variables)]
#![allow(unused)]
use std::borrow::Borrow;

use super::node::Root;

pub struct PersistentBTree<K, V> {
    root: Option<Root<K, V>>,
}

impl<K, V> PersistentBTree<K, V> {
    pub const fn new() -> Self {
        Self { root: None }
    }
}

impl<K: Ord, V> PersistentBTree<K, V> {
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        todo!()
    }

    pub fn insert(&self, key: K, value: V) -> Self {
        todo!()
    }
}

impl<K, V> Default for PersistentBTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
