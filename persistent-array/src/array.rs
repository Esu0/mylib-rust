use std::{alloc::{handle_alloc_error, Layout}, cell::Cell, mem::MaybeUninit, ptr::NonNull};

// struct MemoryPool<T> {
//     pool: NonNull<[T]>,
//     tail: Cell<usize>,
// }

// impl<T> MemoryPool<T> {
//     fn new(count: usize) -> Self {
//         if count == 0 {
//             return Self {
//                 pool: NonNull::slice_from_raw_parts(NonNull::dangling(), 0),
//                 tail: Cell::new(0),
//             }
//         }
//         let layout = Layout::array::<T>(count).unwrap();
//         unsafe {
//             let ptr = std::alloc::alloc(layout) as *mut T;
//             let Some(ptr) = NonNull::new(ptr) else {
//                 std::alloc::handle_alloc_error(layout);
//             };
//             let pool = NonNull::slice_from_raw_parts(ptr, count);
//             Self {
//                 pool,
//                 tail: Cell::new(0),
//             }
//         }
//     }

//     fn checked_get(&self, n: usize) -> Option<NonNull<[T]>> {
//         let tail = self.tail.get();
//         let new_tail = tail.checked_add(n)?;
//         if new_tail > self.pool.len() {
//             None
//         } else {
//             self.tail.set(new_tail);
//             unsafe {
//                 Some(NonNull::slice_from_raw_parts(NonNull::new_unchecked(self.pool.cast::<T>().as_ptr().add(tail)), n))
//             }
//         }
//     }

//     fn get(&self, n: usize) -> NonNull<[T]> {
//         self.checked_get(n).unwrap_or_else(|| {
//             panic!("Memory pool exhausted")
//         })
//     }

//     fn get_array<const N: usize>(&self) -> NonNull<[T; N]> {
//         self.get(N).cast::<[T; N]>()
//     }

//     fn get_one(&self) -> NonNull<T> {
//         self.get(1).cast::<T>()
//     }
// }

// struct PersistentArrayPool<T, const N: usize> {
//     pool: MemoryPool<Node<T, N>>,
// }

struct Node<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    next: [MaybeUninit<NonNull<Node<T, N>>>; N],
}

impl<T, const N: usize> Node<T, N> {
    fn new_ptr() -> NonNull<Self> {
        let layout = Layout::new::<Self>();
        unsafe {
            let ptr = std::alloc::alloc(layout) as *mut Self;
            let Some(ptr) = NonNull::new(ptr) else {
                handle_alloc_error(layout);
            };
            ptr
        }
    }
}

pub struct PersistentArray<T, const N: usize> {
    head: Option<NonNull<Node<T, N>>>,
    len: usize,
}

impl<T, const N: usize> PersistentArray<T, N> {
    pub const fn new() -> Self {
        Self {
            head: None,
            len: 0,
        }
    }
    const fn max_depth() -> usize {
        let mut count = 1;
        let mut n = 1usize;
        while let Some(next) = n.checked_mul(N) {
            if let Some(next) = next.checked_add(1) {
                n = next;
                count += 1;
            } else {
                break;
            }
        }
        count
    }
    const MAX_DEPTH: usize = Self::max_depth();
    
    const fn depth_calculate_data() -> [usize; 64] {
        let mut data = [0usize; 64];
        let mut i = 1;
        while i <= Self::max_depth() {
            data[i] = data[i - 1] * N + 1;
            i += 1;
        }
        data
    }

    const DEPTH_DATA: [usize; 64] = Self::depth_calculate_data();

    pub fn get(&self, index: usize) -> Option<&T> {
        if self.len <= index {
            return None;
        }
        let offset = index % N;
        let mut node_index = index / N;
        let i = Self::DEPTH_DATA[0..=Self::MAX_DEPTH].partition_point(|&d| d <= node_index);
        let k = node_index - Self::DEPTH_DATA[i];
        
    }

    pub fn set(&self, index: usize, value: T) -> Self {
        todo!()
    }

    pub fn from_iter_node_count<Iter: IntoIterator<Item = T>>(iter: Iter, node_count: usize) -> Self {
        let mut iter = iter.into_iter();
        let layout = Layout::array::<Node<T, N>>(node_count).unwrap();
        unsafe {
            let ptr = std::alloc::alloc(layout) as *mut Node<T, N>;
            let Some(ptr) = NonNull::new(ptr) else {
                handle_alloc_error(layout);
            };
            let mut pool = NonNull::slice_from_raw_parts(ptr, node_count);
            let mut ch = ptr.as_ptr().add(1);
            let last = ptr.as_ptr().add(node_count);
            for (i, node) in pool.as_mut().iter_mut().enumerate() {
                for j in 0..N {
                    let Some(item) = iter.next() else {
                        return Self {
                            head: Some(ptr),
                            len: i * N + j,
                        };
                    };
                    node.data[j].write(item);
                    if ch < last {
                        node.next[j].write(NonNull::new_unchecked(ch));
                        ch = ch.add(1);
                    }
                }
            }
            Self {
                head: Some(ptr),
                len: node_count * N,
            }
        }
    }
}

impl<I, const N: usize> FromIterator<I> for PersistentArray<I, N> {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();
        let lower_div_n = lower.div_ceil(N);
        if lower_div_n == 0 {
            return Self::new();
        }
        if upper.is_some_and(|upper| upper.div_ceil(N) == lower_div_n) {
            Self::from_iter_node_count(iter, lower_div_n)
        } else {
            let buf = iter.collect::<Vec<_>>();
            let node_count = buf.len().div_ceil(N);
            Self::from_iter_node_count(buf, node_count)
        }
    }
}
