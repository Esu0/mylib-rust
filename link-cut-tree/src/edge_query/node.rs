use crate::query::PathOperator;

use super::super::util::Direction::{self, *};
use std::{fmt, hash::Hash, mem, ptr::NonNull};

pub struct Node<V, E, Q> {
    value: V,
    query: Q,
    reverse: bool,
    /// このノードを根とする二分木が表すパスの親頂点との辺と対応する
    parent: Option<Edge<E, NodeRef<Self>>>,
    left: Option<Edge<E, NodeRef<Self>>>,
    right: Option<Edge<E, NodeRef<Self>>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Edge<T, NodeRef> {
    val: T,
    node: NodeRef,
}

#[repr(transparent)]
pub struct NodeRef<Node>(NonNull<Node>);

impl<V, E, Q> Node<V, E, Q> {
    pub fn new(value: V, query: Q) -> Self {
        Self {
            value,
            query,
            reverse: false,
            parent: None,
            left: None,
            right: None,
        }
    }
}

impl<Node> Clone for NodeRef<Node> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Node> Copy for NodeRef<Node> {}

impl<Node> PartialEq for NodeRef<Node> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<Node> Eq for NodeRef<Node> {}

impl<Node> Hash for NodeRef<Node> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }

    fn hash_slice<H: std::hash::Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        let ptr = data.as_ptr() as *const NonNull<Node>;
        let len = data.len();
        NonNull::<Node>::hash_slice(unsafe { std::slice::from_raw_parts(ptr, len) }, state);
    }
}

impl<Node> NodeRef<Node> {
    pub fn from_boxed(node: Box<Node>) -> Self {
        Self(NonNull::from(Box::leak(node)))
    }
}

impl<V, E, Q> NodeRef<Node<V, E, Q>> {
    fn node(&self) -> &Node<V, E, Q> {
        unsafe { self.0.as_ref() }
    }

    fn node_mut(&mut self) -> &mut Node<V, E, Q> {
        unsafe { self.0.as_mut() }
    }

    fn set_parent(mut self, parent: Edge<E, Self>) {
        self.node_mut().parent = Some(parent);
    }

    fn rot_right(self, parent_edge: Edge<E, Self>) -> Option<Edge<E, Self>> {
        let mut b_ref = self;
        let edge_to_a = parent_edge;
        let b_mut = b_ref.node_mut();
        if let Some(b_right_mut) = &mut b_mut.right {
            //          A            B
            //         / \          / \
            //        B   C   ->   D   A
            //       / \              / \
            //      D   E            E   C
            unsafe {
                // link B -> A
                let mut e_ref = mem::replace(&mut b_right_mut.node, edge_to_a.node);
                let mut a_ref = edge_to_a.node;
                // link E -> A
                let edge_to_b = mem::replace(
                    e_ref.node_mut().parent.as_mut().unwrap_unchecked(),
                    edge_to_a,
                );
                let a_mut = a_ref.node_mut();
                // link A -> E
                a_mut.left.as_mut().unwrap_unchecked().node = e_ref;
                // link A -> B
                mem::replace(&mut a_mut.parent, Some(edge_to_b))
            }
        } else {
            //          A            B
            //         / \          / \
            //        B   C   ->   D   A
            //       /                  \
            //      D                    C
            unsafe {
                let mut a_ref = edge_to_a.node;
                // link B -> A
                b_mut.right = Some(edge_to_a);
                let a_mut = a_ref.node_mut();
                let edge_to_b_val = a_mut.left.take().unwrap_unchecked().val;
                let edge_to_b = Edge {
                    val: edge_to_b_val,
                    node: b_ref,
                };
                // link A -> B
                mem::replace(&mut a_mut.parent, Some(edge_to_b))
            }
        }
    }

    fn rot_left(self, parent_edge: Edge<E, Self>) -> Option<Edge<E, Self>> {
        let mut b_ref = self;
        let edge_to_a = parent_edge;
        let b_mut = b_ref.node_mut();
        if let Some(b_left_mut) = &mut b_mut.left {
            //          A            B
            //         / \          / \
            //        C   B   ->   A   D
            //           / \      / \
            //          E   D    C   E
            unsafe {
                // link B -> A
                let mut e_ref = mem::replace(&mut b_left_mut.node, edge_to_a.node);
                let mut a_ref = edge_to_a.node;
                // link E -> A
                let edge_to_b = mem::replace(
                    e_ref.node_mut().parent.as_mut().unwrap_unchecked(),
                    edge_to_a,
                );
                let a_mut = a_ref.node_mut();
                // link A -> E
                a_mut.right.as_mut().unwrap_unchecked().node = e_ref;
                // link A -> B
                mem::replace(&mut a_mut.parent, Some(edge_to_b))
            }
        } else {
            //          A            B
            //         / \          / \
            //        C   B   ->   A   D
            //             \      /
            //              D    C
            unsafe {
                let mut a_ref = edge_to_a.node;
                // link B -> A
                b_mut.left = Some(edge_to_a);
                let a_mut = a_ref.node_mut();
                let a_parent = &mut a_mut.parent;
                let edge_to_b_val = a_mut.right.take().unwrap_unchecked().val;
                let edge_to_b = Edge {
                    val: edge_to_b_val,
                    node: b_ref,
                };
                std::hint::black_box(a_parent);
                // link A -> B
                mem::replace(&mut a_mut.parent, Some(edge_to_b))
            }
        }
    }

