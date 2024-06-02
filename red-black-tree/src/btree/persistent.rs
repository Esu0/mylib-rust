#![allow(unused_variables)]
#![allow(unused)]
use std::{borrow::Borrow, ptr::NonNull};

use memory_pool::fixed::fixed_type::FixedMemoryPool;

use super::{alloc::Allocator, node::{self, InternalNode, LeafNode, NodeRef, NodeRefOwned, Root}};

pub struct PersistentBTreePool<K, V> {
    leaf_pool: FixedMemoryPool<LeafNode<K, V>>,
    internal_pool: FixedMemoryPool<InternalNode<K, V>>,
}

impl<K, V> PersistentBTreePool<K, V> {
    pub fn new(max_leaf: usize, max_internal: usize) -> Self {
        Self {
            leaf_pool: FixedMemoryPool::new(max_leaf),
            internal_pool: FixedMemoryPool::new(max_internal),
        }
    }

    pub fn empty_tree(&self) -> PersistentBTree<K, V, &Self> {
        PersistentBTree {
            root: None,
            alloc: self,
        }
    }
}

impl<K, V> Drop for PersistentBTreePool<K, V> {
    fn drop(&mut self) {
        unsafe {
            self.leaf_pool.drop_all();
            self.internal_pool.drop_all();
        }
    }
}

impl<'a, K, V> Allocator<LeafNode<K, V>> for &'a PersistentBTreePool<K, V> {
    unsafe fn allocate(&self) -> NonNull<LeafNode<K, V>> {
        (&self.leaf_pool).allocate()
    }

    unsafe fn deallocate(&self, ptr: NonNull<LeafNode<K, V>>) {
        (&self.leaf_pool).deallocate(ptr)
    }
}

impl<'a, K, V> Allocator<InternalNode<K, V>> for &'a PersistentBTreePool<K, V> {
    unsafe fn allocate(&self) -> NonNull<InternalNode<K, V>> {
        (&self.internal_pool).allocate()
    }

    unsafe fn deallocate(&self, ptr: NonNull<InternalNode<K, V>>) {
        (&self.internal_pool).deallocate(ptr)
    }
}

pub struct PersistentBTree<K, V, A>
where
    A: Allocator<LeafNode<K, V>> + Allocator<InternalNode<K, V>>,
{
    root: Option<Root<K, V>>,
    alloc: A,
}

impl<K: Ord, V, A> PersistentBTree<K, V, A>
where
    A: Allocator<LeafNode<K, V>> + Allocator<InternalNode<K, V>>,
{
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        todo!()
    }

    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        todo!()
    }

    pub fn insert(&self, key: K, value: V) -> Self {
        if let Some(root) = &self.root {
            todo!()
        } else {
            let mut leaf = NodeRefOwned::new_leaf(self.alloc.clone());
            leaf.borrow_mut().push(key, value);
            Self {
                root: Some(leaf.forget_type()),
                alloc: self.alloc.clone(),
            }
        }
    }
}
