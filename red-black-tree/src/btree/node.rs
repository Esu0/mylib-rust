use std::{
    alloc::{self, handle_alloc_error, Layout},
    marker::PhantomData,
    mem::{size_of, MaybeUninit},
    ptr::{self, NonNull},
};

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

pub struct LeafNode<K, V> {
    len: u8,
    keys: [MaybeUninit<K>; CAPACITY],
    vals: [MaybeUninit<V>; CAPACITY],
    _align_marker: [usize; 0],
}
pub type BoxedNode<K, V> = NonNull<LeafNode<K, V>>;

#[repr(C)]
pub struct InternalNode<K, V> {
    data: LeafNode<K, V>,
    edges: [MaybeUninit<BoxedNode<K, V>>; CAPACITY + 1],
}

pub struct NodeRef<BorrowType, K, V, Type> {
    height: usize,
    node: NonNull<LeafNode<K, V>>,
    _marker: PhantomData<(BorrowType, Type)>,
}

pub struct Handle<Node, Type> {
    node: Node,
    idx: usize,
    _marker: PhantomData<Type>,
}

pub type Root<K, V> = NodeRef<marker::Owned, K, V, marker::LeafOrInternal>;

impl<'a, K: 'a, V: 'a, Type> Copy for NodeRef<marker::Immut<'a>, K, V, Type> {}
impl<'a, K: 'a, V: 'a, Type> Clone for NodeRef<marker::Immut<'a>, K, V, Type> {
    fn clone(&self) -> Self {
        *self
    }
}

unsafe impl<BorrowType, K: Sync, V: Sync, Type> Sync for NodeRef<BorrowType, K, V, Type> {}
unsafe impl<K: Sync, V: Sync, Type> Send for NodeRef<marker::Immut<'_>, K, V, Type> {}
unsafe impl<K: Send, V: Send, Type> Send for NodeRef<marker::Mut<'_>, K, V, Type> {}
unsafe impl<K: Send, V: Send, Type> Send for NodeRef<marker::ValMut<'_>, K, V, Type> {}
unsafe impl<K: Send, V: Send, Type> Send for NodeRef<marker::Owned, K, V, Type> {}
unsafe impl<K: Send, V: Send, Type> Send for NodeRef<marker::Dying, K, V, Type> {}

impl<K, V> LeafNode<K, V> {
    /// 新しい`LeafNode`をインプレースに初期化する。
    unsafe fn init(this: *mut Self) {
        ptr::addr_of_mut!((*this).len).write(0);
    }

    fn new() -> Box<Self> {
        unsafe {
            let mut leaf = new_uninit_box();
            LeafNode::init(leaf.as_mut_ptr());
            assume_init_box(leaf)
        }
    }
}

impl<K, V> InternalNode<K, V> {
    /// 新しく`Box<InternalNode<K, V>>`を作成する。
    ///
    /// # Safety
    /// "internal node"の不変条件として、少なくとも一つの有効な辺を持つという
    /// ものがある。この関数はそのような辺のセットアップをしないことに注意する。
    unsafe fn new() -> Box<Self> {
        let mut node = new_uninit_box::<Self>();
        LeafNode::init(ptr::addr_of_mut!((*node.as_mut_ptr()).data));
        assume_init_box(node)
    }
}

impl<K, V> NodeRef<marker::Owned, K, V, marker::Leaf> {
    pub fn new_leaf() -> Self {
        Self::from_new_leaf(LeafNode::new())
    }

    fn from_new_leaf(leaf: Box<LeafNode<K, V>>) -> Self {
        NodeRef {
            height: 0,
            node: Box::leak(leaf).into(),
            _marker: PhantomData,
        }
    }
}

impl<K, V> NodeRef<marker::Owned, K, V, marker::Internal> {
    pub fn new_internal(child: Root<K, V>) -> Self {
        let mut new_node = unsafe { InternalNode::new() };
        new_node.edges[0].write(child.node);
        unsafe { Self::from_new_internal(new_node, child.height + 1)}
    }

    /// # Safety
    /// `height`は0より大きい
    unsafe fn from_new_internal(internal: Box<InternalNode<K, V>>, height: usize) -> Self {
        debug_assert!(height > 0);
        let node = NonNull::from(Box::leak(internal)).cast();
        NodeRef {
            height,
            node,
            _marker: PhantomData,
        }
    }
}

