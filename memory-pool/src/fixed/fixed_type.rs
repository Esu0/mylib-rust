#![allow(unused)]
use std::{
    alloc::{self, handle_alloc_error, Layout},
    cell::Cell,
    marker::PhantomData,
    mem::size_of,
    ptr::NonNull,
};

use crate::MemoryPool;

pub struct FixedMemoryPool<T> {
    memory: NonNull<T>,
    capacity: usize,
    next: Cell<usize>,
}

unsafe impl<T: Send> Send for FixedMemoryPool<T> {}

impl<T> FixedMemoryPool<T> {
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::array::<T>(capacity).unwrap();
        if layout.size() == 0 {
            Self {
                memory: NonNull::dangling(),
                capacity: 0,
                next: Cell::new(0),
            }
        } else {
            let ptr = unsafe { alloc::alloc(layout) };
            let Some(memory) = NonNull::new(ptr) else {
                handle_alloc_error(layout);
            };
            Self {
                memory: memory.cast(),
                capacity,
                next: Cell::new(0),
            }
        }
    }

    /// 過去に取得したポインタまたは`&mut MaybeUninit<T>`の指す先のデータをすべてdropする。
    ///
    /// # Safety
    /// 過去に取得したポインタまたは`&mut MaybeUninit<T>`の指す先がすべて適切に初期化されている必要がある。
    pub unsafe fn drop_all(&mut self) {
        let len = *self.next.get_mut();
        for i in 0..len {
            self.memory.as_ptr().add(i).drop_in_place();
        }
    }
}

impl<T> Drop for FixedMemoryPool<T> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            let layout = Layout::array::<T>(self.capacity).unwrap();
            unsafe {
                alloc::dealloc(self.memory.cast().as_ptr(), layout);
            }
        }
    }
}

unsafe impl<T> MemoryPool<T> for FixedMemoryPool<T> {
    type Error = ();
    fn try_get_mut_ptr(&self) -> Result<NonNull<T>, Self::Error> {
        if size_of::<T>() == 0 {
            return Ok(NonNull::dangling());
        }
        let next = self.next.get();
        if next < self.capacity {
            self.next.set(next + 1);
            Ok(unsafe { NonNull::new_unchecked(self.memory.as_ptr().add(next)) })
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::align_of;

    use super::*;

    #[test]
    fn zero_size() {
        let pool = FixedMemoryPool::<[u64; 0]>::new(100);
        assert_eq!(pool.get_mut_ptr().as_ptr() as usize % align_of::<u64>(), 0);
        let uninit_a = pool.get_uninit_mut();
        let _a = unsafe { uninit_a.assume_init_mut() };

        let pool = FixedMemoryPool::<u32>::new(0);
        assert!(pool.try_get_mut_ptr().is_err());
    }

    #[test]
    fn normal() {
        let mut pool = FixedMemoryPool::<u32>::new(100);
        for i in 0..100 {
            let p = pool.get_uninit_mut();
            p.write(i);
            unsafe {
                assert_eq!(p.assume_init_read(), i);
            }
        }
        assert!(pool.try_get_mut_ptr().is_err());
        unsafe {
            pool.drop_all();
        }
    }
}
