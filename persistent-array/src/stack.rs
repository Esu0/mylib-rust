use std::{
    alloc::{handle_alloc_error, Layout},
    cell::{Cell, UnsafeCell},
    mem::MaybeUninit,
    ptr::NonNull,
};

pub struct PersistentStackPool<T> {
    pool: Box<[UnsafeCell<MaybeUninit<Node<T>>>]>,
    len: Cell<usize>,
}

// unsafe impl<T: Send> Send for PersistentStackPool<T> {}

impl<T> Drop for PersistentStackPool<T> {
    fn drop(&mut self) {
        let len = self.len.get();
        for node in &mut self.pool[..len] {
            unsafe {
                node.get_mut().assume_init_drop();
            }
        }
    }
}

pub struct PersistentStack<'a, T> {
    head: usize,
    pool: &'a PersistentStackPool<T>,
}

impl<'a, T> Clone for PersistentStack<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for PersistentStack<'a, T> {}

struct Node<T> {
    prev: usize,
    value: T,
}

impl<T> Node<T> {
    fn new(prev: usize, value: T) -> Self {
        Self { prev, value }
    }
}

impl<T> PersistentStackPool<T> {
    pub fn new(size: usize) -> Self {
        if size == 0 {
            return Self {
                pool: Box::new([]),
                len: Cell::new(0),
            };
        }
        unsafe {
            let layout = Layout::array::<MaybeUninit<Node<T>>>(size).unwrap();
            let ptr = std::alloc::alloc(layout) as *mut _;
            let Some(ptr) = NonNull::new(ptr) else {
                handle_alloc_error(layout);
            };
            let pool = NonNull::slice_from_raw_parts(ptr, size);
            Self {
                pool: Box::from_raw(pool.as_ptr()),
                len: Cell::new(0),
            }
        }
    }

    pub fn get_empty_stack(&self) -> PersistentStack<'_, T> {
        PersistentStack {
            head: usize::MAX,
            pool: self,
        }
    }

    #[cfg(test)]
    fn check_invariant(&self) {
        let len = self.len.get();
        assert!(len <= self.pool.len());
        for i in 0..len {
            let node_prev = unsafe { (*self.pool[i].get()).assume_init_ref().prev };
            assert!(node_prev == usize::MAX || node_prev < len);
        }
    }
}

impl<'a, T> PersistentStack<'a, T> {
    pub fn push(&self, value: T) -> Self {
        let new_node = Node::new(self.head, value);
        let pool_last = self.pool.len.get();
        unsafe {
            self.pool.pool[pool_last]
                .get()
                .write(MaybeUninit::new(new_node));
        }
        self.pool.len.set(pool_last + 1);
        Self {
            head: pool_last,
            pool: self.pool,
        }
    }

    pub fn top(&self) -> Option<&T> {
        if self.head == usize::MAX {
            None
        } else {
            Some(unsafe { &(*self.pool.pool[self.head].get()).assume_init_ref().value })
        }
    }

    pub fn pop(&self) -> Self {
        if self.head == usize::MAX {
            *self
        } else {
            Self {
                head: unsafe { (*self.pool.pool[self.head].get()).assume_init_ref().prev },
                pool: self.pool,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistent_stack_test() {
        let pool = PersistentStackPool::new(10);
        let mut stack = pool.get_empty_stack();
        pool.check_invariant();
        assert_eq!(stack.top(), None);
        stack = stack.push(42);
        pool.check_invariant();
        assert_eq!(stack.top(), Some(&42));
        stack = stack.push(43);
        pool.check_invariant();
        assert_eq!(stack.top(), Some(&43));
        stack = stack.pop();
        pool.check_invariant();
        assert_eq!(stack.top(), Some(&42));

        let prev_stack = stack;
        stack = stack.push(44);
        pool.check_invariant();
        assert_eq!(prev_stack.top(), Some(&42));
        assert_eq!(stack.top(), Some(&44));
        pool.check_invariant();
    }

    #[test]
    fn elem_with_drop() {
        let pool = PersistentStackPool::new(10);
        let stack = pool.get_empty_stack();
        let hello_world = stack.push("hello".to_owned()).push("world".to_owned());
        pool.check_invariant();
        let foo_bar = stack.push("foo".to_owned()).push("bar".to_owned());
        pool.check_invariant();
        let hello_rust = hello_world.pop().push("rust".to_owned());
        pool.check_invariant();
        assert_eq!(hello_rust.top().unwrap(), "rust");
        assert_eq!(hello_rust.pop().top().unwrap(), "hello");
        assert_eq!(foo_bar.top().unwrap(), "bar");
        assert_eq!(foo_bar.pop().top().unwrap(), "foo");
        assert_eq!(hello_world.top().unwrap(), "world");
        assert_eq!(hello_world.pop().top().unwrap(), "hello");
    }

    #[test]
    #[should_panic]
    fn out_of_bounds() {
        let pool = PersistentStackPool::new(5);
        let stack = pool.get_empty_stack();
        let s = "hello".to_owned();
        stack
            .push(s.clone())
            .push(s.clone())
            .push(s.clone())
            .push(s.clone())
            .push(s.clone())
            .push(s.clone());
    }

    #[test]
    #[should_panic]
    fn zero_size() {
        let pool = PersistentStackPool::new(0);
        let stack = pool.get_empty_stack();
        stack.push(42);
    }
}
