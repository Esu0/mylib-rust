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

pub struct Edge<T, NodeRef> {
    val: T,
    node: NodeRef,
}

#[repr(transparent)]
pub struct NodeRef<Node>(NonNull<Node>);

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
        Self: Sized, {
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
                let edge_to_b_val = a_mut.right.take().unwrap_unchecked().val;
                let edge_to_b = Edge {
                    val: edge_to_b_val,
                    node: b_ref,
                };
                // link A -> B
                mem::replace(&mut a_mut.parent, Some(edge_to_b))
            }
        }
    }

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

    fn take_parent_edge_and_direction(mut self) -> Option<(Option<Direction>, Edge<E, Self>)> {
        self.node_mut().parent.take().map(|edge| (self.which_child(edge.node), edge))
    }

    fn to_tree<O: PathOperator<V = V, E = E, Path = Q> + Copy>(self, op: O) -> RawTree<Self, O> {
        RawTree { node: self, op }
    }

    fn to_path(self) -> Path<V, E, Q> {
        Path(self)
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
    pub fn splay(self) {
        let RawTree { node, op } = self;
        let mut edge_to_parent_opt = node.take_parent_edge_and_direction();
        while let Some((Some(dir1), edge_to_parent)) = edge_to_parent_opt {
            if let Some((Some(dir2), edge_to_grandparent)) = edge_to_parent.node.take_parent_edge_and_direction() {
                if dir1 == dir2 {
                    let gp = edge_to_grandparent.node;
                    let next_p = edge_to_parent.node.rot(edge_to_grandparent, dir2.opposite());
                    node.rot(edge_to_parent, dir1.opposite());
                    edge_to_parent_opt = next_p.map(|edge| (gp.which_child(edge.node), edge));
                } else {
                    let parent_ref = edge_to_parent.node;
                    node.rot(edge_to_parent, dir2);
                    let next_p = parent_ref.rot(edge_to_grandparent, dir1);
                    edge_to_parent_opt = next_p.map(|edge| (node.which_child(edge.node), edge));
                }
            } else {
                node.rot(edge_to_parent, dir1.opposite());
                return;
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::query;

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
}