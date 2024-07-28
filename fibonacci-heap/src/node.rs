use std::num::NonZeroUsize;

#[derive(Debug)]
pub struct NodeArray<K, T> {
    nodes: Vec<Option<Node<K, T>>>,
}

impl<K, T> NodeArray<K, T> {
    pub const fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn get(&self, index: NodeRef) -> &Node<K, T> {
        unsafe {
            self.nodes
                .get_unchecked(index.0.get())
                .as_ref()
                .unwrap_unchecked()
        }
    }

    pub fn get_mut(&mut self, index: NodeRef) -> &mut Node<K, T> {
        unsafe {
            self.nodes
                .get_unchecked_mut(index.0.get())
                .as_mut()
                .unwrap_unchecked()
        }
    }

    pub unsafe fn destroy(&mut self, index: NodeRef) -> Node<K, T> {
        self.nodes
            .get_unchecked_mut(index.0.get())
            .take()
            .unwrap_unchecked()
    }

    pub fn push_new_node(&mut self, node: Node<K, T>) -> NodeRef {
        if self.nodes.is_empty() {
            self.nodes.reserve(2);
            self.nodes.push(None);
            self.nodes.push(Some(node));
            NodeRef(unsafe { NonZeroUsize::new_unchecked(1) })
        } else {
            let index = self.nodes.len();
            self.nodes.push(Some(node));
            NodeRef(unsafe { NonZeroUsize::new_unchecked(index) })
        }
    }

    pub fn insert_new_node_right(&mut self, key: K, value: T, node: NodeRef) -> NodeRef {
        let right = self.get(node).right;
        let new_node = Node::new(key, value, node, right);
        let new_node_ref = self.push_new_node(new_node);
        self.get_mut(node).right = new_node_ref;
        self.get_mut(right).left = new_node_ref;
        new_node_ref
    }

    pub unsafe fn push_cyclic_node(&mut self, key: K, value: T) -> NodeRef {
        let new_node_ref;
        if self.nodes.is_empty() {
            new_node_ref = NodeRef(NonZeroUsize::new_unchecked(1));
            self.nodes.reserve(2);
            self.nodes.push(None);
        } else {
            new_node_ref = NodeRef(NonZeroUsize::new_unchecked(self.nodes.len()));
        }
        let node = Node::new(key, value, new_node_ref, new_node_ref);
        self.nodes.push(Some(node));
        new_node_ref
    }

    /// parentのdegreeを更新しない
    pub fn join_child(&mut self, parent: NodeRef, node: NodeRef) {
        let parent_mut = self.get_mut(parent);
        if let Some(child) = parent_mut.child {
            self.insert_right(child, node);
            self.get_mut(child).parent = Some(parent);
            // self.get_mut(parent).degree += 1;
        } else {
            parent_mut.child = Some(node);
            self.link_cyclic(node);
        }
    }

    pub fn link_cyclic(&mut self, node: NodeRef) {
        let node_mut = self.get_mut(node);
        node_mut.right = node;
        node_mut.left = node;
    }

    pub fn insert_right(&mut self, position: NodeRef, node: NodeRef) {
        let right = self.get(position).right;
        let node_mut = self.get_mut(node);
        node_mut.left = position;
        node_mut.right = right;
        self.get_mut(position).right = node;
        self.get_mut(right).left = node;
    }
}

impl<K: Ord, T> NodeArray<K, T> {
    pub unsafe fn remove_max(&mut self, max: NodeRef) -> (Option<NodeRef>, K, T) {
        let node = self.destroy(max);
        debug_assert!(node.parent.is_none());
        let mut buf = [Option::<NodeRef>::None; 64];
        let mut tree = node.right;
        let mut max_degree = 0;
        while tree != max {
            let next = self.get(tree).right;
            let mut degree = self.get(tree).degree;
            while let Some(other) = buf[degree as usize].take() {
                tree = self.merge_tree(tree, other);
                degree += 1;
            }
            max_degree = max_degree.max(degree);
            buf[degree as usize] = Some(tree);
            self.get_mut(tree).degree = degree;
            tree = next;
        }

        if let Some(child) = node.child {
            let mut tree = child;
            while {
                let next = self.get(tree).right;
                let mut degree = self.get(tree).degree;
                while let Some(other) = buf[degree as usize].take() {
                    tree = self.merge_tree(tree, other);
                    degree += 1;
                }
                max_degree = max_degree.max(degree);
                buf[degree as usize] = Some(tree);
                self.get_mut(tree).degree = degree;
                tree = next;
                tree != child
            } {}
        }

        let mut trees = buf[..=max_degree as usize].iter().copied().flatten();
        (
            trees.next().map(|first| {
                let mut max_tree = first;
                let mut prev = first;
                for tree in trees {
                    if self.get(tree).key > self.get(max_tree).key {
                        max_tree = tree;
                    }
                    self.get_mut(prev).right = tree;
                    self.get_mut(tree).left = prev;
                    self.get_mut(tree).parent = None;
                    prev = tree;
                }
                self.get_mut(prev).right = first;
                self.get_mut(first).left = prev;
                self.get_mut(first).parent = None;
                max_tree
            }),
            node.key,
            node.value,
        )
    }

    /// degreeを更新しない
    pub fn merge_tree(&mut self, tree: NodeRef, other: NodeRef) -> NodeRef {
        let (min, max) = if self.get(tree).key < self.get(other).key {
            (tree, other)
        } else {
            (other, tree)
        };
        self.join_child(max, min);
        max
    }
}

#[derive(Debug)]
pub struct Node<K, T> {
    key: K,
    value: T,
    degree: u16,
    damaged: bool,
    left: NodeRef,
    right: NodeRef,
    child: Option<NodeRef>,
    parent: Option<NodeRef>,
}

impl<K, T> Node<K, T> {
    pub fn new(key: K, value: T, left: NodeRef, right: NodeRef) -> Self {
        Self {
            key,
            value,
            degree: 0,
            damaged: false,
            left,
            right,
            child: None,
            parent: None,
        }
    }

    pub fn key_value(&self) -> (&K, &T) {
        (&self.key, &self.value)
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}
/// 有効なノードを参照するインデックス
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeRef(NonZeroUsize);

impl NodeRef {
    pub const fn inner(self) -> NonZeroUsize {
        self.0
    }
}
