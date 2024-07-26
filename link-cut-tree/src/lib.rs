mod node;
pub mod query;
pub mod tree;

use node::{Node, NodeRef};
use query::{Commutative, Query};

struct NodeValue<T> {
    value: T,
    id: usize,
}

struct NodeOP<OP> {
    inner: OP,
}

pub struct LinkCutTree<T, OP> {
    nodes: Vec<NodeRef<NodeValue<T>, T, NodeOP<OP>>>,
    op: NodeOP<OP>,
}

impl<T, OP> Drop for LinkCutTree<T, OP> {
    fn drop(&mut self) {
        for &node in self.nodes.iter() {
            unsafe {
                node.drop();
            }
        }
    }
}

impl<T, OP> LinkCutTree<T, OP> {
    pub const fn new(op: OP) -> Self {
        Self {
            nodes: Vec::new(),
            op: NodeOP { inner: op },
        }
    }
}

impl<T, OP: Query<ValT = T, QValT = T>> Query for NodeOP<OP> {
    type ValT = NodeValue<T>;
    type QValT = T;
    const IDENT: Self::QValT = OP::IDENT;
    fn op_left(&self, a: &Self::QValT, b: &Self::ValT) -> Self::QValT {
        self.inner.op_left(a, &b.value)
    }
    fn op_right(&self, a: &Self::ValT, b: &Self::QValT) -> Self::QValT {
        self.inner.op_right(&a.value, b)
    }
    fn op(&self, a: &Self::QValT, b: &Self::QValT) -> Self::QValT {
        self.inner.op(a, b)
    }
}

impl<OP> Commutative for NodeOP<OP>
where
    OP: Commutative,
    NodeOP<OP>: Query,
{
}

impl<T: Clone, OP: Query<ValT = T, QValT = T> + Commutative> LinkCutTree<T, OP> {
    pub fn from_iter<I>(op: OP, iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            nodes: iter
                .into_iter()
                .enumerate()
                .map(|(id, value)| {
                    NodeRef::new(Node::new(
                        NodeValue {
                            value: value.clone(),
                            id,
                        },
                        value,
                    ))
                })
                .collect(),
            op: NodeOP { inner: op },
        }
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        self.nodes
            .get(i)
            .map(|&node| unsafe { &node.borrow_val().value })
    }

    pub fn cut(&mut self, i: usize, j: usize) -> bool {
        self.nodes[i].cut_checked(self.nodes[j], &self.op)
    }

    pub fn link(&mut self, i: usize, j: usize) -> bool {
        self.nodes[i].link_checked(self.nodes[j], &self.op)
    }

    pub fn lca(&mut self, p: usize, i: usize, j: usize) -> Option<usize> {
        let p = self.nodes[p];
        let i = self.nodes[i];
        let j = self.nodes[j];
        p.evert(&self.op);
        if p != i.expose(&self.op).1 {
            return None;
        }
        let (lca, prev_p) = j.expose(&self.op);
        if prev_p == i {
            Some(lca.val().id)
        } else {
            None
        }
    }

    pub fn query(&mut self, i: usize, j: usize) -> Option<&T> {
        let i = self.nodes[i];
        let j = self.nodes[j];
        i.evert(&self.op);
        if i != j.expose(&self.op).1 {
            return None;
        }
        j.update_from_child(&self.op);
        Some(unsafe { j.borrow_query() })
    }
}

impl<T, Q: Default> Default for LinkCutTree<T, Q> {
    fn default() -> Self {
        Self::new(Q::default())
    }
}
