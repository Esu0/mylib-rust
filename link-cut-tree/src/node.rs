#![allow(dead_code)]
use std::{
    collections::HashSet,
    fmt,
    hash::Hash,
    marker::PhantomData,
    ptr::{addr_of, addr_of_mut, NonNull},
};

#[derive(Clone)]
pub struct Node<T, Q> {
    value: T,
    query: T,
    parent: Option<NodeRef<T, Q>>,
    left: Option<NodeRef<T, Q>>,
    right: Option<NodeRef<T, Q>>,
    _marker: PhantomData<fn() -> Q>,
}

#[repr(transparent)]
pub struct NodeRef<T, Q>(NonNull<Node<T, Q>>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Left => Right,
            Right => Left,
        }
    }
}

use Direction::*;

use crate::query::{Commutative, Query};

impl<T, Q> Clone for NodeRef<T, Q> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, Q> Copy for NodeRef<T, Q> {}

impl<T, Q> PartialEq for NodeRef<T, Q> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T, Q> Eq for NodeRef<T, Q> {}

impl<T, Q> Hash for NodeRef<T, Q> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }

    fn hash_slice<H: std::hash::Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        NonNull::hash_slice(
            unsafe {
                std::slice::from_raw_parts(data.as_ptr() as *const NonNull<Node<T, Q>>, data.len())
            },
            state,
        )
    }
}

impl<T: Clone, Q> Node<T, Q> {
    pub fn new(value: T) -> Self {
        Self {
            query: value.clone(),
            value,
            parent: None,
            left: None,
            right: None,
            _marker: PhantomData,
        }
    }
}

impl<T, Q> NodeRef<T, Q> {
    pub fn new(node: Node<T, Q>) -> Self {
        Self(NonNull::from(Box::leak(Box::new(node))))
    }

    pub fn child(self, dir: Direction) -> Option<Self> {
        unsafe {
            let ptr = match dir {
                Left => addr_of!((*self.0.as_ptr()).left),
                Right => addr_of!((*self.0.as_ptr()).right),
            };
            ptr.read()
        }
    }

    pub fn parent(self) -> Option<Self> {
        unsafe { addr_of!((*self.0.as_ptr()).parent).read() }
    }

    pub fn set_child(self, dir: Direction, child: Option<Self>) -> Option<Self> {
        unsafe {
            let ptr = match dir {
                Left => addr_of_mut!((*self.0.as_ptr()).left),
                Right => addr_of_mut!((*self.0.as_ptr()).right),
            };
            let old = ptr.read();
            ptr.write(child);
            old
        }
    }

    pub fn set_parent(self, parent: Option<Self>) -> Option<Self> {
        unsafe {
            let ptr = addr_of_mut!((*self.0.as_ptr()).parent);
            let old = ptr.read();
            ptr.write(parent);
            old
        }
    }

    /// selfを親、childを子とし、selfとchildを双方向にリンクする
    ///
    /// # Returns
    /// もともとselfの子だったノードと、もともとchildの親だったノード
    pub fn link_child(self, dir: Direction, child: Option<Self>) -> (Option<Self>, Option<Self>) {
        let old_parent = child.and_then(|child| child.set_parent(Some(self)));
        let old_child = self.set_child(dir, child);
        (old_child, old_parent)
    }

    /// dirの方向に木の回転を行う。childはdirの反対方向の子であると仮定して回転を行うことに注意
    /// selfとchildが双方向にリンクされている必要はない
    pub fn rot_child(self, child: Self, dir: Direction) -> Option<Self> {
        let (old_child, old_parent) = child.link_child(dir, Some(self));
        self.link_child(dir.opposite(), old_child);
        child.set_parent(old_parent);
        old_parent
    }

    pub fn rot(self, dir: Direction) -> Option<(Self, Option<Self>)> {
        self.child(dir.opposite())
            .map(|child| (child, self.rot_child(child, dir)))
    }

    /// 回転を行わなかった場合None、回転を行った場合Some((新しい親, 回転した方向))
    pub fn rot_parent(self) -> Option<(Option<Self>, Direction)> {
        self.parent().and_then(|parent| {
            parent
                .direction(self)
                .map(|dir| (parent.rot_child(self, dir.opposite()), dir.opposite()))
        })
    }

    pub fn direction(self, child: Self) -> Option<Direction> {
        if self.child(Left) == Some(child) {
            Some(Left)
        } else if self.child(Right) == Some(child) {
            Some(Right)
        } else {
            None
        }
    }

    pub fn parent_and_direction(self) -> Option<(Self, Option<Direction>)> {
        self.parent().map(|parent| (parent, parent.direction(self)))
    }

