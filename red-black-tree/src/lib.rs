use std::borrow::Borrow;

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
}

impl<T: Ord> Node<T> {
    fn insert(this: &mut Option<Box<Self>>, value: T) {
        if let (Color::Red, Color::Red, _) = Self::insert_rec(this, value) {
            this.as_mut().unwrap().color = Color::Black;
        }
    }

    fn insert_rec(this: &mut Option<Box<Self>>, value: T) -> (Color, Color, Direction) {
        if let Some(root) = this {
            match value.cmp(&root.value) {
                std::cmp::Ordering::Less => {
                    let (col1, col2, dir) = Self::insert_rec(&mut root.left, value);
                    if col1 == Color::Red && col2 == Color::Red {
                        debug_assert_eq!(root.color, Color::Black);
                        if root.right.as_ref().is_some_and(|r| r.color == Color::Red) {
                            root.color = Color::Red;
                            root.rank += 1;
                            root.left.as_mut().unwrap().color = Color::Black;
                            root.right.as_mut().unwrap().color = Color::Black;
                            (Color::Red, Color::Black, Direction::Right)
                        } else {
                            if dir == Direction::Right {
                                root.left.as_mut().unwrap().rotl();
                            }
                            root.rotr();
                            debug_assert_eq!(root.color, Color::Red);
                            root.color = Color::Black;
                            root.right.as_mut().unwrap().color = Color::Red;
                            (Color::Black, Color::Red, Direction::Left)
                        }
                    } else {
                        (root.color, col1, Direction::Left)
                    }
                }
                std::cmp::Ordering::Equal => (Color::Black, Color::Black, Direction::Left),
                std::cmp::Ordering::Greater => {
                    let (col1, col2, dir) = Self::insert_rec(&mut root.right, value);
                    if col1 == Color::Red && col2 == Color::Red {
                        debug_assert_eq!(root.color, Color::Black);
                        if root.left.as_ref().is_some_and(|l| l.color == Color::Red) {
                            root.color = Color::Red;
                            root.rank += 1;
                            root.left.as_mut().unwrap().color = Color::Black;
                            root.right.as_mut().unwrap().color = Color::Black;
                            (Color::Red, Color::Black, Direction::Left)
                        } else {
                            if dir == Direction::Left {
                                root.right.as_mut().unwrap().rotr();
                            }
                            root.rotl();
                            debug_assert_eq!(root.color, Color::Red);
                            root.color = Color::Black;
                            root.left.as_mut().unwrap().color = Color::Red;
                            (Color::Black, Color::Red, Direction::Right)
                        }
                    } else {
                        (root.color, col1, Direction::Right)
                    }
                }
            }
        } else {
            *this = Some(Box::new(Node {
                value,
                left: None,
                right: None,
                color: Color::Red,
                rank: 1,
            }));
            (Color::Red, Color::Black, Direction::Left)
        }
    }

    fn remove<Q>(this: &mut Option<Box<Self>>, key: &Q) -> Option<T>
    where
        Q: ?Sized + Ord,
        T: Borrow<Q>,
    {
        todo!()
    }

    fn remove_rec<Q>(this: &mut Option<Box<Self>>, key: &Q) -> Option<T>
    where
        Q: ?Sized + Ord,
        T: Borrow<Q>,
    {
        if let Some(root) = this {
            match key.cmp(root.value.borrow()) {
                std::cmp::Ordering::Less => {
                    let value = Self::remove_rec(&mut root.left, key);
                    if value.is_some() {
                        todo!()
                    } else {
                        todo!()
                    }
                }
                std::cmp::Ordering::Equal => {
                    if let Some(m) = Self::remove_min(&mut root.right) {
                        // Some(std::mem::replace(&mut root.value, m))
                        todo!()
                    } else {
                        todo!()
                    }
                }
                std::cmp::Ordering::Greater => {
                    let value = Self::remove_rec(&mut root.right, key);
                    if value.is_some() {
                        todo!()
                    } else {
                        todo!()
                    }
                }
            }
        } else {
            None
        }
    }

    fn remove_min(this: &mut Option<Box<Self>>) -> Option<(T, bool)> {
        if let Some(root) = this {
            if let Some((value, flg)) = Self::remove_min(&mut root.left) {
                if flg {
                    let root = if root.right.as_ref().is_some_and(|r| r.color == Color::Red) {
                        root.right.as_mut().unwrap().color = Color::Black;
                        debug_assert_eq!(root.color, Color::Black);
                        root.color = Color::Red;
                        root.rotl();
                        root.left.as_mut().unwrap()
                    } else if root.color == Color::Red
                        && root.right.as_ref().is_some_and(|r| {
                            !r.left.as_ref().is_some_and(|l| l.color == Color::Red)
                                && !r.right.as_ref().is_some_and(|r| r.color == Color::Red)
                        })
                    {
                        root.right.as_mut().unwrap().color = Color::Red;
                        root.rank -= 1;
                        return Some((value, true));
                    } else {
                        root
                    };
                    
                    todo!()
                } else {
                    Some((value, false))
                }
                // drop(root);
            } else {
                let right = root.right.take();
                if root.color == Color::Black {
                    let value = std::mem::replace(this, right).unwrap().value;
                    if this.as_ref().is_some_and(|r| r.color == Color::Red) {
                        this.as_mut().unwrap().color = Color::Black;
                        Some((value, false))
                    } else {
                        Some((value, true))
                    }
                } else {
                    Some((std::mem::replace(this, right).unwrap().value, false))
                }
                // drop(root);
            }
        } else {
            None
        }
    }
}

impl<T> RedBlackTree<T> {
    pub const fn new() -> Self {
        Self { root: None }
    }
}

impl<T: Ord> RedBlackTree<T> {
    pub fn insert(&mut self, value: T) {
        Node::insert(&mut self.root, value);
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
