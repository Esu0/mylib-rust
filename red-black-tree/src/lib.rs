use std::{
    borrow::Borrow,
    cmp, fmt,
    ptr::{addr_of, addr_of_mut, NonNull},
};

#[derive(Debug)]
pub struct RedBlackTree<T> {
    root: Option<NodeRef<T>>,
}

#[derive(Debug)]
struct Node<T> {
    value: T,
    color: Color,
    left: Option<NodeRef<T>>,
    right: Option<NodeRef<T>>,
    parent: Option<NodeRef<T>>,
}

#[derive(Debug)]
struct NodeRef<T>(NonNull<Node<T>>);

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
}

use Direction::*;

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Left => Right,
            Right => Left,
        }
    }
}

impl TryFrom<cmp::Ordering> for Direction {
    type Error = ();

    fn try_from(value: cmp::Ordering) -> Result<Self, Self::Error> {
        use cmp::Ordering::*;
        match value {
            Less => Ok(Left),
            Equal => Err(()),
            Greater => Ok(Right),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Color {
    Red,
    Black,
}

use Color::*;

impl Color {
    fn flip(self) -> Self {
        match self {
            Red => Black,
            Black => Red,
        }
    }
}

struct SearchResult<T> {
    dir: Option<Direction>,
    node: NodeRef<T>,
}

impl<T> SearchResult<T> {
    fn new(dir: Option<Direction>, node: NodeRef<T>) -> Self {
        Self { dir, node }
    }

    fn insert(self, value: T) -> Option<(NodeRef<T>, Option<NodeRef<T>>)> {
        self.dir.map(|dir| self.node.insert(value, dir))
    }
}

fn unwrap_debug<T>(value: Option<T>) -> T {
    debug_assert!(value.is_some());
    unsafe { value.unwrap_unchecked() }
}

impl<T> NodeRef<T> {
    fn node(&self) -> &Node<T> {
        unsafe { self.0.as_ref() }
    }

    fn node_mut(&mut self) -> &mut Node<T> {
        unsafe { self.0.as_mut() }
    }

    fn borrow<'a>(self) -> &'a Node<T> {
        unsafe { self.0.as_ref() }
    }

    fn child(self, dir: Direction) -> Option<Self> {
        unsafe {
            let ptr = match dir {
                Left => addr_of!((*self.0.as_ptr()).left),
                Right => addr_of!((*self.0.as_ptr()).right),
            };
            *ptr
        }
    }

    fn parent(self) -> Option<Self> {
        unsafe { (*self.0.as_ptr()).parent }
    }

    fn set_child(self, child: Option<Self>, dir: Direction) -> Option<Self> {
        unsafe {
            let ptr = match dir {
                Left => addr_of_mut!((*self.0.as_ptr()).left),
                Right => addr_of_mut!((*self.0.as_ptr()).right),
            };
            let old = *ptr;
            ptr.write(child);
            old
        }
    }

    fn set_parent(self, parent: Option<Self>) -> Option<Self> {
        unsafe {
            let ptr = addr_of_mut!((*self.0.as_ptr()).parent);
            let old = *ptr;
            ptr.write(parent);
            old
        }
    }

    fn which_child(self, parent: Self) -> Direction {
        if parent.child(Left) == Some(self) {
            Left
        } else {
            Right
        }
    }

    fn set_color(self, col: Color) {
        unsafe {
            let ptr = addr_of_mut!((*self.0.as_ptr()).color);
            ptr.write(col);
        }
    }

    fn rot(self, parent: Self, dir: Direction) -> Option<Self> {
        let old_child = self.set_child(Some(parent), dir);
        if let Some(child) = old_child {
            child.set_parent(Some(parent));
        }
        parent.set_child(old_child, dir.opposite());
        parent.set_parent(Some(self))
    }

    fn new_node_ref(value: T, color: Color) -> Self {
        Self(NonNull::from(Box::leak(Box::new(Node {
            value,
            color,
            left: None,
            right: None,
            parent: None,
        }))))
    }

    fn insert(self, value: T, dir: Direction) -> (Self, Option<Self>) {
        let new_node = Node {
            value,
            color: Color::Red,
            left: None,
            right: None,
            parent: Some(self),
        };
        let mut ret = None;
        // ループ不変条件: ノードnの色は赤であり、p-n間を除いて木全体が赤黒木の条件を満たす。すなわち、pの色は赤でも構わない。
        // p-n間は実際はポインタで連結されていないが、辺があるものとして扱う。
        let new_node_ref = Self(NonNull::from(Box::leak(Box::new(new_node))));
        let mut n = new_node_ref;
        self.set_child(Some(n), dir);
        while let Some(mut p) = n.parent() {
            let dir = n.which_child(p);
            if p.node().color == Black {
                break;
            }
            if let Some(g) = p.parent() {
                let dir2 = p.which_child(g);
                let u = g.child(dir2.opposite());
                if let Some(u) = u {
                    if u.node().color == Red {
                        u.set_color(Black);
                        p.set_color(Black);
                        g.set_color(Red);
                        n = g;
                        continue;
                    }
                }
                if dir != dir2 {
                    n.rot(p, dir2);
                    p = n;
                }
                let pp_opt = p.rot(g, dir2.opposite());
                if let Some(pp) = pp_opt {
                    pp.set_child(Some(p), g.which_child(pp));
                } else {
                    ret = Some(p);
                }
                p.set_parent(pp_opt);
                p.set_color(Black);
                g.set_color(Red);
                break;
            } else {
                p.set_color(Black);
                break;
            }
        }
        (new_node_ref, ret)
    }

    fn search(self, mut f: impl FnMut(&T) -> Option<Direction>) -> SearchResult<T> {
        let mut n = self;
        loop {
            let node = n.node();
            let dir = f(&node.value);
            if let Some(dir) = dir {
                if let Some(next_n) = n.child(dir) {
                    n = next_n;
                } else {
                    return SearchResult::new(Some(dir), n);
                }
            } else {
                return SearchResult::new(None, n);
            }
        }
    }
    fn destroy(self) -> Node<T> {
        unsafe { *Box::from_raw(self.0.as_ptr()) }
    }

    fn remove_black_leaf(self) -> (Node<T>, Option<Option<Self>>) {
        let node = self.destroy();
        let mut n = None;
        let mut p_opt = node.parent.map(|p| {
            let dir = self.which_child(p);
            p.set_child(None, dir);
            (dir, p)
        });
        let mut ret = None;
        loop {
            if let Some((dir, p)) = p_opt {
                let mut p_col = p.node().color;
                let mut s = unwrap_debug(p.child(dir.opposite()));
                let mut c = s.child(dir);
                let mut d = s.child(dir.opposite());
                let mut s_col = s.node().color;
                let mut c_col = c.map_or(Black, |c| c.node().color);
                let mut d_col = d.map_or(Black, |d| d.node().color);
                match (p_col, s_col, c_col, d_col) {
                    (Black, Black, Black, Black) => {
                        s.set_color(Red);
                        n = Some(p);
                        p_opt = p.parent().map(|pp| (p.which_child(pp), pp));
                        continue;
                    }
                    (Black, Red, _, _) => {
                        let g = s.rot(p, dir);
                        s.set_parent(g);
                        if let Some(g) = g {
                            g.set_child(Some(s), p.which_child(g));
                        } else {
                            ret = Some(Some(s));
                        }
                        s.set_color(Black);
                        p.set_color(Red);
                        s = unwrap_debug(c);
                        c = s.child(dir);
                        d = s.child(dir.opposite());
                        p_col = Red;
                        s_col = Black;
                        c_col = c.map_or(Black, |c| c.node().color);
                        d_col = d.map_or(Black, |d| d.node().color);
                    }
                    (Red, _, Black, Black) => {
                        p.set_color(Black);
                        s.set_color(Red);
                        break;
                    }
                    _ => {}
                }
                if let (Black, Red, Black) = (s_col, c_col, d_col) {
                    let c = unwrap_debug(c);
                    c.rot(s, dir.opposite());
                    p.set_child(Some(c), dir.opposite());
                    c.set_parent(Some(p));
                    c.set_color(Black);
                    s.set_color(Red);
                    s = c;
                    d = Some(s);
                    s_col = Black;
                    d_col = Red;
                }

                if let (Black, Red) = (s_col, d_col) {
                    let g = s.rot(p, dir);
                    s.set_parent(g);
                    if let Some(g) = g {
                        g.set_child(Some(s), p.which_child(g));
                    } else {
                        ret = Some(Some(s));
                    }
                    s.set_color(p_col);
                    p.set_color(Black);
                    unwrap_debug(d).set_color(Black);
                }
                break;
            } else {
                ret = Some(n);
                break;
            }
        }
        (node, ret)
    }

    fn remove_leaf(self) -> (Node<T>, Option<Option<Self>>) {
        if self.node().color == Red {
            let mut ret = None;
            let node = self.destroy();
            let p = node.parent;
            if let Some(p) = p {
                p.set_child(None, self.which_child(p));
            } else {
                ret = Some(None);
            }
            (node, ret)
        } else {
            self.remove_black_leaf()
        }
    }

    fn swap_value(mut self, mut other: Self) {
        let (n1, n2) = (self.node_mut(), other.node_mut());
        std::mem::swap(&mut n1.value, &mut n2.value);
    }

    fn remove(self) -> (Node<T>, Option<Option<Self>>) {
        let mut ret = None;
        let mut left = self.child(Left);
        let mut right = self.child(Right);
        let mut n = self;
        if let (Some(l), Some(_)) = (left, right) {
            n = l.search(|_| Some(Right)).node;
            self.swap_value(n);
            left = n.child(Left);
            right = None;
        }
        let child = left.or(right);
        if let Some(child) = child {
            let node = n.destroy();
            let p = node.parent;
            if let Some(p) = p {
                p.set_child(Some(child), n.which_child(p));
            } else {
                ret = Some(Some(child));
            }
            child.set_color(Black);
            child.set_parent(p);
            (node, ret)
        } else {
            n.remove_leaf()
        }
    }
}

impl<T: fmt::Display> NodeRef<T> {
    fn fmt_rec(self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        let node = self.node();
        if let Some(right) = node.right {
            right.fmt_rec(f, depth + 1)?;
        }
        let col_char = match node.color {
            Red => '🔴',
            Black => '⚫',
        };
        writeln!(
            f,
            "{:indent$}{}{}",
            "",
            col_char,
            node.value,
            indent = depth * 2
        )?;
        if let Some(left) = node.left {
            left.fmt_rec(f, depth + 1)?;
        }
        Ok(())
    }
}

struct Tree<T>(NodeRef<T>);
impl<T: fmt::Display> fmt::Display for Tree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt_rec(f, 0)
    }
}

impl<T> RedBlackTree<T> {
    pub const fn new() -> Self {
        Self { root: None }
    }
}

impl<T: Ord> RedBlackTree<T> {
    pub fn insert(&mut self, value: T) {
        if let Some(root) = self.root {
            let result = root.search(|v| Direction::try_from(value.cmp(v)).ok());
            if let Some((_, Some(new_root))) = result.insert(value) {
                self.root = Some(new_root);
            }
        } else {
            self.root = Some(NodeRef::new_node_ref(value, Black));
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        Q: ?Sized + Ord,
        T: Borrow<Q>,
    {
        // let mut current = &self.root;
        // while let Some(node) = current {
        //     match key.cmp(node.value.borrow()) {
        //         std::cmp::Ordering::Less => current = &node.left,
        //         std::cmp::Ordering::Equal => return Some(&node.value),
        //         std::cmp::Ordering::Greater => current = &node.right,
        //     }
        // }
        // None
        self.root.and_then(|root| {
            let result = root.search(|v| Direction::try_from(key.cmp(v.borrow())).ok());
            if result.dir.is_some() {
                None
            } else {
                Some(&result.node.borrow().value)
            }
        })
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<T>
    where
        Q: ?Sized + Ord,
        T: Borrow<Q>,
    {
        let root = self.root?;
        let result = root.search(|v| Direction::try_from(key.cmp(v.borrow())).ok());
        if result.dir.is_none() {
            let (node, new_root) = result.node.remove();
            if let Some(new_root) = new_root {
                self.root = new_root;
            }
            Some(node.value)
        } else {
            None
        }
    }
}

impl<T> Default for RedBlackTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn insert_node_test() {
    //     let root = NodeRef::new_node_ref(0, Black);
    //     root.insert(1, Left);
    //     let node2 = root.insert(2, Right);
    //     node2.insert(3, Left);
    //     let node4 = node2.insert(4, Right);
    //     node4.insert(5, Left).insert(6, Left).insert(7, Left).insert(8, Right);
    //     node2.insert(9, Right);
    //     println!("{}", Tree(root));
    // }

    #[test]
    fn insert_test() {
        let mut tree = RedBlackTree::new();
        tree.insert(3);
        tree.insert(2);
        tree.insert(10);
        tree.insert(1);
        tree.insert(4);
        tree.insert(5);
        tree.insert(6);
        tree.insert(14);
        tree.insert(13);
        tree.insert(13);
        tree.insert(13);
        tree.insert(14);

        assert_eq!(tree.get(&1), Some(&1));
        assert_eq!(tree.get(&2), Some(&2));
        assert_eq!(tree.get(&3), Some(&3));
        assert_eq!(tree.get(&4), Some(&4));
        assert_eq!(tree.get(&5), Some(&5));
        assert_eq!(tree.get(&6), Some(&6));
        assert_eq!(tree.get(&7), None);
        assert_eq!(tree.get(&8), None);
        assert_eq!(tree.get(&9), None);
        assert_eq!(tree.get(&10), Some(&10));
        assert_eq!(tree.get(&11), None);
        assert_eq!(tree.get(&12), None);
        assert_eq!(tree.get(&13), Some(&13));
        assert_eq!(tree.get(&14), Some(&14));
        assert_eq!(tree.get(&15), None);
    }

    #[test]
    fn remove_test() {
        let mut tree = RedBlackTree::new();
        tree.insert(3);
        tree.insert(2);
        tree.insert(10);
        tree.insert(1);
        tree.insert(4);
        tree.insert(5);
        tree.insert(6);
        tree.insert(14);
        println!("{}", Tree(tree.root.unwrap()));
        tree.remove(&3);
        println!("{}", Tree(tree.root.unwrap()));
        assert_eq!(tree.get(&3), None);
        tree.insert(13);
        println!("-------------------------------------");
        println!("{}", Tree(tree.root.unwrap()));
        tree.remove(&13);
        assert_eq!(tree.get(&13), None);
        println!("{}", Tree(tree.root.unwrap()));

        println!("-------------------------------------");
        println!("{}", Tree(tree.root.unwrap()));
        tree.remove(&14);
        println!("{}", Tree(tree.root.unwrap()));
        assert_eq!(tree.get(&14), None);

        tree.insert(20);
        tree.insert(15);
        tree.insert(14);
        println!("-------------------------------------");
        println!("{}", Tree(tree.root.unwrap()));
        tree.remove(&10);
        println!("{}", Tree(tree.root.unwrap()));

        for i in 10..30 {
            tree.insert(i);
        }
        println!("-------------------------------------");
        println!("{}", Tree(tree.root.unwrap()));
        tree.remove(&1);
        println!("{}", Tree(tree.root.unwrap()));
        tree.remove(&17);
        println!("{}", Tree(tree.root.unwrap()));
        tree.remove(&16);
        println!("{}", Tree(tree.root.unwrap()));
        tree.remove(&21);
        println!("{}", Tree(tree.root.unwrap()));
        tree.insert(21);
        println!("{}", Tree(tree.root.unwrap()));
    }
}