impl<BorrowType, K, V> NodeRef<BorrowType, K, V, marker::Internal> {
    /// もともとは`NodeRef`の`parent`フィールドから親の`NodeRef`を取得するための
    /// メソッド。現在の実装では`parent`フィールドは存在しないので、使う予定はない。
    #[allow(dead_code)]
    fn from_internal(node: NonNull<InternalNode<K, V>>, height: usize) -> Self {
        debug_assert!(height > 0);
        NodeRef {
            height,
            node: node.cast(),
            _marker: PhantomData,
        }
    }

    /// `internal node`のデータを外に公開する。
    /// 
    /// このノードへのほかの参照を無効化しないように、生ポインタで返す。
    fn as_internal_ptr(this: &Self) -> *mut InternalNode<K, V> {
        this.node.as_ptr() as *mut InternalNode<K, V>
    }
}

impl<'a, K, V> NodeRef<marker::Mut<'a>, K, V, marker::Internal> {
    /// `internal node`のデータの排他的な参照を返す。
    fn as_internal_mut(&mut self) -> &mut InternalNode<K, V> {
        unsafe { &mut *NodeRef::as_internal_ptr(self) }
    }
}

impl<BorrowType, K, V, Type> NodeRef<BorrowType, K ,V, Type> {
    /// ノードの長さを返す。これはキーと値のペアの数である。
    /// 辺の数は`len() + 1`である。
    /// この関数は安全ではあるが、unsafeなコードから生成された排他参照を無効化する
    /// ことに注意する。
    pub fn len(&self) -> usize {
        unsafe { (*Self::as_leaf_ptr(self)).len as usize }
    }

    /// ノードの高さを返す。葉ノードの高さは0である。
    pub fn height(&self) -> usize {
        self.height
    }

    /// 同じノードへの一時的な共有参照を返す。
    pub fn reborrow(&self) -> NodeRef<marker::Immut<'_>, K, V, Type> {
        NodeRef { height: self.height, node: self.node, _marker: PhantomData }
    }

    /// "leaf node"あるいは"internal node"のデータを外に公開する。
    /// 
    /// このノードへのほかの参照を無効化しないように、生ポインタで返す。
    fn as_leaf_ptr(this: &Self) -> *mut LeafNode<K, V> {
        this.node.as_ptr()
    }

    fn eq(&self, other: &Self) -> bool {
        let Self { node, height, _marker } = self;
        if node.eq(&other.node) {
            debug_assert_eq!(*height, other.height);
            true
        } else {
            false
        }
    }
}

impl<'a, K: 'a, V: 'a, Type> NodeRef<marker::Immut<'a>, K, V, Type> {
    fn into_leaf(self) -> &'a LeafNode<K, V> {
        unsafe { &*NodeRef::as_leaf_ptr(&self) }
    }
}

impl<BorrowType: marker::BorrowType, K, V, Type> NodeRef<BorrowType, K, V, Type> {
    pub fn first_edge(self) -> Handle<Self, marker::Edge> {
        unsafe { Handle::new_edge(self, 0) }
    }

    pub fn last_edge(self) -> Handle<Self, marker::Edge> {
        let len = self.len();
        unsafe { Handle::new_edge(self, len) }
    }

    /// `self`の長さが0であるとき、panicすることに注意
    pub fn first_kv(self) -> Handle<Self, marker::KV> {
        let len = self.len();
        assert!(len > 0);
        unsafe { Handle::new_kv(self, 0) }
    }
    
    /// `self`の長さが0であるとき、panicすることに注意
    pub fn last_kv(self) -> Handle<Self, marker::KV> {
        let len = self.len();
        assert!(len > 0);
        unsafe { Handle::new_kv(self, len - 1) }
    }
}

impl<BorrowType, K, V, NodeType> Handle<NodeRef<BorrowType, K, V, NodeType>, marker::Edge> {
    /// edgeに対する新しいhandleを作成する。
    /// 呼び出し側は`idx <= node.len()`であることを保証しなければならない。
    pub unsafe fn new_edge(node: NodeRef<BorrowType, K, V, NodeType>, idx: usize) -> Self {
        debug_assert!(idx <= node.len());
        Handle {
            node,
            idx,
            _marker: PhantomData,
        }
    }
}

impl<BorrowType, K, V, NodeType> Handle<NodeRef<BorrowType, K, V, NodeType>, marker::KV> {
    pub unsafe fn new_kv(node: NodeRef<BorrowType, K, V, NodeType>, idx: usize) -> Self {
        debug_assert!(idx < node.len());
        Handle {
            node,
            idx,
            _marker: PhantomData,
        }
    }
}
pub mod marker {
    use std::marker::PhantomData;

