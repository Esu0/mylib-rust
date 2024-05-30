use node::NodeRef;

pub struct BTree<T> {
    root: Option<NodeRef<T>>,
}

impl<T> BTree<T> {
    pub const fn new() -> Self {
        Self { root: None }
    }

    pub fn merge(&mut self, other: Self, value: T) {

    }
}
mod node {
    use std::{mem::MaybeUninit, ptr::NonNull};

    pub const CAPACITY: usize = 3;

    struct FixedStack<T, const N: usize> {
        buf: [MaybeUninit<T>; N],
        len: usize,
    }

    impl<T, const N: usize> FixedStack<T, N> {
        const fn new() -> Self {
            Self {
                buf: unsafe { MaybeUninit::uninit().assume_init() },
                len: 0,
            }
        }

        fn push_unchecked(&mut self, value: T) {
            debug_assert!(self.len < N);
            unsafe {
                self.buf.get_unchecked_mut(self.len).write(value);
            }
            self.len += 1;
        }

        fn push(&mut self, value: T) {
            if self.len < N {
                self.push_unchecked(value);
            } else {
                panic!("stack overflow");
            }
        }

        fn pop(&mut self) -> Option<T> {
            if self.len == 0 {
                None
            } else {
                self.len -= 1;
                Some(unsafe { self.buf.get_unchecked(self.len).assume_init_read() })
            }
        }
    }

    pub struct Node<T> {
        keys: [MaybeUninit<T>; CAPACITY],
        edges: [MaybeUninit<NonNull<Node<T>>>; CAPACITY + 1],
        len: u8,
    }

    pub struct NodeRef<T> {
        height: usize,
        node: NonNull<Node<T>>,
    }

    impl<T> Clone for NodeRef<T> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<T> Copy for NodeRef<T> {}

    pub struct Handle<T> {
        node: NodeRef<T>,
        idx: usize,
    }

    impl<T> Node<T> {
        fn get_edge(&self, idx: usize) -> NonNull<Node<T>> {
            debug_assert!(idx < self.len as usize);
            unsafe { self.edges.get_unchecked(idx).assume_init_read() }
        }

        fn get_edges_ptr(&self) -> *const [NonNull<Node<T>>] {
            let edges_head_ptr = self.edges.as_ptr() as _;
            std::ptr::slice_from_raw_parts(edges_head_ptr, self.len as usize)
        }

        fn get_edges_ptr_mut(&mut self) -> *mut [NonNull<Node<T>>] {
            let edges_head_ptr = self.edges.as_mut_ptr() as _;
            std::ptr::slice_from_raw_parts_mut(edges_head_ptr, self.len as usize)
        }

        fn get_edges(&self) -> &[NonNull<Node<T>>] {
            unsafe { &*self.get_edges_ptr() }
        }

        fn first_edge_mut(&mut self) -> &mut NonNull<Node<T>> {
            unsafe { (*self.get_edges_ptr_mut()).first_mut().unwrap_unchecked() }
        }

        fn get_val_ptr(&self, idx: usize) -> *const T {
            debug_assert!(idx < self.len as usize);
            unsafe { self.keys.get_unchecked(idx).as_ptr() }
        }

        fn get_val_ptr_mut(&mut self, idx: usize) -> *mut T {
            debug_assert!(idx < self.len as usize);
            unsafe { self.keys.get_unchecked_mut(idx).as_mut_ptr() }
        }

        fn get_edge_ptr(&self, idx: usize) -> *const NonNull<Node<T>> {
            debug_assert!(idx < self.len as usize + 1);
            unsafe { self.edges.get_unchecked(idx).as_ptr() }
        }

        fn get_edge_ptr_mut(&mut self, idx: usize) -> *mut NonNull<Node<T>> {
            debug_assert!(idx < self.len as usize + 1);
            unsafe { self.edges.get_unchecked_mut(idx).as_mut_ptr() }
        }

        fn last_edge(&self) -> NonNull<Node<T>> {
            unsafe {
                self.edges
                    .get_unchecked(self.len as usize)
                    .assume_init_read()
            }
        }

        fn last_edge_mut(&mut self) -> &mut NonNull<Node<T>> {
            unsafe { (*self.get_edges_ptr_mut()).last_mut().unwrap_unchecked() }
        }

        fn new_leaf(val: T) -> Self {
            unsafe {
                let mut keys = MaybeUninit::<[MaybeUninit<_>; CAPACITY]>::uninit().assume_init();
                keys[0].write(val);
                Self {
                    keys,
                    edges: MaybeUninit::uninit().assume_init(),
                    len: 1,
                }
            }
        }