    pub fn insert_val(self, dir: Direction, value: T) -> Self
    where
        T: Clone,
    {
        let child = self.child(dir);
        let mut new_node = Node {
            query: value.clone(),
            value,
            parent: Some(self),
            left: None,
            right: None,
            _marker: PhantomData,
        };
        match dir {
            Left => new_node.left = child,
            Right => new_node.right = child,
        };
        let new_node_ref = NodeRef::new(new_node);
        if let Some(child) = child {
            child.set_parent(Some(new_node_ref));
        }
        self.set_child(dir, Some(new_node_ref));
        new_node_ref
    }

    pub fn dfs(self, mut f: impl FnMut(Self, Self)) {
        let mut stack = vec![self];
        let mut visited = HashSet::from([self]);
        while let Some(node) = stack.pop() {
            let left = node.child(Left);
            let right = node.child(Right);
            let parent = node.parent();
            for next in [left, right, parent].into_iter().flatten() {
                f(node, next);
                if visited.insert(next) {
                    stack.push(next);
                }
            }
        }
    }

    pub fn node(&self) -> &Node<T, Q> {
        unsafe { self.0.as_ref() }
    }

    pub fn node_mut(&mut self) -> &mut Node<T, Q> {
        unsafe { self.0.as_mut() }
    }

    pub fn val(&self) -> &T {
        unsafe {
            let ptr = addr_of!((*self.0.as_ptr()).value);
            &*ptr
        }
    }

    pub fn val_mut(&mut self) -> &mut T {
        unsafe {
            let ptr = addr_of_mut!((*self.0.as_ptr()).value);
            &mut *ptr
        }
    }

    pub fn query(&self) -> &T {
        unsafe {
            let ptr = addr_of!((*self.0.as_ptr()).query);
            &*ptr
        }
    }

    pub fn query_mut(&mut self) -> &mut T {
        unsafe {
            let ptr = addr_of_mut!((*self.0.as_ptr()).query);
            &mut *ptr
        }
    }
}

impl<T: Clone, Q: Query<Elem = T> + Commutative> NodeRef<T, Q> {
    pub fn update_from_child(mut self, op: &Q) {
        let Node {
            value: ref val,
            query: query_mut,
            left,
            right,
            ..
        } = self.node_mut();
        match (*left, *right) {
            (Some(left), Some(right)) => {
                *query_mut = op.query(&op.query(left.query(), val), right.query())
            }
            (Some(left), None) => *self.query_mut() = op.query(left.query(), val),
            (None, Some(right)) => *self.query_mut() = op.query(val, right.query()),
            (None, None) => *self.query_mut() = val.clone(),
        };
    }

    /// selfのクエリの値は更新しない
    pub fn splay(self, op: &Q) -> Option<Self> {
        let mut pd = self.parent_and_direction();
        while let Some((p, Some(dir1))) = pd {
            if let Some((gp, Some(dir2))) = p.parent_and_direction() {
                if dir1 == dir2 {
                    let next_p = gp.rot_child(p, dir1.opposite());
                    p.rot_child(self, dir1.opposite());
                    pd = next_p.map(|p| (p, p.direction(gp)));
                } else {
                    p.rot_child(self, dir2);
                    let next_p = gp.rot_child(self, dir1);
                    pd = next_p.map(|p| (p, p.direction(gp)));
                }
                gp.update_from_child(op);
                p.update_from_child(op);
                // self.update_from_child(query);
            } else {
                let ret = p.rot_child(self, dir1.opposite());
                p.update_from_child(op);
                // self.update_from_child(op);
                return ret;
            }
        }
        // self.update_from_child(op);
        pd.map(|(p, _)| p)
    }

    pub fn expose(self, op: &Q) {
        let mut prev = None;
        let mut current = self;
        while let Some(p) = current.splay(op) {
            current.set_child(Right, prev);
            prev = Some(current);
            current = p;
        }
        current.set_child(Right, prev);
        self.splay(op);
        // self.update_from_child(op);
    }

    pub fn cut(self, op: &Q) {
        self.expose(op);
        if let Some(node) = self.child(Left) {
            self.set_child(Left, None);
            node.set_parent(None);
        }
    }

    /// nodeを親としてパスselfをくっつける
    pub fn link(self, node: Self, op: &Q) {
        self.expose(op);
        self.update_from_child(op);
        node.expose(op);
        node.link_child(Right, Some(self));
        node.update_from_child(op);
    }

    pub fn update_and_get_query(&self, op: &Q) -> &T {
        self.update_from_child(op);
        self.query()
    }
}

pub struct Tree<T, Q>(Option<NodeRef<T, Q>>);

impl<T, Q> Clone for Tree<T, Q> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, Q> Copy for Tree<T, Q> {}

