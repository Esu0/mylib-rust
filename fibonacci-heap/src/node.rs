use std::num::NonZeroUsize;

#[derive(Debug)]
pub(super) struct NodeArray<K, T> {
    nodes: Vec<Option<Node<K, T>>>,
}

impl<K, T> NodeArray<K, T> {
    pub(super) const fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub(super) fn get(&self, index: NodeRef) -> &Node<K, T> {
        unsafe {
            self.nodes
                .get_unchecked(index.0.get())
                // .get(index.0.get())
                // .unwrap()
                .as_ref()
                .unwrap_unchecked()
                // .unwrap()
        }
    }

    pub(super) fn get_mut(&mut self, index: NodeRef) -> &mut Node<K, T> {
        unsafe {
            self.nodes
                .get_unchecked_mut(index.0.get())
                .as_mut()
                .unwrap_unchecked()
        }
    }

    pub(super) unsafe fn destroy(&mut self, index: NodeRef) -> Node<K, T> {
        self.nodes
            .get_unchecked_mut(index.0.get())
            .take()
            .unwrap_unchecked()
    }

    pub(super) fn push_new_node(&mut self, node: Node<K, T>) -> NodeRef {
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

    pub(super) fn insert_new_node_right(&mut self, key: K, value: T, node: NodeRef) -> NodeRef {
        let right = self.get(node).right;
        let new_node = Node::new(key, value, node, right);
        let new_node_ref = self.push_new_node(new_node);
        self.get_mut(node).right = new_node_ref;
        self.get_mut(right).left = new_node_ref;
        new_node_ref
    }

    pub(super) unsafe fn push_cyclic_node(&mut self, key: K, value: T) -> NodeRef {
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

    pub(super) fn join_child(&mut self, parent: NodeRef, node: NodeRef) {
        let parent_mut = self.get_mut(parent);
        if let Some(child) = parent_mut.child {
            self.insert_right(child, node);
            self.get_mut(parent).degree += 1;
        } else {
            parent_mut.child = Some(node);
            parent_mut.degree = 1;
            self.link_cyclic(node);
        }
        self.get_mut(node).parent = Some(parent);
    }

    pub(super) fn link_cyclic(&mut self, node: NodeRef) {
        let node_mut = self.get_mut(node);
        node_mut.right = node;
        node_mut.left = node;
    }

    pub(super) fn insert_right(&mut self, position: NodeRef, node: NodeRef) {
        let right = self.get(position).right;
        let node_mut = self.get_mut(node);
        node_mut.left = position;
        node_mut.right = right;
        self.get_mut(position).right = node;
        self.get_mut(right).left = node;
    }

    pub(super) fn get_checked(&self, index: NonZeroUsize) -> Option<&Node<K, T>> {
        self.nodes.get(index.get()).and_then(|node| node.as_ref())
    }

    pub(super) fn get_checked_mut(&mut self, index: NonZeroUsize) -> Option<&mut Node<K, T>> {
        self.nodes.get_mut(index.get()).and_then(|node| node.as_mut())
    }

    fn get_node_ref(&self, index: NonZeroUsize) -> Option<(&Node<K, T>, NodeRef)> {
        self.nodes.get(index.get()).and_then(|node| node.as_ref().map(|node| (node, NodeRef(index))))
    }

    fn cut_recursive(&mut self, node: NodeRef, parent: NodeRef) -> NodeRef {
        let mut node = node;
        let mut parent = parent;
        // let mut node_list_tail = node;
        let node_mut = self.get_mut(node);
        let mut old_left = node_mut.left;
        node_mut.parent = None;
        node_mut.damaged = false;
        while {
            let old_right = self.get(node).right;

            let parent_mut = self.get_mut(parent);
            if old_left == node {
                parent_mut.child = None;
                parent_mut.degree = 0;
            } else {
                parent_mut.child = Some(old_left);
                parent_mut.degree -= 1;
                self.get_mut(old_left).right = old_right;
                self.get_mut(old_right).left = old_left;
            }
            self.get(parent).damaged
        } {
            self.get_mut(node).right = parent;
            let node_mut = self.get_mut(parent);
            old_left = node_mut.left;
            node_mut.left = node;
            debug_assert!(node_mut.parent.is_some());
            parent = unsafe { node_mut.parent.unwrap_unchecked() };
            node_mut.parent = None;
            node_mut.damaged = false;
            node = parent;
        }
        let parent_mut = self.get_mut(parent);
        if parent_mut.parent.is_some() {
            parent_mut.damaged = true;
        }
        node
    }
}

impl<K: Ord, T> NodeArray<K, T> {
    pub(super) unsafe fn remove_max(&mut self, max: NodeRef) -> (Option<NodeRef>, K, T) {
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
            // self.get_mut(tree).degree = degree;
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
                // self.get_mut(tree).degree = degree;
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

    pub(super) fn merge_tree(&mut self, tree: NodeRef, other: NodeRef) -> NodeRef {
        let (min, max) = if self.get(tree).key < self.get(other).key {
            (tree, other)
        } else {
            (other, tree)
        };
        self.join_child(max, min);
        max
    }

    pub(super) fn increase_key_force_checked(&mut self, max: NodeRef, node: NonZeroUsize, key: K) -> Result<(K, NodeRef), K> {
        if let Some((node_ref, node)) = self.get_node_ref(node){
            // let old_key = std::mem::replace(&mut node_ref.key, key);
            let update_max = self.get(max).key < key;
            let new_max = if update_max { node } else { max };
            if let Some(parent) = node_ref.parent {
                if update_max || self.get(parent).key < key {
                    let node_list_tail = self.cut_recursive(node, parent);
                    let root_mut = self.get_mut(max);
                    let right = root_mut.right;
                    root_mut.right = node;
                    self.get_mut(node).left = max;
                    self.get_mut(node_list_tail).right = right;
                    self.get_mut(right).left = node_list_tail;
                }
            }
            Ok((std::mem::replace(&mut self.get_mut(node).key, key), new_max))
        } else {
            Err(key)
        }
    }

    pub(super) fn increase_key_checked(&mut self, max: NodeRef, node: NonZeroUsize, key: K) -> Result<(K, NodeRef), IncreaseKeyError<K>> {
        if let Some((node_ref, node)) = self.get_node_ref(node){
            // let old_key = std::mem::replace(&mut node_ref.key, key);
            if node_ref.key > key {
                return Err(IncreaseKeyError::LessKey(key));
            }
            let update_max = self.get(max).key < key;
            let new_max = if update_max { node } else { max };
            if let Some(parent) = node_ref.parent {
                if update_max || self.get(parent).key < key {
                    let node_list_tail = self.cut_recursive(node, parent);
                    let root_mut = self.get_mut(max);
                    let right = root_mut.right;
                    root_mut.right = node;
                    self.get_mut(node).left = max;
                    self.get_mut(node_list_tail).right = right;
                    self.get_mut(right).left = node_list_tail;
                }
            }
            Ok((std::mem::replace(&mut self.get_mut(node).key, key), new_max))
        } else {
            Err(IncreaseKeyError::NotFound(key))
        }
    }
}

pub enum IncreaseKeyError<K> {
    NotFound(K),
    LessKey(K),
}

#[derive(Debug)]
pub(super) struct Node<K, T> {
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
    pub(super) fn new(key: K, value: T, left: NodeRef, right: NodeRef) -> Self {
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

    pub(super) fn key_value(&self) -> (&K, &T) {
        (&self.key, &self.value)
    }

    pub(super) fn key(&self) -> &K {
        &self.key
    }

    pub(super) fn value(&self) -> &T {
        &self.value
    }

    pub(super) fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub(super) fn key_value_mut(&mut self) -> (&mut K, &mut T) {
        (&mut self.key, &mut self.value)
    }
}
/// 有効なノードを参照するインデックス
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NodeRef(NonZeroUsize);

impl NodeRef {
    pub(super) const fn inner(self) -> NonZeroUsize {
        self.0
    }
}