    /// `parent_edge`が指すノードを中心に回転を行う。
    ///
    /// `parent_edge`が指すノードの親ノードを指す辺を返す。
    fn rot(self, parent_edge: Edge<E, Self>, dir: Direction) -> Option<Edge<E, Self>> {
        match dir {
            Left => self.rot_left(parent_edge),
            Right => self.rot_right(parent_edge),
        }
    }

    fn which_child(self, parent: Self) -> Option<Direction> {
        let parent_node = parent.node();
        let node_ref_eq = |Edge { node, .. }: &Edge<E, Self>| *node == self;
        if parent_node.left.as_ref().is_some_and(node_ref_eq) {
            Some(Left)
        } else if parent_node.right.as_ref().is_some_and(node_ref_eq) {
            Some(Right)
        } else {
            None
        }
    }

    fn take_parent(mut self) -> Option<Edge<E, Self>> {
        self.node_mut().parent.take()
    }

    fn take_parent_edge_and_direction(self) -> Option<(Option<Direction>, Edge<E, Self>)> {
        self.take_parent()
            .map(|edge| (self.which_child(edge.node), edge))
    }

    fn to_tree<O: PathOperator<V = V, E = E, Path = Q> + Copy>(self, op: O) -> RawTree<Self, O> {
        RawTree { node: self, op }
    }

    fn to_path(self) -> Path<V, E, Q> {
        Path(self)
    }

    fn to_bintree(self) -> Bintree<V, E, Q> {
        Bintree(self)
    }
}

#[cfg(test)]
impl NodeRef<Node<usize, EdgeIndex, ()>> {
    fn make_list(n: usize) -> Vec<Self> {
        if n == 0 {
            panic!();
        }
        let mut v = Vec::with_capacity(n);
        let mut node = NodeRef::from_boxed(Box::new(Node {
            value: 0,
            query: (),
            reverse: false,
            parent: None,
            left: None,
            right: None,
        }));
        v.push(node);
        for i in 1..n {
            let new_node = NodeRef::from_boxed(Box::new(Node {
                value: i,
                query: (),
                reverse: false,
                parent: Some(Edge {
                    val: EdgeIndex(i - 1, i),
                    node,
                }),
                left: None,
                right: None,
            }));
            node.node_mut().right = Some(Edge {
                val: EdgeIndex(i, i - 1),
                node: new_node,
            });
            v.push(new_node);
            node = new_node;
        }
        v
    }
}

struct EdgeIndex(usize, usize);
impl fmt::Display for EdgeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}-{})", self.0, self.1)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct RawTree<NodeRef, O> {
    node: NodeRef,
    op: O,
}

