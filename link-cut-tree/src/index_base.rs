pub mod operation;

pub struct LinkCutTree<T, Q, OP> {
    nodes: Vec<Node<T, Q>>,
    op: OP,
}

#[derive(Debug, Clone)]
struct Node<T, Q> {
    value: T,
    query: Q,
    reverse: bool,
    parent: usize,
    left: usize,
    right: usize,
}

impl<T, Q> Node<T, Q> {
    fn child(&self, dir: Direction) -> usize {
        match dir {
            Direction::Left => self.left,
            Direction::Right => self.right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Left => Right,
            Right => Left,
        }
    }
}

use operation::Operator;
use Direction::*;

impl<T, Q, OP> LinkCutTree<T, Q, OP> {
    pub const fn new(op: OP) -> Self {
        Self {
            nodes: Vec::new(),
            op,
        }
    }

    fn child(&self, i: usize, dir: Direction) -> usize {
        match dir {
            Direction::Left => self.nodes[i].left,
            Direction::Right => self.nodes[i].right,
        }
    }

    fn set_child(&mut self, i: usize, child: usize, dir: Direction) -> usize {
        let mref = match dir {
            Left => &mut self.nodes[i].left,
            Right => &mut self.nodes[i].right,
        };
        let old = *mref;
        *mref = child;
        old
    }

    fn set_parent(&mut self, i: usize, parent: usize) -> usize {
        let mref = &mut self.nodes[i].parent;
        let old = *mref;
        *mref = parent;
        old
    }

    fn link_child(&mut self, i: usize, child: usize, dir: Direction) -> (usize, usize) {
        let old_child = self.set_child(i, child, dir);
        let old_parent = self.set_parent(child, i);
        (old_child, old_parent)
    }

    fn rot_child(&mut self, i: usize, child: usize, dir: Direction) -> usize {
        let (old_child, old_parent) = self.link_child(child, i, dir);
        self.link_child(i, old_child, dir.opposite());
        self.set_parent(child, old_parent);
        old_parent
    }

    fn reverse(&mut self, i: usize) {
        let node = &mut self.nodes[i];
        node.reverse = !node.reverse;
        std::mem::swap(&mut node.left, &mut node.right);
    }

    fn push(&mut self, i: usize) -> bool {
        let node = &mut self.nodes[i];
        if node.reverse {
            node.reverse = false;
            let left = node.left;
            let right = node.right;
            if left != usize::MAX {
                self.reverse(left);
            }
            if right != usize::MAX {
                self.reverse(right);
            }
            true
        } else {
            false
        }
    }

    fn push_either(&mut self, i: usize, dir: Direction) -> bool {
        let node = &mut self.nodes[i];
        if node.reverse {
            node.reverse = false;
            let child = node.child(dir);
            if child != usize::MAX {
                self.reverse(child);
            }
            true
        } else {
            false
        }
    }

    fn direction(&self, parent: usize, child: usize) -> Option<Direction> {
        let node = &self.nodes[parent];
        if node.left == child {
            Some(Left)
        } else if node.right == child {
            Some(Right)
        } else {
            None
        }
    }

    fn parent_and_direction(&self, i: usize) -> (usize, Option<Direction>) {
        let parent = self.nodes[i].parent;
        if parent == usize::MAX {
            (usize::MAX, None)
        } else {
            (parent, self.direction(parent, i))
        }
    }
}

impl<T, Q, OP> LinkCutTree<T, Q, OP>
where
    OP: Operator<ValT = T, QValT = Q>,
{
    fn update_from_child(&mut self, i: usize) {
        let node = &self.nodes[i];
        let left = node.left;
        let right = node.right;
        let mid_query = self.op.val_to_query(&node.value);
        let new_value = match (left, right) {
            (usize::MAX, usize::MAX) => mid_query,
            (left, usize::MAX) => self.op.operate(&self.nodes[left].query, &mid_query),
            (usize::MAX, right) => self.op.operate(&mid_query, &self.nodes[right].query),
            (left, right) => self.op.operate(&self.op.operate(&self.nodes[left].query, &mid_query), &self.nodes[right].query),
        };
        self.nodes[i].query = new_value;
    }

    fn splay(&mut self, i: usize) -> (usize, usize) {
        let mut pd = self.parent_and_direction(i);
        let mut prev_parent = i;
        while let (parent, Some(dir1)) = pd {
            if let (grandparent, Some(dir2)) = self.parent_and_direction(parent) {
                let dir1 = if self.push(grandparent) { dir1.opposite() } else { dir1 };
                if self.push_either(parent, dir1.opposite()) {
                    self.reverse(i);
                }
                self.push(i);
                let next_parent = if dir1 == dir2 {
                    let tmp = self.rot_child(grandparent, parent, dir1.opposite());
                    self.rot_child(parent, i, dir1.opposite());
                    tmp
                } else {
                    self.rot_child(parent, i, dir2);
                    self.rot_child(grandparent, i, dir1)
                };
                pd.0 = next_parent;
                if next_parent != usize::MAX {
                    pd.1 = self.direction(next_parent, grandparent);
                } else {
                    pd.1 = None;
                }
                self.update_from_child(grandparent);
                self.update_from_child(parent);
                prev_parent = grandparent;
            } else {
                if self.push_either(parent, dir1.opposite()) {
                    self.reverse(i);
                }
                self.push(i);
                let next_parent = self.rot_child(parent, i, dir1.opposite());
                self.update_from_child(parent);
                return (next_parent, parent);
            }
        }
        (pd.0, prev_parent)
    }
}

impl<T, Q, OP: Default> Default for LinkCutTree<T, Q, OP> {
    fn default() -> Self {
        Self::new(OP::default())
    }
}
