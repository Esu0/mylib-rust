use std::{alloc::{handle_alloc_error, Layout}, cell::Cell, ptr::NonNull};

use crate::MemoryPool;

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

unsafe impl MemoryPool for FixedSizeMemoryPool {
    type Error = ();

    fn try_allocate(&self, layout: Layout) -> Result<NonNull<u8>, Self::Error> {
        let size = layout.size();
        let align = layout.align();
        if size == 0 {
            unsafe {
                return Ok(NonNull::new_unchecked((usize::MAX - align + 1) as *mut u8));
            }
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
