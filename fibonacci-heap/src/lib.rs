use std::num::NonZeroUsize;

use node::{NodeArray, NodeRef};

pub mod node;
pub use node::IncreaseKeyError;

#[derive(Debug)]
pub struct FibonacciHeap<K, T> {
    nodes: NodeArray<K, T>,
    max: Option<NodeRef>,
    len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cursor(NonZeroUsize);

impl From<NodeRef> for Cursor {
    fn from(value: NodeRef) -> Self {
        Self(value.inner())
    }
}

impl From<usize> for Cursor {
    fn from(value: usize) -> Self {
        Self(NonZeroUsize::new(value + 1).expect("overflowed in Cursor::from(usize)"))
    }
}

impl<K, T> FibonacciHeap<K, T> {
    pub const fn new() -> Self {
        Self {
            nodes: NodeArray::new(),
            max: None,
            len: 0,
        }
    }

    pub fn peek(&self) -> Option<(&K, &T)> {
        self.max
            .map(|node_ref| self.nodes.get(node_ref).key_value())
    }

    pub fn peek_key(&self) -> Option<&K> {
        self.max.map(|node_ref| self.nodes.get(node_ref).key())
    }

    pub fn peek_value(&self) -> Option<&T> {
        self.max.map(|node_ref| self.nodes.get(node_ref).value())
    }

    pub fn cursor_max(&self) -> Option<Cursor> {
        self.max.map(|node_ref| node_ref.into())
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get_key(&self, cursor: Cursor) -> Option<&K> {
        self.nodes.get_checked(cursor.0).map(|node| node.key())
    }

    pub fn get_value(&self, cursor: Cursor) -> Option<&T> {
        self.nodes.get_checked(cursor.0).map(|node| node.value())
    }

    pub fn get_key_value(&self, cursor: Cursor) -> Option<(&K, &T)> {
        self.nodes.get_checked(cursor.0).map(|node| node.key_value())
    }

    pub fn get_value_mut(&mut self, cursor: Cursor) -> Option<&mut T> {
        self.nodes.get_checked_mut(cursor.0).map(|node| node.value_mut())
    }

    pub fn get_key_value_mut(&mut self, cursor: Cursor) -> Option<(&K, &mut T)> {
        self.nodes.get_checked_mut(cursor.0).map(|node| {
            let (k_ref, v_ref) = node.key_value_mut();
            (&*k_ref, v_ref)
        })
    }
}

impl<K: Ord, T> FibonacciHeap<K, T> {
    pub fn push(&mut self, key: K, value: T) -> Cursor {
        let new_node_ref;
        if let Some(max) = self.max {
            let max_key = self.nodes.get(max).key();
            if max_key < &key {
                new_node_ref = self.nodes.insert_new_node_right(key, value, max);
                self.max = Some(new_node_ref);
            } else {
                new_node_ref = self.nodes.insert_new_node_right(key, value, max);
            }
        } else {
            unsafe {
                new_node_ref = self.nodes.push_cyclic_node(key, value);
                self.max = Some(new_node_ref);
            }
        }
        self.len += 1;
        new_node_ref.into()
    }

    pub fn pop(&mut self) -> Option<(K, T)> {
        self.max.map(|max| {
            let (new_max, k, v) = unsafe { self.nodes.remove_max(max) };
            self.max = new_max;
            self.len -= 1;
            (k, v)
        })
    }

    /// cursorが指すノードのキーをそれよりも大きいキー値`new_key`に更新する。
    ///
    /// 元のキー値よりも小さいキー値を指定した場合はヒープ構造が壊れる
    pub fn increase_key(&mut self, cursor: Cursor, new_key: K) -> Result<K, K> {
        if let Some(max) = self.max {
            self.nodes.increase_key_force_checked(max, cursor.0, new_key).map(|(old_key, new_max)| {
                self.max = Some(new_max);
                old_key
            })
        } else {
            Err(new_key)
        }
    }

