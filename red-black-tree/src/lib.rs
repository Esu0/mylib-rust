pub mod btree;

use std::{
    borrow::Borrow,
    fmt::{self, Display},
};

#[derive(Debug)]
pub struct RedBlackTree<T> {
    root: Option<Box<Node<T>>>,
}

#[derive(Debug)]
struct Node<T> {
    value: T,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
    color: Color,
    rank: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Color {
    Red,
    Black,
}

impl Color {
    fn flip(self) -> Self {
        match self {
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        }
    }
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            left: None,
            right: None,
            color: Color::Black,
            rank: 1,
        }
    }
    // 左回転
    fn rotl(&mut self) {
        let mut right = self.right.take().unwrap();
        self.right = right.left.take();
        std::mem::swap(self, &mut *right);
        self.left = Some(right);
    }

    // 右回転
    fn rotr(&mut self) {
        let mut left = self.left.take().unwrap();
        self.left = left.right.take();
        std::mem::swap(self, &mut *left);
        self.right = Some(left);
    }

    fn merge(this: &mut Option<Box<Self>>, value: T, other: Box<Self>) {
        if this.is_some() {
            // if root.rank < other.rank {
            //     let tmp = this.take().unwrap();
            //     other.merge_rec(tmp);
            //     *this = Some(other);
            // } else {
            //     root.merge_rec(other);
            // }
            // root.merge_rec(other);
            let tmp = this.take().unwrap();
            let (mut new_root, _) = tmp.merge_rec(value, other);
            new_root.color = Color::Black;
            *this = Some(new_root);
        } else {
            *this = Some(other);
        }
    }

    fn merge_rec(mut self: Box<Self>, value: T, mut other: Box<Self>) -> (Box<Self>, bool) {
        use std::cmp::Ordering::*;
        match self.rank.cmp(&other.rank) {
            Less => {
                let (left, rr) = self.merge_rec(value, other.left.unwrap());
                let left_color = left.color;
                other.left = Some(left);
                if rr {
                    debug_assert_eq!(other.color, Color::Black);
                    let right_color = other.right.as_ref().map_or(Color::Black, |r| r.color);
                    if right_color == Color::Red {
                        other.color = Color::Red;
                        other.rank += 1;
                        other.right.as_mut().unwrap().color = Color::Black;
                        other.left.as_mut().unwrap().color = Color::Black;
                        (other, false)
                    } else {
                        other.color = Color::Black;
                        other.left.as_mut().unwrap().color = Color::Red;
                        other.rotr();
                        (other, false)
                    }
                } else {
                    let flg = left_color == Color::Red && other.color == Color::Red;
                    (other, flg)
                }
            }
            Equal => {
                assert_eq!(self.color, Color::Black);
                assert_eq!(other.color, Color::Black);
                let root = Box::new(Node {
                    value,
                    rank: self.rank + 1,
                    left: Some(self),
                    right: Some(other),
                    color: Color::Red,
                });
                (root, false)
            }
            Greater => {
                let (right, rr) = self.right.unwrap().merge_rec(value, other);
                let right_color = right.color;
                self.right = Some(right);
                if rr {
                    debug_assert_eq!(self.color, Color::Black);
                    let left_color = self.left.as_ref().map_or(Color::Black, |l| l.color);
                    if left_color == Color::Red {
                        self.color = Color::Red;
                        self.rank += 1;
                        self.left.as_mut().unwrap().color = Color::Black;
                        self.right.as_mut().unwrap().color = Color::Black;
                        (self, false)
                    } else {
                        self.color = Color::Black;
                        self.right.as_mut().unwrap().color = Color::Red;
                        self.rotl();
                        (self, false)
                    }
                } else {
                    let flg = right_color == Color::Red && self.color == Color::Red;
                    (self, flg)
                }
            }
        }
    }

    fn fmt_rec(this: &Option<Box<Self>>, f: &mut fmt::Formatter<'_>) -> fmt::Result
    where
        T: Display,
    {
        if let Some(root) = this {
            Self::fmt_rec(&root.left, f)?;
            write!(f, "{}, ", root.value)?;
            Self::fmt_rec(&root.right, f)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
}

impl<T: Ord> Node<T> {}

impl<T> RedBlackTree<T> {
    pub const fn new() -> Self {
        Self { root: None }
    }

    fn merge(&mut self, value: T, other: Self) {
        // TODO どちらかがNoneのとき挿入処理が必要
        if let Some(other) = other.root {
            Node::merge(&mut self.root, value, other);
        }
    }
}

impl<T: Ord> RedBlackTree<T> {
    pub fn insert(&mut self, value: T) {
        todo!()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        Q: ?Sized + Ord,
        T: Borrow<Q>,
    {
        let mut current = &self.root;
        while let Some(node) = current {
            match key.cmp(node.value.borrow()) {
                std::cmp::Ordering::Less => current = &node.left,
                std::cmp::Ordering::Equal => return Some(&node.value),
                std::cmp::Ordering::Greater => current = &node.right,
            }
        }
        None
    }
}

impl<T> Default for RedBlackTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Display> Display for RedBlackTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        Node::fmt_rec(&self.root, f)?;
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn insert_test() {
    //     let mut tree = RedBlackTree::new();
    //     tree.insert(3);
    //     tree.insert(2);
    //     tree.insert(10);
    //     tree.insert(1);
    //     tree.insert(4);
    //     tree.insert(5);
    //     tree.insert(6);
    //     tree.insert(14);
    //     tree.insert(13);
    //     tree.insert(13);
    //     tree.insert(13);
    //     tree.insert(14);

    //     assert_eq!(tree.get(&1), Some(&1));
    //     assert_eq!(tree.get(&2), Some(&2));
    //     assert_eq!(tree.get(&3), Some(&3));
    //     assert_eq!(tree.get(&4), Some(&4));
    //     assert_eq!(tree.get(&5), Some(&5));
    //     assert_eq!(tree.get(&6), Some(&6));
    //     assert_eq!(tree.get(&7), None);
    //     assert_eq!(tree.get(&8), None);
    //     assert_eq!(tree.get(&9), None);
    //     assert_eq!(tree.get(&10), Some(&10));
    //     assert_eq!(tree.get(&11), None);
    //     assert_eq!(tree.get(&12), None);
    //     assert_eq!(tree.get(&13), Some(&13));
    //     assert_eq!(tree.get(&14), Some(&14));
    //     assert_eq!(tree.get(&15), None);
    // }

    #[test]
    fn merge_test() {
        let mut tree = RedBlackTree::new();
        tree.merge(3, RedBlackTree::new());
        tree.merge(2, RedBlackTree::new());
        tree.merge(10, RedBlackTree::new());
        let mut tree2 = RedBlackTree::new();
        tree2.merge(1, RedBlackTree::new());
        tree2.merge(4, RedBlackTree::new());
        tree2.merge(5, RedBlackTree::new());
        tree.merge(0, tree2);
        println!("{}", tree);
    }
}
