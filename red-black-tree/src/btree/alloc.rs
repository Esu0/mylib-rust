use std::{alloc::{handle_alloc_error, Layout}, ptr::NonNull};

use memory_pool::{MemoryPool, fixed::fixed_type::FixedMemoryPool};

pub trait Allocator<T>: Clone {
    unsafe fn allocate(&self) -> NonNull<T>;
    unsafe fn deallocate(&self, ptr: NonNull<T>);
}

#[derive(Clone, Copy)]
pub struct GlobalAlloc;

impl<T> Allocator<T> for GlobalAlloc {
    /// `T`がZSTの場合を考慮していないので注意
    unsafe fn allocate(&self) -> NonNull<T> {
        let layout = Layout::new::<T>();
        NonNull::new(std::alloc::alloc(layout)).unwrap_or_else(|| handle_alloc_error(layout)).cast()
    }

    /// `T`がZSTの場合を考慮していないので注意
    unsafe fn deallocate(&self, ptr: NonNull<T>) {
        std::alloc::dealloc(ptr.cast().as_ptr(), Layout::new::<T>());
    }
}

impl<'a, T> Allocator<T> for &'a FixedMemoryPool<T> {
    unsafe fn allocate(&self) -> NonNull<T> {
        MemoryPool::get_mut_ptr(*self)
    }

    /// 呼ばれても何もしない
    unsafe fn deallocate(&self, _ptr: NonNull<T>) {}
}