    pub fn increase_key_checked(&mut self, cursor: Cursor, new_key: K) -> IncreaseKeyResult<K> {
        if let Some(max) = self.max {
            let (old_key, new_max) = self.nodes.increase_key_checked(max, cursor.0, new_key)?;
            self.max = Some(new_max);
            Ok(old_key)
        } else {
            Err(IncreaseKeyError::NotFound(new_key))
        }
    }
}

pub type IncreaseKeyResult<K> = Result<K, IncreaseKeyError<K>>;

impl<K, T> Default for FibonacciHeap<K, T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push() {
        let mut heap = FibonacciHeap::new();
        heap.push(3, ());
        heap.push(3, ());
        heap.push(10, ());
        assert_eq!(heap.pop(), Some((10, ())));
        assert_eq!(heap.pop(), Some((3, ())));
        heap.push(45, ());
        heap.push(1, ());
        heap.push(3, ());
        assert_eq!(heap.pop(), Some((45, ())));
        heap.push(10, ());
        heap.push(11, ());
        assert_eq!(heap.pop(), Some((11, ())));
        assert_eq!(heap.pop(), Some((10, ())));
        assert_eq!(heap.pop(), Some((3, ())));
        assert_eq!(heap.pop(), Some((3, ())));
        assert_eq!(heap.pop(), Some((1, ())));
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn dijkstra() {
        use proconio::input;
        use std::cmp::Reverse;
        let s = "17 44
1 2 104
1 3 80
2 4 64
2 5 60
2 6 56
2 7 52
2 8 48
3 4 72
3 5 48
3 6 44
3 7 40
3 8 36
4 9 32
4 10 28
4 11 24
5 9 40
5 10 16
5 11 12
6 9 48
6 10 24
6 11 0
7 9 56
7 10 32
7 11 8
8 9 64
8 10 40
8 11 16
9 12 48
9 13 44
9 14 40
9 15 36
10 12 56
10 13 32
10 14 28
10 15 24
11 12 64
11 13 40
11 14 16
11 15 12
12 16 0
13 16 8
14 16 16
15 16 24
16 17 132";
        let source = proconio::source::line::LineSource::new(s.as_bytes());
        input! {
            from source,
            n: usize,
            m: usize,
            uvb: [(usize, usize, u64); m],
        }

        let mut adj_list = vec![vec![]; n];
        for &(u, v, b) in &uvb {
            adj_list[u - 1].push((v - 1, b));
        }

        let mut heap = FibonacciHeap::new();
        heap.push(Reverse(0u64), 0usize);
        let mut dist = vec![u64::MAX; n];
        dist[0] = 0;
        while let Some((Reverse(w), node)) = heap.pop() {
            if w > dist[node] {
                continue;
            }
            for &(next, b) in &adj_list[node] {
                let next_w = w + b;
                if next_w < dist[next] {
                    dist[next] = next_w;
                    heap.push(Reverse(next_w), next);
                }
            }
        }
        assert_eq!(
            dist[1..],
            [104, 80, 152, 128, 124, 120, 116, 168, 144, 124, 188, 164, 140, 136, 156, 288]
        );
    }

    #[test]
    fn increase_key() {
        fn mpow(mut base: u32, mut exp: u64, modulo: u32) -> u32 {
            let mut result = 1;
            while exp > 0 {
                if exp & 1 != 0 {
                    result = (result as u64 * base as u64 % modulo as u64) as u32;
                }
                base = (base as u64 * base as u64 % modulo as u64) as u32;
                exp >>= 1;
            }
            result
        }

        let mut heap = FibonacciHeap::new();
        let cursors = (0usize..10).map(|i| heap.push(dbg!(mpow(7, i as u64, 41)), i)).collect::<Vec<_>>();
        assert_eq!(heap.pop().unwrap().0, 38);
        assert_eq!(heap.increase_key(cursors[0], 10), Ok(1));
        assert_eq!(heap.increase_key(cursors[2], 20), Ok(8));
        assert_eq!(heap.increase_key(cursors[6], 40), Ok(20));
        assert_eq!(heap.pop().unwrap().0, 40);
        assert_eq!(heap.pop().unwrap().0, 37);
        assert_eq!(heap.increase_key(cursors[0], 25), Ok(10));
    }

    #[test]
    fn dijkstra_fast() {
        use proconio::input;
        use std::cmp::Reverse;
        let s = "17 44
1 2 104
1 3 80
2 4 64
2 5 60
2 6 56
2 7 52
2 8 48
3 4 72
3 5 48
3 6 44
3 7 40
3 8 36
4 9 32
4 10 28
4 11 24
5 9 40
5 10 16
5 11 12
6 9 48
6 10 24
6 11 0
7 9 56
7 10 32
7 11 8
8 9 64
8 10 40
8 11 16
9 12 48
9 13 44
9 14 40
9 15 36
10 12 56
10 13 32
10 14 28
10 15 24
11 12 64
11 13 40
11 14 16
11 15 12
12 16 0
13 16 8
14 16 16
15 16 24
16 17 132";
        let source = proconio::source::line::LineSource::new(s.as_bytes());
        input! {
            from source,
            n: usize,
            m: usize,
            uvb: [(usize, usize, u64); m],
        }

        let mut adj_list = vec![vec![]; n];
        for &(u, v, b) in &uvb {
            adj_list[u - 1].push((v - 1, b));
        }

        let mut heap = FibonacciHeap::new();
        heap.push(Reverse(0u64), 0);
        for i in 1..n {
            heap.push(Reverse(u64::MAX), i);
        }

        let mut dist = vec![u64::MAX; n];
        while let Some((Reverse(w), node)) = heap.pop() {
            dist[node] = w;
            for &(next, b) in &adj_list[node] {
                let next_w = w + b;
                let _ = heap.increase_key_checked(Cursor::from(next), Reverse(next_w));
            }
            eprintln!("{:#?}", heap);
        }
        assert_eq!(
            dist[1..],
            [104, 80, 152, 128, 124, 120, 116, 168, 144, 124, 188, 164, 140, 136, 156, 288]
        );
    }
}