    pub enum Leaf {}
    pub enum Internal {}
    pub enum LeafOrInternal {}

    pub enum Owned {}
    pub enum Dying {}
    pub enum DormantMut {}
    pub struct Immut<'a>(PhantomData<&'a ()>);
    pub struct Mut<'a>(PhantomData<&'a mut ()>);
    pub struct ValMut<'a>(PhantomData<&'a mut ()>);

    pub trait BorrowType {
        const TRAVERSAL_PERMIT: bool = true;
    }

    impl BorrowType for Owned {
        const TRAVERSAL_PERMIT: bool = false;
    }
    impl BorrowType for Dying {}
    impl BorrowType for DormantMut {}
    impl<'a> BorrowType for Immut<'a> {}
    impl<'a> BorrowType for Mut<'a> {}
    impl<'a> BorrowType for ValMut<'a> {}

    pub enum KV {}
    pub enum Edge {}
}

/// 初期化済みの要素とそれに続く一つの未初期化の要素で構成されるスライスに要素を挿入する。
/// 挿入位置の後ろの要素は一つずつ後ろにずらされる。
///
/// # Safety
/// スライスが`idx`以上の要素をもつこと
unsafe fn slice_insert<T>(slice: &mut [MaybeUninit<T>], idx: usize, val: T) {
    let len = slice.len();
    debug_assert!(idx < len);
    let slice_ptr = slice.as_mut_ptr();
    if len > idx + 1 {
        ptr::copy(slice_ptr.add(idx), slice_ptr.add(idx + 1), len - idx - 1);
    }
    (*slice_ptr.add(idx)).write(val);
}

/// 要素をスライスから削除し、その要素を返す。
/// このとき、削除位置の後ろの要素は一つずつ前にずらされ、後ろの未初期化の要素が一つ増える。
///
/// # Safety
/// スライスが`idx`個以上の要素をもつこと
unsafe fn slice_remove<T>(slice: &mut [MaybeUninit<T>], idx: usize) -> T {
    let len = slice.len();
    debug_assert!(idx < len);
    let slice_ptr = slice.as_mut_ptr();
    let val = (*slice_ptr.add(idx)).assume_init_read();
    ptr::copy(slice_ptr.add(idx + 1), slice_ptr.add(idx), len - idx - 1);
    val
}

/// スライスの要素を左に`distance`だけシフトする。
///
/// # Safety
/// スライスが`distance`個以上の要素をもつこと
unsafe fn slice_shl<T>(slice: &mut [MaybeUninit<T>], distance: usize) {
    let slice_ptr = slice.as_mut_ptr();
    ptr::copy(slice_ptr.add(distance), slice_ptr, slice.len() - distance);
}

/// スライスの要素を右に`distance`だけシフトする。
///
/// # Safety
/// スライスが`distance`個以上の要素をもつこと
unsafe fn slice_shr<T>(slice: &mut [MaybeUninit<T>], distance: usize) {
    let slice_ptr = slice.as_mut_ptr();
    ptr::copy(slice_ptr, slice_ptr.add(distance), slice.len() - distance);
}

/// スライスの全ての初期化済みの要素をムーブする。
/// `dst.copy_from_slice(src)`のような動作をするが、`T: Copy`を要求しない。
///
/// # Panics
/// 二つのスライスの長さが異なるとき
fn move_to_slice<T>(src: &mut [MaybeUninit<T>], dst: &mut [MaybeUninit<T>]) {
    assert!(src.len() == dst.len());
    unsafe {
        ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }
}

fn new_uninit_box<T>() -> Box<MaybeUninit<T>> {
    let ptr = if size_of::<T>() == 0 {
        NonNull::dangling()
    } else {
        let layout = Layout::new::<MaybeUninit<T>>();
        unsafe {
            NonNull::new(alloc::alloc(layout))
                .unwrap_or_else(|| handle_alloc_error(layout))
                .cast()
        }
    };
    unsafe { Box::from_raw(ptr.as_ptr()) }
}

unsafe fn assume_init_box<T>(val: Box<MaybeUninit<T>>) -> Box<T> {
    let ptr = Box::into_raw(val) as *mut T;
    unsafe { Box::from_raw(ptr) }
}
