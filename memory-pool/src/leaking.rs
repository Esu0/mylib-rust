use std::{alloc::{handle_alloc_error, Layout}, ptr::NonNull};

use crate::MemoryPool;

pub struct LeakingMemoryPool;

unsafe impl MemoryPool for LeakingMemoryPool {
    type Error = ();
    fn try_allocate(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error> {
        unsafe {
            if layout.size() == 0 {
                let align = layout.align();
                // `align`は1以上だから、オーバーフローしない
                return Ok(NonNull::new_unchecked((usize::MAX - align + 1) as *mut u8));
            }
            let ptr = std::alloc::alloc(layout);
            let Some(ptr) = NonNull::new(ptr) else {
                handle_alloc_error(layout);
            };
            Ok(ptr)
        }
    }
}
