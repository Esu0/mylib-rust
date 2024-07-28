use std::num::NonZeroUsize;

use node::{NodeArray, NodeRef};

mod node;

#[derive(Debug)]
pub struct FibonacciHeap<K, T> {
    nodes: NodeArray<K, T>,
    max: Option<NodeRef>,
    len: usize,
}

pub struct Cursor(NonZeroUsize);

impl From<NodeRef> for Cursor {
    fn from(value: NodeRef) -> Self {
        Self(value.inner())
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
            (k, v)
        })
    }
}

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
        use std::{cmp::Reverse, collections::BinaryHeap};
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
}
