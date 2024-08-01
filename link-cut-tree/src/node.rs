#![allow(dead_code)]
use std::{
    collections::HashSet,
    fmt,
    hash::Hash,
    marker::PhantomData,
    ptr::{addr_of, addr_of_mut, NonNull},
};

pub trait Forest {
    type NodeRef: Copy + Eq;

    fn child(&self, node: Self::NodeRef, dir: Direction) -> Option<Self::NodeRef>;
    fn parent(&self, node: Self::NodeRef) -> Option<Self::NodeRef>;
    fn set_child(&mut self, node: Self::NodeRef, dir: Direction, child: Option<Self::NodeRef>) -> Option<Self::NodeRef>;
    fn set_parent(&mut self, node: Self::NodeRef, parent: Option<Self::NodeRef>) -> Option<Self::NodeRef>;

    fn rot_child(&mut self, node: Self::NodeRef, child: Self::NodeRef, dir: Direction) -> Option<Self::NodeRef> {
        let old_child = self.set_child(child, dir, Some(node));
        let old_parent = self.set_parent(node, Some(child));
        self.set_child(node, dir.opposite(), old_child);
        if let Some(ch) = old_child {
            self.set_parent(ch, Some(node));
        }
        old_parent
    }
}
#[derive(Clone)]
pub struct Node<T, Q, OP> {
    value: T,
    query: Q,
    reverse: bool,
    parent: Option<NodeRef<T, Q, OP>>,
    left: Option<NodeRef<T, Q, OP>>,
    right: Option<NodeRef<T, Q, OP>>,
    _marker: PhantomData<fn() -> OP>,
}

#[repr(transparent)]
pub struct NodeRef<T, Q, OP>(NonNull<Node<T, Q, OP>>);

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

use super::query::{Commutative, Query};

impl<T, Q, OP> Clone for NodeRef<T, Q, OP> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, Q, OP> Copy for NodeRef<T, Q, OP> {}

impl<T, Q, OP> PartialEq for NodeRef<T, Q, OP> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T, Q, OP> Eq for NodeRef<T, Q, OP> {}

impl<T, Q, OP> Hash for NodeRef<T, Q, OP> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }

    fn hash_slice<H: std::hash::Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        NonNull::hash_slice(
            unsafe {
                std::slice::from_raw_parts(
                    data.as_ptr() as *const NonNull<Node<T, Q, OP>>,
                    data.len(),
                )
            },
            state,
        )
    }
}

impl<T, Q, OP> Node<T, Q, OP> {
    pub const fn new(value: T, query: Q) -> Self {
        Self {
            query,
            value,
            reverse: false,
            parent: None,
            left: None,
            right: None,
            _marker: PhantomData,
        }
    }
}

impl<T, Q, OP> Node<T, Q, OP> {
    pub fn child(&self, dir: Direction) -> Option<NodeRef<T, Q, OP>> {
        match dir {
            Left => self.left,
            Right => self.right,
        }
    }
}