        fn new_internal(val: T, left: NonNull<Self>, right: NonNull<Self>) -> Self {
            unsafe {
                let mut keys = MaybeUninit::<[MaybeUninit<_>; CAPACITY]>::uninit().assume_init();
                keys[0].write(val);
                let mut edges =
                    MaybeUninit::<[MaybeUninit<_>; CAPACITY + 1]>::uninit().assume_init();
                edges[0].write(left);
                edges[1].write(right);
                Self {
                    keys,
                    edges,
                    len: 1,
                }
            }
        }

        fn merge(left: NonNull<Self>, mid: T, right: NonNull<Self>) -> NonNull<Self> {
            let root = Box::leak(Box::new(Self::new_internal(mid, left, right))).into();
            root
        }

        /// should be leaf node
        fn pop_val(mut this: NonNull<Self>) -> (MaybeZeroLength<T>, T) {
            unsafe {
                let node_mut = this.as_mut();
                let len = node_mut.len;
                let val = node_mut.keys.get_unchecked(len as usize - 1).assume_init_read();
                node_mut.len = len - 1;
                (MaybeZeroLength(this), val)
            }
        }

        /// must be internal node
        fn pop_val_edge(mut this: NonNull<Self>) -> (MaybeZeroLength<T>, T, NonNull<Self>) {
            unsafe {
                let node_mut = this.as_mut();
                let len = node_mut.len;
                let edge = node_mut
                    .edges
                    .get_unchecked(len as usize)
                    .assume_init_read();
                let len = len - 1;
                let val = node_mut.keys.get_unchecked(len as usize).assume_init_read();
                node_mut.len = len;
                (MaybeZeroLength(this), val, edge)
            }
        }
        
        /// must be internal node
        fn pop_val_edge_front(mut this: NonNull<Self>) -> (NonNull<Self>, T, MaybeZeroLength<T>) {
            unsafe {
                let node_mut = this.as_mut();
                let len = node_mut.len - 1;
                node_mut.len = len;
                let vals_head = this.as_mut().keys.as_mut_ptr() as *mut T;
                let val = vals_head.read();
                vals_head.copy_from(vals_head.add(1), len as usize);
                let edges_head = this.as_mut().edges.as_mut_ptr() as *mut NonNull<Node<T>>;
                let edge = edges_head.read();
                edges_head.copy_from(edges_head.add(1), len as usize + 1);
                (edge, val, MaybeZeroLength(this))
            }
        }
        
        fn push_leaf(mut this: NonNull<Self>, val: T) -> (NonNull<Self>, bool) {
            unsafe {
                let len = this.as_ref().len as usize + 1;
                if len > CAPACITY {
                    let new_len_left = len / 2;
                    let new_len_right = len - new_len_left - 1;

                    let val_src = this.as_mut().get_val_ptr(new_len_left);
                    let mut right: NonNull<Node<T>> = NonNull::from(Box::leak(Box::new(Node {
                        keys: MaybeUninit::uninit().assume_init(),
                        edges: MaybeUninit::uninit().assume_init(),
                        len: new_len_right as u8,
                    })));
                    let root_val = val_src.read();
                    let val_dst = right.as_mut().get_val_ptr_mut(0);
                    val_dst.copy_from_nonoverlapping(val_src.add(1), new_len_right - 1);
                    val_dst.add(new_len_right - 1).write(val);
                    this.as_mut().len = new_len_left as u8;
                    (
                        Box::leak(Box::new(Node::new_internal(root_val, this, right))).into(),
                        true,
                    )
                } else {
                    let val_dst = this.as_mut().get_val_ptr_mut(len - 1);
                    val_dst.write(val);
                    this.as_mut().len = len as u8;
                    (this, false)
                }
            }
        }
        /// thisのheightはedgeのheightに1を足した値でなければならない
        fn push_internal(mut this: NonNull<Self>, val: T, edge: NonNull<Self>) -> (NonNull<Self>, bool) {
            
            unsafe {
                let len = this.as_ref().len as usize + 1;
                if len > CAPACITY {
                    let new_len_left = len / 2;
                    let new_len_right = len - new_len_left - 1;
                    let mut right: NonNull<Node<T>> = NonNull::from(Box::leak(Box::new(Node {
                        keys: MaybeUninit::uninit().assume_init(),
                        edges: MaybeUninit::uninit().assume_init(),
                        len: new_len_right as u8,
                    })));
                    let edge_dst = right.as_mut().get_edge_ptr_mut(0);
                    let edge_src = this.as_mut().get_edge_ptr(new_len_left + 1);
                    edge_dst.add(new_len_right).copy_from(edge_dst, 1);
                    edge_dst.copy_from_nonoverlapping(edge_src, new_len_right);

                    let val_src = this.as_mut().get_val_ptr(new_len_left);
                    let root_val = val_src.read();
                    let val_dst = right.as_mut().get_val_ptr_mut(0);
                    val_dst.copy_from_nonoverlapping(val_src.add(1), new_len_right - 1);
                    val_dst.add(new_len_right - 1).write(val);
                    this.as_mut().len = new_len_left as u8;
                    (
                        Box::leak(Box::new(Node::new_internal(root_val, this, right))).into(),
                        true,
                    )
                } else {
                    let val_dst = this.as_mut().get_val_ptr_mut(len - 1);
                    val_dst.write(val);
                    let edge_dst = this.as_mut().get_edge_ptr_mut(len);
                    *edge_dst = edge;
                    this.as_mut().len = len as u8;
                    (this, false)
                }
            }
        }
    }

