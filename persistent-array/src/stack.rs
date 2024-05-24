use std::{
    alloc::{handle_alloc_error, Layout},
    cell::Cell,
    mem::MaybeUninit,
    ptr::NonNull,
};

pub struct PersistentStackPool<T: ?Sized> {
    pool: Box<[Cell<MaybeUninit<*mut Node<T>>>]>,
    len: Cell<usize>,
}

unsafe impl<T: ?Sized + Send> Send for PersistentStackPool<T> {}

impl<T: ?Sized> Drop for PersistentStackPool<T> {
    fn drop(&mut self) {
        let len = self.len.get();
        for node in &self.pool[..len] {
            unsafe {
                let node_ptr = node.get().assume_init();
                drop(Box::from_raw(node_ptr));
            }
        }
    }
}

pub struct PersistentStack<'a, T: ?Sized> {
    head: Option<NonNull<Node<T>>>,
    pool: &'a PersistentStackPool<T>,
}

impl<'a, T: ?Sized> Clone for PersistentStack<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: ?Sized> Copy for PersistentStack<'a, T> {}

struct Node<T: ?Sized> {
    prev: Option<NonNull<Node<T>>>,
    value: T,
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
            let layout = Layout::array::<MaybeUninit<NonNull<Node<T>>>>(size).unwrap();
            let ptr = std::alloc::alloc(layout);
            if ptr.is_null() {
                handle_alloc_error(layout);
            }
            let pool = std::ptr::slice_from_raw_parts_mut(
                ptr as *mut Cell<MaybeUninit<*mut Node<T>>>,
                size,
            );
            Self {
                pool: Box::from_raw(pool),
                len: Cell::new(0),
            }
        }
    }

    pub fn get_empty_stack(&self) -> PersistentStack<'_, T> {
        PersistentStack {
            head: None,
            pool: self,
        }
    }
}

impl<'a, T> PersistentStack<'a, T> {
    pub fn push(&self, value: T) -> Self {
        let new_node = Node {
            prev: self.head,
            value,
        };
        let new_node_ptr = NonNull::from(Box::leak(Box::new(new_node)));
        let pool_last = self.pool.len.get();
        self.pool.pool[pool_last].set(MaybeUninit::new(new_node_ptr.as_ptr()));
        self.pool.len.set(pool_last + 1);
        Self {
            head: Some(new_node_ptr),
            pool: self.pool,
        }
    }

    pub fn top(&self) -> Option<&T> {
        self.head
            .as_ref()
            .map(|node| unsafe { &node.as_ref().value })
    }

    pub fn pop(&self) -> Self {
        Self {
            head: self.head.and_then(|node| unsafe { node.as_ref().prev }),
            pool: self.pool,
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
        assert_eq!(stack.top(), None);
        stack = stack.push(42);
        assert_eq!(stack.top(), Some(&42));
        stack = stack.push(43);
        assert_eq!(stack.top(), Some(&43));
        stack = stack.pop();
        assert_eq!(stack.top(), Some(&42));

        let prev_stack = stack;
        stack = stack.push(44);
        assert_eq!(prev_stack.top(), Some(&42));
        assert_eq!(stack.top(), Some(&44));
    }
}
