use std::ptr::NonNull;

use crate::{node::Direction::{self, *}, query::Operator};


pub struct Node<T, Q> {
    reverse: bool,
    query: Q,
    parent: Option<(NodeRef<T, Q>, T)>,
    left: Option<(NodeRef<T, Q>, T)>,
    right: Option<(NodeRef<T, Q>, T)>,
}

pub struct NodeRef<T, Q>(NonNull<Node<T, Q>>);

impl<T, Q> Clone for NodeRef<T, Q> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, Q> Copy for NodeRef<T, Q> {}

impl<T, Q> PartialEq for NodeRef<T, Q> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T, Q> Eq for NodeRef<T, Q> {}