    struct MaybeZeroLength<T>(NonNull<Node<T>>);
    impl<T> MaybeZeroLength<T> {
        fn get_edge_ptr_mut(&self, offset: usize) -> *mut NonNull<Node<T>> {
            unsafe {
                let ptr = self.0.as_ptr();
                let edges_head_ptr = (*ptr).edges.as_mut_ptr() as *mut NonNull<Node<T>>;
                edges_head_ptr.add(offset)
            }
        }

        fn get_edge_ptr(&self, offset: usize) -> *const NonNull<Node<T>> {
            unsafe {
                let ptr = self.0.as_ptr();
                let edges_head_ptr = (*ptr).edges.as_ptr() as *const NonNull<Node<T>>;
                edges_head_ptr.add(offset)
            }
        }

        fn get_val_ptr(&self, offset: usize) -> *const T {
            unsafe {
                let ptr = self.0.as_ptr();
                let edges_head_ptr = (*ptr).keys.as_ptr() as *const T;
                edges_head_ptr.add(offset)
            }
        }

        fn get_val_ptr_mut(&self, offset: usize) -> *mut T {
            unsafe {
                let ptr = self.0.as_ptr();
                let edges_head_ptr = (*ptr).keys.as_mut_ptr() as *mut T;
                edges_head_ptr.add(offset)
            }
        }

        fn node_mut(&mut self) -> &mut Node<T> {
            unsafe { self.0.as_mut() }
        }

        /// 高さの同じ2つの木をマージする
        /// 高さの変更があった場合はtrueを返す
        fn merge(mut self, mid: T, mut right: Self, is_internal: bool) -> (NonNull<Node<T>>, bool) {
            unsafe {
                let left_len = self.0.as_ref().len as usize;
                let right_len = right.0.as_ref().len as usize;
                let sum = left_len + right_len + 1;
                if sum <= CAPACITY {
                    if is_internal {
                        let edge_dst = self.get_edge_ptr_mut(left_len + 1);
                        let edge_src = right.get_edge_ptr(0);
                        edge_dst.copy_from_nonoverlapping(edge_src, right_len + 1);
                    }
                    let val_dst = self.get_val_ptr_mut(left_len);
                    let val_src = right.get_val_ptr(0);
                    val_dst.write(mid);
                    val_dst.add(1).copy_from_nonoverlapping(val_src, right_len);
                    drop(Box::from_raw(right.0.as_ptr()));
                    self.node_mut().len = sum as u8;
                    (self.0, false)
                } else if left_len == 0 {
                    let shift = sum / 2;
                    if is_internal {
                        let edge_dst = self.get_edge_ptr_mut(1);
                        let edge_src = right.get_edge_ptr(0);
                        edge_dst.copy_from_nonoverlapping(edge_src, shift);

                        let edge_dst = right.get_edge_ptr_mut(0);
                        let edge_src = right.get_edge_ptr(shift);
                        edge_dst.copy_from(edge_src, right_len + 1 - shift);
                    }
                    let shift = shift - 1;
                    let val_dst = self.get_val_ptr_mut(0);
                    let val_src = right.get_val_ptr(0);
                    val_dst.write(mid);
                    val_dst.add(1).copy_from_nonoverlapping(val_src, shift);
                    let val_dst = right.get_val_ptr_mut(0);
                    let val_src = right.get_val_ptr(shift);
                    let root_val = val_src.read();
                    val_dst.copy_from(val_src.add(1), right_len - shift - 1);
                    self.node_mut().len = shift as u8 + 1;
                    right.node_mut().len = right_len as u8 - shift as u8 - 1;
                    (
                        Box::leak(Box::new(Node::new_internal(root_val, self.0, right.0))).into(),
                        true,
                    )
                } else if right_len == 0 {
                    let new_len_left = sum / 2;
                    let new_len_right = left_len - new_len_left;
                    if is_internal {
                        let edge_dst = right.get_edge_ptr_mut(0);
                        let edge_src = self.get_edge_ptr(new_len_left + 1);
                        edge_dst.add(new_len_right).copy_from(edge_dst, 1);
                        edge_dst.copy_from_nonoverlapping(edge_src, new_len_right);
                    }
                    let val_src = self.get_val_ptr(new_len_left);
                    let root_val = val_src.read();
                    let val_dst = right.get_val_ptr_mut(0);
                    val_dst.copy_from_nonoverlapping(val_src.add(1), new_len_right - 1);
                    val_dst.add(new_len_right - 1).write(mid);
                    self.node_mut().len = new_len_left as u8;
                    right.node_mut().len = new_len_right as u8;
                    (
                        Box::leak(Box::new(Node::new_internal(root_val, self.0, right.0))).into(),
                        true,
                    )
                } else {
                    (
                        Box::leak(Box::new(Node::new_internal(mid, self.0, right.0))).into(),
                        true,
                    )
                }
            }
        }
    }
    impl<T> NodeRef<T> {
        fn node(&self) -> &Node<T> {
            unsafe { self.node.as_ref() }
        }