impl<T, Q, OP> NodeRef<T, Q, OP> {
    pub fn new(node: Node<T, Q, OP>) -> Self {
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

    // pub fn insert_val(self, dir: Direction, value: T) -> Self
    // where
    //     T: Clone,
    // {
    //     let child = self.child(dir);
    //     let mut new_node = Node {
    //         query: value.clone(),
    //         value,
    //         reverse: false,
    //         parent: Some(self),
    //         left: None,
    //         right: None,
    //         _marker: PhantomData,
    //     };
    //     match dir {
    //         Left => new_node.left = child,
    //         Right => new_node.right = child,
    //     };
    //     let new_node_ref = NodeRef::new(new_node);
    //     if let Some(child) = child {
    //         child.set_parent(Some(new_node_ref));
    //     }
    //     self.set_child(dir, Some(new_node_ref));
    //     new_node_ref
    // }

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

    pub fn node(&self) -> &Node<T, Q, OP> {
        unsafe { self.0.as_ref() }
    }

    pub fn node_mut(&mut self) -> &mut Node<T, Q, OP> {
        unsafe { self.0.as_mut() }
    }

    pub fn val(&self) -> &T {
        unsafe { self.borrow_val() }
    }

    pub unsafe fn borrow_val<'a>(self) -> &'a T {
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

    pub fn query(&self) -> &Q {
        unsafe { self.borrow_query() }
    }

    pub unsafe fn borrow_query<'a>(self) -> &'a Q {
        unsafe {
            let ptr = addr_of!((*self.0.as_ptr()).query);
            &*ptr
        }
    }

    pub fn query_mut(&mut self) -> &mut Q {
        unsafe {
            let ptr = addr_of_mut!((*self.0.as_ptr()).query);
            &mut *ptr
        }
    }

    pub fn reverse(self) {
        unsafe {
            let reverse_ptr = addr_of_mut!((*self.0.as_ptr()).reverse);
            let left_ptr = addr_of_mut!((*self.0.as_ptr()).left);
            let right_ptr = addr_of_mut!((*self.0.as_ptr()).right);
            *reverse_ptr = !*reverse_ptr;
            let left = left_ptr.read();
            left_ptr.write(right_ptr.read());
            right_ptr.write(left);
        }
    }

    pub fn push(mut self) -> bool {
        let node = self.node_mut();
        if node.reverse {
            if let Some(left) = node.left {
                left.reverse();
            }
            if let Some(right) = node.right {
                right.reverse();
            }
            node.reverse = false;
            true
        } else {
            false
        }
    }

    pub fn push_either(mut self, dir: Direction) -> bool {
        let node = self.node_mut();
        if node.reverse {
            if let Some(child) = node.child(dir) {
                child.reverse();
            }
            node.reverse = false;
            true
        } else {
            false
        }
    }

    pub unsafe fn drop(self) {
        drop(Box::from_raw(self.0.as_ptr()));
    }
}

impl<T, Q, OP: Query<ValT = T, QValT = Q> + Commutative> NodeRef<T, Q, OP> {
    pub fn update_from_child(mut self, op: &OP) {
        let Node {
            value: ref val,
            query: query_mut,
            left,
            right,
            ..
        } = self.node_mut();
        match (*left, *right) {
            (Some(left), Some(right)) => {
                *query_mut = op.op(&op.op_left(left.query(), val), right.query())
            }
            (Some(left), None) => *self.query_mut() = op.op_left(left.query(), val),
            (None, Some(right)) => *self.query_mut() = op.op_right(val, right.query()),
            (None, None) => *self.query_mut() = op.val_to_query(val),
        };
    }

    /// selfのクエリの値は更新しない
    ///
    /// # Returns
    /// * 元のルートノードの親ノードと元のルートノード
    pub fn splay(self, op: &OP) -> (Option<Self>, Self) {
        let mut pd = self.parent_and_direction();
        let mut prev_p = self;
        while let Some((p, Some(dir1))) = pd {
            if let Some((gp, Some(dir2))) = p.parent_and_direction() {
                let dir1 = if gp.push() { dir1.opposite() } else { dir1 };
                if p.push_either(dir1.opposite()) {
                    self.reverse();
                }
                self.push();
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
                prev_p = gp;
                // self.update_from_child(query);
            } else {
                if p.push_either(dir1.opposite()) {
                    self.reverse();
                }
                self.push();
                let ret = p.rot_child(self, dir1.opposite());
                p.update_from_child(op);
                // self.update_from_child(op);
                return (ret, p);
            }
        }
        // self.update_from_child(op);
        (pd.map(|(p, _)| p), prev_p)
    }

    /// # Returns
    ///
    /// * タプルの二つ目は元のルートノード
    /// * タプルの一つ目は元のルートノードと`self`のLCA
    pub fn expose(self, op: &OP) -> (Self, Self) {
        let mut prev = None;
        let mut current = self;
        let mut prev_root;
        while let Some(p) = {
            let (p, prev_parent) = current.splay(op);
            prev_root = prev_parent;
            p
        } {
            current.push();
            current.set_child(Right, prev);
            prev = Some(current);
            current = p;
        }
        current.push();
        current.set_child(Right, prev);
        self.splay(op);
        // self.update_from_child(op);
        (current, prev_root)
    }

    pub fn cut(self, op: &OP) {
        self.expose(op);
        if let Some(node) = self.child(Left) {
            self.set_child(Left, None);
            node.set_parent(None);
        }
    }

    pub fn cut_checked(self, parent: Self, op: &OP) -> bool {
        parent.evert(op);
        self.expose(op);
        if Some(parent) == self.child(Left) && parent.child(Right).is_none() {
            self.set_child(Left, None);
            parent.set_parent(None);
            true
        } else {
            false
        }
    }

    /// nodeを親としてパスselfをくっつける
    pub fn link(self, node: Self, op: &OP) {
        self.expose(op);
        self.update_from_child(op);
        node.expose(op);
        node.link_child(Right, Some(self));
        node.update_from_child(op);
    }

    pub fn link_checked(self, parent: Self, op: &OP) -> bool {
        self.evert(op);
        if self != parent.expose(op).1 {
            self.set_parent(Some(parent));
            true
        } else {
            false
        }
    }

    pub fn update_and_get_query(&self, op: &OP) -> &Q {
        self.update_from_child(op);
        self.query()
    }

    pub fn evert(self, op: &OP) {
        self.expose(op);
        self.reverse();
    }
}

pub struct Tree<T, Q, OP>(Option<NodeRef<T, Q, OP>>);

impl<T, Q, OP> Clone for Tree<T, Q, OP> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, Q, OP> Copy for Tree<T, Q, OP> {}

impl<T: fmt::Display, Q, OP> Tree<T, Q, OP> {
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

impl<T: fmt::Display, Q, OP> fmt::Display for Tree<T, Q, OP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_rec(f, 0)
    }
}

impl<T, Q, OP> From<NodeRef<T, Q, OP>> for Tree<T, Q, OP> {
    fn from(value: NodeRef<T, Q, OP>) -> Self {
        Tree(Some(value))
    }
}

impl<T: fmt::Debug, Q, OP> NodeRef<T, Q, OP> {
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

