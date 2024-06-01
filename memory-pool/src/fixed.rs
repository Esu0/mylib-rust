pub mod fixed_type;

use std::{
    alloc::{handle_alloc_error, Layout},
    cell::Cell,
    ptr::NonNull,
};

use crate::MemoryPoolAlloc;

pub struct FixedSizeMemoryPool {
    memory: NonNull<u8>,
    capacity: usize,
    max_align: usize,
    next: Cell<usize>,
}

unsafe impl Send for FixedSizeMemoryPool {}

impl FixedSizeMemoryPool {
    pub fn new(capacity: usize, max_align: usize) -> Self {
        if capacity == 0 {
            Self {
                memory: NonNull::dangling(),
                capacity: 0,
                max_align,
                next: Cell::new(0),
            }
        } else {
            unsafe {
                let layout = Layout::from_size_align(capacity, max_align).unwrap();
                let ptr = std::alloc::alloc(layout);

                let Some(memory) = NonNull::new(ptr) else {
                    handle_alloc_error(layout)
                };
                Self {
                    memory,
                    capacity,
                    max_align,
                    next: Cell::new(0),
                }
            }
        }
    }
}

impl Drop for FixedSizeMemoryPool {
    fn drop(&mut self) {
        if self.capacity != 0 {
            unsafe {
                let layout = Layout::from_size_align_unchecked(self.capacity, self.max_align);
                std::alloc::dealloc(self.memory.as_ptr(), layout);
            }
        }
    }
}

/// alignは2の冪乗である必要がある
fn dangling_ptr_with_align(align: usize) -> NonNull<u8> {
    debug_assert!(align.is_power_of_two());
    unsafe { NonNull::new_unchecked(std::ptr::null_mut::<u8>().wrapping_sub(align)) }
}

unsafe impl MemoryPoolAlloc for FixedSizeMemoryPool {
    type Error = ();

    fn try_allocate(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error> {
        let size = layout.size();
        let align = layout.align();
        if size == 0 {
            return Ok(dangling_ptr_with_align(align));
        }
        assert!(align <= self.max_align);
        let next = self.next.get();
        // `start >= next`で、`start % align == 0`となる最小の`start`
        let start = (next + align - 1) & !(align - 1);
        // メモリサイズは`isize::MAX`を超えないから、`self.next`も`isize::MAX`を超えない
        // `start`の最大値は`isize::MAX + 1`だから、ギリギリオーバーフローしない
        let next = start + size;
        if next <= self.capacity {
            self.next.set(next);
            Ok(unsafe { NonNull::new_unchecked(self.memory.as_ptr().add(start)) })
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::MemoryPool;

    use super::*;
    use std::{alloc::Layout, thread};

    #[test]
    fn test_allocate() {
        let pool = FixedSizeMemoryPool::new(4096, 16);
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
        let pool = FixedSizeMemoryPool::new(4096, 16);
        let layout = Layout::new::<[usize; 0]>();
        let ptr = pool.allocate(layout);
        let addr = ptr.as_ptr() as usize;
        assert_eq!(addr % layout.align(), 0);
    }

    #[test]
    #[should_panic]
    fn over_capacity() {
        let pool = FixedSizeMemoryPool::new(4096, 16);
        let layout = Layout::new::<[u64; 1024]>(); // 8 * 1024 > 4096
        let _ptr = pool.allocate(layout);
    }

    #[test]
    #[should_panic]
    fn over_max_align() {
        let pool = FixedSizeMemoryPool::new(4096, 4);
        let layout = Layout::new::<u64>().align_to(8).unwrap();
        let _ptr = pool.allocate(layout);
    }

    #[test]
    fn multi_thread() {
        let pool = FixedSizeMemoryPool::new(4096, 16);
        let a = pool.get_uninit_mut();
        let b = pool.get_uninit_mut();
        a.write(42u32);
        let a = unsafe { a.assume_init_mut() };
        let c = pool.get_uninit_mut();
        b.write(10u32);
        let b = unsafe { b.assume_init_mut() };
        c.write(32u32);
        let c = unsafe { c.assume_init_mut() };
        assert_eq!(*a, 42);
        assert_eq!(*b, 10);
        assert_eq!(*c, 32);
        let th2 = thread::spawn(move || {
            let uninit_d = pool.get_uninit_mut();
            uninit_d.write("hello".to_owned());
            let d = unsafe { uninit_d.assume_init_mut() };
            d.push_str(" world");
            assert_eq!(&*d, "hello world");
            // dropは手動で呼ぶ必要がある。
            unsafe { uninit_d.assume_init_drop() };
        });
        th2.join().unwrap();
    }
}
