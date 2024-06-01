#![allow(unused)]
use std::{cell::Cell, marker::PhantomData, ptr::NonNull};

use crate::MemoryPool;

pub struct FixedMemoryPool<T> {
    memory: NonNull<T>,
    capacity: usize,
    next: Cell<usize>,
    _marker: PhantomData<T>,
}

impl<T> FixedMemoryPool<T> {
    pub fn new(size: usize) -> Self {
        todo!()
    }
}

impl<T> Drop for FixedMemoryPool<T> {
    fn drop(&mut self) {
        todo!()
    }
}

unsafe impl<T> MemoryPool<T> for FixedMemoryPool<T> {
    type Error = ();
    fn try_get_mut_ptr(&self) -> Result<NonNull<T>, Self::Error> {
        todo!()
    }
}
