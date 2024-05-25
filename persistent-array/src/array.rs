#[derive(Debug, Clone)]
struct Node<T: 'static> {
    value: T,
    left: Option<&'static Node<T>>,
    right: Option<&'static Node<T>>,
}

#[derive(Debug)]
pub struct PersistentArray<T: 'static> {
    root: Option<&'static Node<T>>,
    len: usize,
}

impl<T> Clone for PersistentArray<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for PersistentArray<T> {}

impl<T> PersistentArray<T> {
    pub const fn new() -> Self {
        Self {
            root: None,
            len: 0,
        }
    }

    fn get_node(self, index: usize) -> Option<&'static Node<T>> {
        if index >= self.len {
            return None;
        }
        let mut index = index + 1;
        let mut node = self.root?;
        while index > 1 {
            node = if index & 1 == 0 {
                node.left?
            } else {
                node.right?
            };
            index >>= 1;
        }
        Some(node)
    }

    pub fn get(self, index: usize) -> Option<&'static T> {
        self.get_node(index).map(|node| &node.value)
    }

    pub fn set(self, index: usize, value: T) -> Self
    where
        T: Clone,
    {
        if index >= self.len {
            return self;
        }
        let index = index + 1;
        let mut i = index;
        let mut node = self.root.unwrap();
        let depth = i.ilog2();

        let mut value = Some(value);
        let mut mem = Box::leak((0..=depth).map(|d| {
            if d == depth {
                Node {
                    value: value.take().unwrap(),
                    left: node.left,
                    right: node.right,
                }
            } else {
                let new_node = node.clone();
                node = if i & 1 == 0 {
                    node.left.unwrap()
                } else {
                    node.right.unwrap()
                };
                i >>= 1;
                new_node
            }
        }).collect::<Box<[_]>>());
        let mut mask = 1 << depth;
        while mem.len() > 1 {
            let (last, next) = mem.split_last_mut().unwrap();
            mem = next;
            mask >>= 1;
            if index & mask == 0 {
                mem.last_mut().unwrap().left = Some(last);
            } else {
                mem.last_mut().unwrap().right = Some(last);
            }
        }
        Self {
            root: Some(&mem[0]),
            len: self.len,
        }
    }

    pub fn len(self) -> usize {
        self.len
    }

    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    pub fn push(self, value: T) -> Self
    where
        T: Clone,
    {
        let index = self.len + 1;
        let mut i = index;
        let mut node = self.root;
        let depth = i.ilog2();

        let mut value = Some(value);
        let mut mem = Box::leak((0..=depth).map(|d| {
            if d == depth {
                Node {
                    value: value.take().unwrap(),
                    left: None,
                    right: None,
                }
            } else {
                let node_prev = node.unwrap();
                let new_node = node.unwrap().clone();
                node = if i & 1 == 0 {
                    node_prev.left
                } else {
                    node_prev.right
                };
                i >>= 1;
                new_node
            }
        }).collect::<Box<[_]>>());
        let mut mask = 1 << depth;
        while mem.len() > 1 {
            let (last, next) = mem.split_last_mut().unwrap();
            mem = next;
            mask >>= 1;
            if index & mask == 0 {
                mem.last_mut().unwrap().left = Some(last);
            } else {
                mem.last_mut().unwrap().right = Some(last);
            }
        }
        Self {
            root: Some(&mem[0]),
            len: index,
        }
    }
}

impl<I> FromIterator<I> for PersistentArray<I> {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let mut v = Vec::leak(iter.into_iter().map(|item| Node {
            value: item,
            left: None,
            right: None,
        }).collect());
        let len = v.len();
        while v.len() > 1 {
            let i = v.len() - 1;
            let a = 1usize << v.len().ilog2();
            let (last, next) = v.split_last_mut().unwrap();
            v = next;
            if (i + 1) & (a >> 1) != 0 {
                debug_assert!(v[i - a].right.is_none());
                v[i - a].right = Some(last);
            } else {
                debug_assert!(v[i - a / 2].left.is_none());
                v[i - a / 2].left = Some(last);
            }
        }
        assert!(v.len() == 1);
        Self {
            root: Some(&v[0]),
            len,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_iter() {
        let a = (0..10).collect::<PersistentArray<_>>();
        for i in 0..10 {
            assert_eq!(a.get(i), Some(&i));
        }
        assert!(a.get(10).is_none());
        assert!(a.get(usize::MAX).is_none());
    }

    #[test]
    fn set() {
        let a = (0..10).collect::<PersistentArray<_>>();
        let b = a.set(5, 100);
        for i in 0..10 {
            if i == 5 {
                assert_eq!(*b.get(i).unwrap(), 100);
            } else {
                assert_eq!(*b.get(i).unwrap(), i);
            }
            assert_eq!(*a.get(i).unwrap(), i);
        }
        assert!(a.get(10).is_none());
        assert!(b.get(10).is_none());
        println!("{:#?}", b);
        let c = b.set(6, 200);
        for i in 0..10 {
            if i == 5 {
                assert_eq!(*c.get(i).unwrap(), 100);
            } else if i == 6 {
                assert_eq!(*c.get(i).unwrap(), 200);
            } else {
                assert_eq!(*c.get(i).unwrap(), i);
            }
        }
        assert!(c.get(10).is_none());
        assert!(c.get(usize::MAX).is_none());
    }

    #[test]
    fn big() {
        let a = (0..1_000_000).collect::<PersistentArray<_>>();
        let b = a.set(3, 0);
        assert_eq!(*b.get(3).unwrap(), 0);
        assert_eq!(*a.get(3).unwrap(), 3);
    }

    #[test]
    fn push() {
        let a = PersistentArray::new();
        let b = a.push(0usize);
        assert_eq!(*b.get(0).unwrap(), 0);
        assert!(b.get(1).is_none());
        let c = b.push(1);
        assert_eq!(*b.get(0).unwrap(), 0);
        assert!(b.get(1).is_none());
        assert_eq!(*c.get(0).unwrap(), 0);
        assert_eq!(*c.get(1).unwrap(), 1);
        assert!(c.get(2).is_none());
        let d = c.push(5).push(6).push(7);
        assert_eq!(d.len(), 5);
        assert_eq!(*d.get(4).unwrap(), 7);
    }
}
