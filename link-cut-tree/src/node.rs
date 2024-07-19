#![allow(dead_code)]
use std::{
    collections::HashSet,
    fmt,
    hash::Hash,
    ptr::{addr_of, addr_of_mut, NonNull},
};

#[derive(Clone)]
pub struct Node<T> {
    value: T,
    parent: Option<NodeRef<T>>,
    left: Option<NodeRef<T>>,
    right: Option<NodeRef<T>>,
}

#[repr(transparent)]
pub struct NodeRef<T>(NonNull<Node<T>>);

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

impl<T> Clone for NodeRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for NodeRef<T> {}

impl<T> PartialEq for NodeRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for NodeRef<T> {}

impl<T> Hash for NodeRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }

    fn hash_slice<H: std::hash::Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        NonNull::hash_slice(
            unsafe {
                std::slice::from_raw_parts(data.as_ptr() as *const NonNull<Node<T>>, data.len())
            },
            state,
        )
    }
}

impl<T> Node<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value,
            parent: None,
            left: None,
            right: None,
        }
    }
}

impl<T> NodeRef<T> {
    pub fn new(node: Node<T>) -> Self {
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

    /// dirの方向に木の回転を行う。childはdirの反対方向の子である必要があることに注意
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

    pub fn insert_val(self, dir: Direction, value: T) -> Self {
        let child = self.child(dir);
        let mut new_node = Node {
            value,
            parent: Some(self),
            left: None,
            right: None,
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

    pub fn node(&self) -> &Node<T> {
        unsafe { self.0.as_ref() }
    }

    pub fn splay(self) -> Option<Self> {
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
            } else {
                return p.rot_child(self, dir1.opposite());
            }
        }
        pd.map(|(p, _)| p)
    }

    pub fn expose(self) {
        let mut prev = None;
        let mut current = self;
        while let Some(p) = current.splay() {
            current.set_child(Right, prev);
            prev = Some(current);
            current = p;
        }
        current.set_child(Right, prev);
        self.splay();
    }

    pub fn cut(self) {
        self.expose();
        if let Some(node) = self.child(Left) {
            self.set_child(Left, None);
            node.set_parent(None);
        }
    }

    /// nodeを親としてパスselfをくっつける
    pub fn link(self, node: Self) {
        self.expose();
        node.expose();
        node.link_child(Right, Some(self));
    }
}

pub struct Tree<T>(Option<NodeRef<T>>);

impl<T> Clone for Tree<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Tree<T> {}

impl<T: fmt::Display> Tree<T> {
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

impl<T: fmt::Display> fmt::Display for Tree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_rec(f, 0)
    }
}

impl<T> From<NodeRef<T>> for Tree<T> {
    fn from(value: NodeRef<T>) -> Self {
        Tree(Some(value))
    }
}

impl<T: fmt::Debug> NodeRef<T> {
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
        let node = NodeRef::new(Node::new(100u32));
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

    #[test]
    fn splay_test() {
        let node = NodeRef::new(Node::new(100u32));
        let node70 = node
            .insert_val(Left, 50)
            .insert_val(Left, 30)
            .insert_val(Right, 70);
        node.insert_val(Right, 45);

        println!("{}", Tree::from(node));
        println!("{}", Tree::from(node70));
        node70.splay();
        println!("{}", Tree::from(node));
        println!("{}", Tree::from(node70));
        node70.debug_ancestor();

        node.splay();
        println!("{}", Tree::from(node));
        println!("{}", Tree::from(node70));
        node70.debug_ancestor();

        node70.dfs(|node, next| {
            println!("{} -> {}", node.node().value, next.node().value);
        });
    }

    #[test]
    fn expose_test() {
        let nodes = (0usize..10)
            .map(|i| NodeRef::new(Node::new(i)))
            .collect::<Vec<_>>();
        nodes[0].link(nodes[1]);
        nodes[1].link(nodes[2]);
        nodes[2].link(nodes[4]);
        nodes[3].link(nodes[4]);
        nodes[5].link(nodes[4]);
        nodes[6].link(nodes[1]);
        nodes[7].link(nodes[6]);
        nodes[8].link(nodes[1]);

        for &nodei in &nodes[0..9] {
            nodei.expose();
            println!("--------------------------");
            println!("{}", Tree::from(nodei));
        }
        nodes[1].cut();

        for &nodei in &nodes[0..9] {
            nodei.expose();
            println!("--------------------------");
            println!("{}", Tree::from(nodei));
        }

    }
}