impl<V, E, Q, O> RawTree<NodeRef<Node<V, E, Q>>, O>
where
    O: PathOperator<V = V, E = E, Path = Q> + Copy,
{
    /// `vertices.len() == edges.len() + 1 && vertices.len() >= 1`
    fn make_list(
        vertices: impl IntoIterator<Item = V>,
        edges: impl IntoIterator<Item = E>,
        op: O,
    ) -> Vec<Self>
    where
        E: Clone,
    {
        let mut iter_v = vertices.into_iter();
        let left_v = iter_v.next().unwrap();
        let left_q = op.val_to_path(&left_v);
        let mut left = NodeRef::from_boxed(Box::new(Node::new(left_v, left_q)));
        let mut res = Vec::with_capacity(iter_v.size_hint().0);
        res.push(RawTree { node: left, op });
        for (v, e) in iter_v.zip(edges) {
            let q_prev = &left.node().query;
            let p2 = &op.val_to_path(&v);
            let mut new_node = Node::new(v, op.connect_path(q_prev, &e, p2));
            new_node.left = Some(Edge {
                val: e.clone(),
                node: left,
            });
            let new_node_ref = NodeRef::from_boxed(Box::new(new_node));
            left.node_mut().parent = Some(Edge {
                val: e.clone(),
                node: new_node_ref,
            });
            res.push(RawTree {
                node: new_node_ref,
                op,
            });
            left = new_node_ref;
        }
        res
    }

    fn update_query_from_child(self) {
        let Self { mut node, op } = self;
        let mut q = op.val_to_path(&node.node().value);
        if let Some(ref left) = node.node().left {
            let Edge { node, val } = left;
            let left_query = &node.node().query;
            let new_q = op.connect_path(left_query, val, &q);
            q = new_q;
        }
        if let Some(ref right) = node.node().right {
            let Edge { node, val } = right;
            let right_query = &node.node().query;
            let new_q = op.connect_path(&q, val, right_query);
            q = new_q;
        }
        node.node_mut().query = q;
    }

    fn update_and_get(&self) -> &Q {
        self.update_query_from_child();
        &self.node.node().query
    }

    pub fn splay(self) -> Option<Edge<E, NodeRef<Node<V, E, Q>>>> {
        let Self { node, op } = self;
        let mut edge_to_parent_opt = node.take_parent_edge_and_direction();
        while let Some((Some(dir1), edge_to_parent)) = edge_to_parent_opt {
            let p = edge_to_parent.node;
            let edge_to_grandparent_opt = p.take_parent_edge_and_direction();
            if let Some((Some(dir2), edge_to_grandparent)) = edge_to_grandparent_opt
            {
                let gp = edge_to_grandparent.node;
                let next_p;
                if dir1 == dir2 {
                    next_p = edge_to_parent
                        .node
                        .rot(edge_to_grandparent, dir2.opposite());
                    node.rot(edge_to_parent, dir1.opposite());
                } else {
                    node.rot(edge_to_parent, dir2);
                    next_p = node.rot(edge_to_grandparent, dir1);
                    // edge_to_parent_opt = next_p.map(|edge| (node.which_child(edge.node), edge));
                }
                edge_to_parent_opt = next_p.map(|edge| (gp.which_child(edge.node), edge));
                gp.to_tree(op).update_query_from_child();
                p.to_tree(op).update_query_from_child();
            } else {
                node.rot(edge_to_parent, dir1.opposite());
                p.to_tree(op).update_query_from_child();
                // self.update_query_from_child();
                return edge_to_grandparent_opt.map(|(_, edge)| edge);
            }
        }
        // self.update_query_from_child();
        edge_to_parent_opt.map(|(_, edge)| edge)
    }

    pub fn expose(self)
    where
        E: Clone,
    {
        let mut node = self;
        let mut right = None;
        while let Some(edge_to_parent) = node.splay() {
            node.node.node_mut().right = right;
            right = Some(Edge {
                val: edge_to_parent.val.clone(),
                node: node.node,
            });
            let p = edge_to_parent.node;
            node.node.node_mut().parent = Some(edge_to_parent);
            node.node = p;
        }
        node.node.node_mut().right = right;
        self.splay();
    }

    pub fn link(self, parent: NodeRef<Node<V, E, Q>>, edge: E)
    where
        E: Clone,
    {
        self.expose();
        self.node.set_parent(Edge {
            val: edge,
            node: parent,
        });
    }

    pub fn cut(mut self) -> Option<Edge<E, NodeRef<Node<V, E, Q>>>>
    where
        E: Clone,
    {
        self.expose();
        let left = self.node.node_mut().left.take();
        if let Some(ref left) = left {
            let mut node = left.node;
            node.node_mut().parent = None;
        }
        left
    }
}

struct Path<V, E, Q>(NodeRef<Node<V, E, Q>>);

impl<V: fmt::Display, E: fmt::Display, Q> NodeRef<Node<V, E, Q>> {
    fn fmt_path_rec(self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let node = self.node();
        if let Some(ref left) = node.left {
            left.node.fmt_path_rec(f)?;
            write!(f, " -{}-> ", left.val)?;
        }
        write!(f, "{}", node.value)?;
        if let Some(ref right) = node.right {
            write!(f, " -{}-> ", right.val)?;
            right.node.fmt_path_rec(f)?;
        }
        Ok(())
    }
}

impl<V: fmt::Display, E: fmt::Display, Q> fmt::Display for Path<V, E, Q> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt_path_rec(f)
    }
}

struct Bintree<V, E, Q>(NodeRef<Node<V, E, Q>>);
impl<V: fmt::Display, E, Q> NodeRef<Node<V, E, Q>> {
    fn fmt_tree_rec(self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        let node = self.node();
        if let Some(ref right) = node.right {
            let node = right.node;
            node.fmt_tree_rec(f, depth + 1)?;
        }
        writeln!(f, "{: >width$}", node.value, width = depth * 4)?;
        if let Some(ref left) = node.left {
            let node = left.node;
            node.fmt_tree_rec(f, depth + 1)?;
        }
        Ok(())
    }
}