impl<T: fmt::Display, Q> Tree<T, Q> {
    fn fmt_rec(&self, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
        if let Some(node) = self.0 {
            let node = node.node();
            Tree(node.right).fmt_rec(f, depth + 1)?;
            writeln!(f, "{:indent$}{}", "", node.value, indent = depth * 2)?;
            Tree(node.left).fmt_rec(f, depth + 1)?;
        }
        Ok(())
    }
}

impl<T: fmt::Display, Q> fmt::Display for Tree<T, Q> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_rec(f, 0)
    }
}

impl<T, Q> From<NodeRef<T, Q>> for Tree<T, Q> {
    fn from(value: NodeRef<T, Q>) -> Self {
        Tree(Some(value))
    }
}

impl<T: fmt::Debug, Q> NodeRef<T, Q> {
    fn debug_ancestor(self) {
        let mut current = self;
        print!("{:?}", current.node().value);
        while let Some(parent) = current.parent() {
            current = parent;
            print!(" -> {:?}", current.node().value);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dfs_test() {
        let node = NodeRef::new(Node::<_, ()>::new(100u32));
        let node70 = node
            .insert_val(Left, 50)
            .insert_val(Left, 30)
            .insert_val(Right, 70);
        node.insert_val(Right, 45);

        println!("{}", Tree::from(node));
        println!("{}", Tree::from(node70));
        node.dfs(|node, next| {
            println!("{} -> {}", node.node().value, next.node().value);
        });
        println!();
        node70.dfs(|node, next| {
            println!("{} -> {}", node.node().value, next.node().value);
        });
        node70.rot_parent();
        println!();
        node70.dfs(|node, next| {
            println!("{} -> {}", node.node().value, next.node().value);
        });
        println!();
        println!("{}", Tree::from(node));
        println!("{}", Tree::from(node70));

        node70.debug_ancestor();
    }

    struct Add;
    impl Query for Add {
        type Elem = u32;
        const IDENT: Self::Elem = 0;
        fn query(&self, a: &Self::Elem, b: &Self::Elem) -> Self::Elem {
            *a + *b
        }
    }
    impl Commutative for Add {}

    #[test]
    fn splay_test() {
        let op = Add;
        let node = (0..10u32)
            .map(|i| NodeRef::new(Node::<_, Add>::new(i)))
            .collect::<Vec<_>>();
        node[3].link_child(Right, Some(node[4]));
        node[3].link_child(Left, Some(node[5]));
        node[3].update_from_child(&op);
        node[2].link_child(Right, Some(node[3]));
        node[2].link_child(Left, Some(node[6]));
        node[2].update_from_child(&op);
        node[1].link_child(Right, Some(node[2]));
        node[1].update_from_child(&op);
        node[7].link_child(Right, Some(node[9]));
        node[7].link_child(Left, Some(node[8]));
        node[7].update_from_child(&op);
        node[0].link_child(Right, Some(node[7]));
        node[0].link_child(Left, Some(node[1]));
        node[0].update_from_child(&op);

        println!("{}", Tree::from(node[0]));
        for &n in &node {
            println!("{{{}, {}}}", n.val(), n.query());
        }
        println!();
        node[4].splay(&op);
        node[4].update_from_child(&op);
        for &n in &node {
            println!("--------------------");
            println!("{}", Tree::from(n));
            println!("{{{}, {}}}", n.val(), n.query());
        }
        println!();

        node[9].splay(&op);
        node[9].update_from_child(&op);
        for &n in &node {
            println!("--------------------");
            println!("{}", Tree::from(n));
            println!("{{{}, {}}}", n.val(), n.query());
        }
        println!();
    }

    #[test]
    fn expose_test() {
        let op = Add;
        let nodes = (0u32..10)
            .map(|i| NodeRef::new(Node::<_, Add>::new(i)))
            .collect::<Vec<_>>();
        nodes[0].link(nodes[1], &op);
        nodes[1].link(nodes[2], &op);
        nodes[2].link(nodes[4], &op);
        nodes[3].link(nodes[4], &op);
        nodes[5].link(nodes[4], &op);
        nodes[6].link(nodes[1], &op);
        nodes[7].link(nodes[6], &op);
        nodes[8].link(nodes[1], &op);

        for &nodei in &nodes[0..9] {
            nodei.expose(&op);
            println!("--------------------------");
            println!("{}", Tree::from(nodei));
            println!("query: {}", nodei.update_and_get_query(&op));
        }
        nodes[1].cut(&op);

        for &nodei in &nodes[0..9] {
            nodei.expose(&op);
            println!("--------------------------");
            println!("{}", Tree::from(nodei));
            println!("query: {}", nodei.update_and_get_query(&op));
        }
    }
}