    // #[test]
    // fn dfs_test() {
    //     let node = NodeRef::new(Node::<_, ()>::new(100u32));
    //     let node70 = node
    //         .insert_val(Left, 50)
    //         .insert_val(Left, 30)
    //         .insert_val(Right, 70);
    //     node.insert_val(Right, 45);

    //     println!("{}", Tree::from(node));
    //     println!("{}", Tree::from(node70));
    //     node.dfs(|node, next| {
    //         println!("{} -> {}", node.node().value, next.node().value);
    //     });
    //     println!();
    //     node70.dfs(|node, next| {
    //         println!("{} -> {}", node.node().value, next.node().value);
    //     });
    //     node70.rot_parent();
    //     println!();
    //     node70.dfs(|node, next| {
    //         println!("{} -> {}", node.node().value, next.node().value);
    //     });
    //     println!();
    //     println!("{}", Tree::from(node));
    //     println!("{}", Tree::from(node70));

    //     node70.debug_ancestor();
    // }

    struct Add;
    impl Query for Add {
        type ValT = u32;
        type QValT = u32;
        const IDENT: Self::QValT = 0;
        fn op(&self, a: &Self::QValT, b: &Self::QValT) -> Self::QValT {
            a + b
        }
        fn op_left(&self, a: &Self::QValT, b: &Self::ValT) -> Self::QValT {
            a + b
        }
        fn op_right(&self, a: &Self::ValT, b: &Self::QValT) -> Self::QValT {
            a + b
        }
    }
    impl Commutative for Add {}