impl<V: fmt::Display, E, Q> fmt::Display for Bintree<V, E, Q> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt_tree_rec(f, 0)
    }
}

struct Ancestors<V, E, Q>(NodeRef<Node<V, E, Q>>);
impl<V: fmt::Display, E, Q> NodeRef<Node<V, E, Q>> {
    fn fmt_ancestors(self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut node = self;
        write!(f, "{}", node.node().value)?;
        while let Some(edge) = &node.node().parent {
            node = edge.node;
            write!(f, " -> {}", node.node().value)?;
        }
        Ok(())
    }
}

impl<V: fmt::Display, E, Q> fmt::Display for Ancestors<V, E, Q> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt_ancestors(f)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::query;
    use super::*;

    #[test]
    fn rot_test() {
        let mut node = NodeRef::make_list(10);
        println!("{}", node[0].to_path());
        let edge_to_0 = node[1].node_mut().parent.take().unwrap();
        node[1].rot(edge_to_0, Left);
        println!("{}", node[1].to_path());
        println!("{}", node[0].to_path());
        node[8].to_tree(query::noop()).splay();
        println!("{}", node[8].to_path());
    }

    #[derive(Clone, Copy)]
    struct EdgeVerticeAdd;
    impl query::PathOperator for EdgeVerticeAdd {
        type V = usize;
        type E = usize;
        type Path = (usize, usize);
        fn val_to_path(&self, val: &Self::V) -> Self::Path {
            (0, *val)
        }
        fn connect_path(&self, p1: &Self::Path, edge: &Self::E, p2: &Self::Path) -> Self::Path {
            (p1.0 + *edge + p2.0, p1.1 + p2.1)
        }
    }

    impl query::ReversablePathOperator for EdgeVerticeAdd {
        fn reverse_path(&self, _: &mut Self::Path) {}
    }
    #[test]
    fn splay_test() {
        let nodes = RawTree::make_list(1..=10, 1..=9, EdgeVerticeAdd);
        eprintln!("{}", nodes.last().unwrap().node.to_path());
        eprintln!("{:?}", nodes.last().unwrap().node.node().query);
        // nodes[0].splay();
        // eprintln!("{}", nodes[0].node.to_bintree());
        // nodes[1].splay();
        for &t in &nodes {
            t.splay();
            eprintln!("{}:", t.node.node().value);
            eprintln!("{:?}", t.node.node().query);
            eprintln!("{}", t.node.to_bintree());
        }
    }

    #[derive(Clone, Copy, Debug)]
    struct EdgeAdd;
    impl query::PathOperator for EdgeAdd {
        type V = usize;
        type E = usize;
        type Path = usize;
        fn connect_path(&self, p1: &Self::Path, edge: &Self::E, p2: &Self::Path) -> Self::Path {
            *p1 + *edge + *p2
        }
        fn val_to_path(&self, _: &Self::V) -> Self::Path {
            0
        }
    }

    impl query::ReversablePathOperator for EdgeAdd {
        fn reverse_path(&self, _: &mut Self::Path) {}
    }

    #[test]
    fn link_cut_test() {
        let nodes = (0..=11).map(|i| NodeRef::from_boxed(Box::new(Node::new(i, 0))).to_tree(EdgeAdd)).collect::<Vec<_>>();
        let link = |i: usize, j: usize, e| nodes[i].link(nodes[j].node, e);
        link(1, 0, 1);
        link(4, 1, 4);
        link(5, 1, 5);
        link(2, 0, 2);
        link(6, 2, 6);
        link(8, 6, 8);
        link(9, 6, 9);
        link(10, 6, 10);
        link(11, 6, 11);
        link(3, 0, 3);
        link(7, 3, 7);
        // nodes[7].expose();

        // nodes[11].expose();
        for &t in &nodes {
            t.expose();
            eprintln!("{}:", t.node.node().value);
            eprintln!("{}", t.node.to_bintree());
            // eprintln!("{}", Ancestors(t.node));
            eprintln!();
        }
        eprintln!("-------------------------");
        eprintln!("{}", Ancestors(nodes[8].node));
        for &t in &nodes {
            eprintln!("{}:", t.node.node().value);
            eprintln!("{}", t.node.to_bintree());
        }
        nodes[8].expose();
        for &t in &nodes {
            eprintln!("{}:", t.node.node().value);
            eprintln!("{}", t.node.to_bintree());
        }
    }
}
