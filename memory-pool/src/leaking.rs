use std::{
    alloc::{handle_alloc_error, Layout},
    ptr::NonNull,
};

use crate::MemoryPoolAlloc;

pub struct LeakingMemoryPool;

unsafe impl MemoryPoolAlloc for LeakingMemoryPool {
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

#[cfg(test)]
mod tests {
    use crate::MemoryPool;

    use super::*;
    use std::alloc::Layout;

    #[test]
    fn test_allocate() {
        let pool = LeakingMemoryPool;
        let layout = Layout::new::<u32>();
        let ptr = pool.allocate(layout);
        unsafe {
            ptr.cast::<u32>().as_ptr().write(42);
            assert_eq!(ptr.cast::<u32>().as_ptr().read(), 42);
        }
        let addr = ptr.as_ptr() as usize;
        assert_eq!(addr % layout.align(), 0);
    }

    #[test]
    fn test_zero_size() {
        let pool = LeakingMemoryPool;
        let layout = Layout::new::<[usize; 0]>();
        let ptr = pool.allocate(layout);
        let addr = ptr.as_ptr() as usize;
        assert_eq!(addr % layout.align(), 0);
    }

    #[test]
    fn test_get_mut_ref() {
        let pool = LeakingMemoryPool;
        let value = 42u32;
        let a = pool.get_uninit_mut();
        a.write(value);
        let b = unsafe { a.assume_init_mut() };
        *b += 1;
        assert_eq!(*b, value + 1);
    }
}
