mod node;
pub mod query;
pub mod edge_query;

use node::{Node, NodeRef};
use query::{Commutative, Operator};

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

impl<T, OP: Operator<ValT = T, QValT = T>> Operator for NodeOP<OP> {
    type ValT = NodeValue<T>;
    type QValT = T;
    const IDENT: Self::QValT = OP::IDENT;
    fn op(&self, a: &Self::QValT, b: &Self::QValT) -> Self::QValT {
        self.inner.op(a, b)
    }
    fn val_to_query(&self, val: &Self::ValT) -> Self::QValT {
        self.inner.val_to_query(&val.value)
    }
}

impl<OP> Commutative for NodeOP<OP>
where
    OP: Commutative,
    NodeOP<OP>: Operator,
{
}

impl<T: Clone, OP: Operator<ValT = T, QValT = T> + Commutative> LinkCutTree<T, OP> {
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

    pub fn path_query(&mut self, i: usize, j: usize) -> Option<&T> {
        let i = self.nodes[i];
        let j = self.nodes[j];
        i.evert(&self.op);
        if i != j.expose(&self.op).1 {
            return None;
        }
        j.update_from_child(&self.op);
        Some(unsafe { j.borrow_query() })
    }

    pub fn parent(&mut self, root: usize, i: usize) -> Option<usize> {
        if root == i {
            return None;
        }
        let root = self.nodes[root];
        let i = self.nodes[i];
        root.evert(&self.op);
        if root != i.expose(&self.op).1 {
            return None;
        }
        use node::Direction::*;
        let mut parent = i.child(Left).unwrap();
        while let Some(next) = parent.child(Right) {
            parent = next;
        }
        Some(parent.val().id)
    }
}

impl<T, Q: Default> Default for LinkCutTree<T, Q> {
    fn default() -> Self {
        Self::new(Q::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_cut() {
        let mut lct = LinkCutTree::from_iter(query::Add::new(), vec![3, 4, 1, 2, 0, 7]);
        assert!(lct.path_query(0, 1).is_none());
        assert!(lct.link(0, 1));
        assert!(lct.link(2, 1));
        assert!(lct.link(1, 3));
        assert!(lct.link(0, 5));
        assert!(lct.link(4, 5));
        for i in 0..6 {
            for j in 0..6 {
                assert!(!lct.link(i, j));
            }
        }

        assert_eq!(*lct.path_query(0, 4).unwrap(), 10);
        assert_eq!(*lct.path_query(1, 5).unwrap(), 14);
        assert!(lct.cut(1, 0));
        assert!(lct.link(1, 4));
        assert_eq!(*lct.path_query(2, 0).unwrap(), 15);
    }
}