        pub fn first_edge(self) -> Option<Self> {
            if self.height == 0 {
                None
            } else {
                let first_edge = self.node().get_edge(0);
                Some(Self {
                    height: self.height - 1,
                    node: first_edge,
                })
            }
        }

        pub fn last_edge(self) -> Option<Self> {
            if self.height == 0 {
                None
            } else {
                let last_edge = self.node().last_edge();
                Some(Self {
                    height: self.height - 1,
                    node: last_edge,
                })
            }
        }

        pub fn merge(left: Option<Self>, key: T, right: Option<Self>) -> Self {
            let mut stack = FixedStack::<_, 128>::new();
            let left_height = left.map_or(-1i32, |node| node.height as i32);
            let right_height = right.map_or(-1i32, |node| node.height as i32);
            if left_height < right_height {
                let right = right.unwrap();
                let mut node = right;
                while node.height as i32 - left_height > 1 {
                    stack.push(right.node);
                }
                // node.height == left_height + 1
                let (merged, flg) = if node.height == 1 {
                    Node::push_leaf(node.node, key)
                } else {
                    Node::push_internal(node.node, key, )
                }
                let mut merged = Self {
                    node: Node::merge(self.node, key, node.node),
                    height: self.height + 1,
                };
                while let Some(parent) = stack.pop() {
                    // parentのheightはmerged.heightと等しい
                    // parentの左端の子は無効なポインタ
                    let (_, val, parent) = Node::pop_val_edge_front(parent);
                    let (new_node, height_changed) =
                        MaybeZeroLength(merged.node).merge(val, parent, true);
                    merged.node = new_node;
                    if !height_changed {
                        if let Some(mut parent) = stack.pop() {
                            // parentのheightはmerged.heightより1大きい
                            // parentの左端の子は無効なポインタ
                            unsafe {
                                *parent.as_mut().first_edge_mut() = merged.node;
                            }
                            return right;
                        } else {
                            return merged;
                        }
                    }
                }
                merged
            } else {
                let mut node = self;
                while node.height > right.height {
                    stack.push(node.node);
                    node = node.last_edge().unwrap();
                }
                // node.height == right.height
                let mut merged = Self {
                    node: Node::merge(node.node, key, right.node),
                    height: node.height + 1,
                };
                while let Some(parent) = stack.pop() {
                    // parentのheightはmerged.heightと等しい
                    // parentの右端の子は無効なポインタ
                    let (parent, val, _) = Node::pop_val_edge(parent);
                    let (new_node, height_changed) =
                        parent.merge(val, MaybeZeroLength(merged.node), true);
                    merged.node = new_node;
                    if !height_changed {
                        if let Some(mut parent) = stack.pop() {
                            // parentのheightはmerged.heightより1大きい
                            // parentの右端の子は無効なポインタ
                            unsafe {
                                *parent.as_mut().last_edge_mut() = merged.node;
                            }
                            return self;
                        } else {
                            return merged;
                        }
                    }
                    merged.height += 1;
                }
                merged
            }
        }
    }
}