    #[test]
    fn splay_test() {
        let op = Add;
        let node = (0..10u32)
            .map(|i| NodeRef::new(Node::<_, _, Add>::new(i, i)))
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
            .map(|i| NodeRef::new(Node::<_, _, Add>::new(i, i)))
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

    #[test]
    fn evert_test() {
        let op = Add;
        let nodes = (0u32..10)
            .map(|i| NodeRef::new(Node::<_, _, Add>::new(i, i)))
            .collect::<Vec<_>>();
        nodes[0].link(nodes[1], &op);
        nodes[1].link(nodes[2], &op);
        nodes[2].link(nodes[4], &op);
        nodes[3].link(nodes[4], &op);
        nodes[5].link(nodes[4], &op);
        nodes[6].link(nodes[1], &op);
        nodes[7].link(nodes[6], &op);
        nodes[8].link(nodes[1], &op);

        nodes[8].evert(&op);
        nodes[3].expose(&op);
        assert_eq!(*nodes[3].update_and_get_query(&op), 18);
        nodes[4].evert(&op);
        nodes[5].expose(&op);
        assert_eq!(*nodes[5].update_and_get_query(&op), 9);
        nodes[5].evert(&op);
        nodes[7].expose(&op);
        assert_eq!(*nodes[7].update_and_get_query(&op), 25);
        nodes[7].evert(&op);
        nodes[5].expose(&op);
        assert_eq!(*nodes[5].update_and_get_query(&op), 25);
        nodes[6].expose(&op);
        assert_eq!(*nodes[6].update_and_get_query(&op), 13);
        nodes[8].expose(&op);
        assert_eq!(*nodes[8].update_and_get_query(&op), 22);
        nodes[3].evert(&op);
        nodes[5].expose(&op);
        assert_eq!(*nodes[5].update_and_get_query(&op), 12);
    }

    #[test]
    fn link_cut_test() {
        let op = Add;
        let nodes = (0u32..11)
            .map(|i| NodeRef::new(Node::<_, _, Add>::new(i, i)))
            .collect::<Vec<_>>();
        let link = |n: usize, p: usize| nodes[n].link(nodes[p], &op);
        let expose = |n: usize| nodes[n].expose(&op);
        let query = |n: usize| *nodes[n].update_and_get_query(&op);
        let evert = |n: usize| nodes[n].evert(&op);
        // let cut = |n: usize| nodes[n].cut(&op);
        let cut_checked = |i: usize, j: usize| nodes[i].cut_checked(nodes[j], &op);
        let link_checked = |i: usize, j: usize| nodes[i].link_checked(nodes[j], &op);
        link(0, 3);
        link(6, 3);
        link(3, 4);
        link(10, 5);
        link(1, 5);
        link(2, 5);
        link(5, 4);
        link(8, 9);
        link(7, 9);
        link(9, 4);
        expose(0);
        assert_eq!(query(0), 7);
        expose(10);
        assert_eq!(query(10), 19);
        expose(8);
        assert_eq!(query(8), 21);
        evert(0);
        expose(5);
        assert_eq!(query(5), 12);
        expose(1);
        assert_eq!(query(1), 13);
        expose(7);
        assert_eq!(query(7), 23);
        evert(2);
        expose(1);
        assert_eq!(query(1), 8);
        assert!(cut_checked(4, 5));
        evert(2);
        expose(10);
        assert_eq!(query(10), 17);
        evert(8);
        assert!(link_checked(8, 10));
        evert(3);
        expose(5);
        assert_eq!(query(5), 39);
        expose(2);
        assert_eq!(query(2), 41);
        evert(1);
        expose(7);
        assert_eq!(query(7), 40);
        cut_checked(3, 4);
        evert(6);
        link(6, 5);
        evert(5);
        expose(0);
        assert_eq!(query(0), 14);
        expose(4);
        assert_eq!(query(4), 36);
        assert!(!link_checked(4, 5));
        assert!(!cut_checked(5, 9));
        assert!(!cut_checked(0, 6));
        assert!(cut_checked(0, 3));
        assert!(!cut_checked(0, 3));
        assert!(link_checked(0, 9));
        assert!(!link_checked(4, 0));
        unsafe {
            nodes.into_iter().for_each(|node| node.drop());
        }
    }
}
