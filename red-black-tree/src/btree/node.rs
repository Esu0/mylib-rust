#![allow(dead_code)]
use std::{
    alloc::{self, handle_alloc_error, Layout},
    marker::PhantomData,
    mem::{size_of, MaybeUninit},
    ptr::{self, NonNull},
    slice::SliceIndex,
};

use super::alloc::Allocator;

const B: usize = 3;
pub const CAPACITY: usize = 2 * B - 1;
pub const MIN_LEN_AFTER_SPLIT: usize = B - 1;
const KV_IDX_CENTER: usize = B - 1;
const EDGE_IDX_LEFT_OF_CENTER: usize = B - 1;
const EDGE_IDX_RIGHT_OF_CENTER: usize = B;

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

pub type NodeRefMut<'a, K, V, Type> = NodeRef<marker::Mut<'a>, K, V, Type>;
pub type NodeRefValMut<'a, K, V, Type> = NodeRef<marker::ValMut<'a>, K, V, Type>;
pub type NodeRefOwned<K, V, Type> = NodeRef<marker::Owned, K, V, Type>;
pub type NodeRefDying<K, V, Type> = NodeRef<marker::Dying, K, V, Type>;
pub type NodeRefDormantMut<K, V, Type> = NodeRef<marker::DormantMut, K, V, Type>;
pub type NodeRefImmut<'a, K, V, Type> = NodeRef<marker::Immut<'a>, K, V, Type>;
pub type LeafNodeRef<BorrowType, K, V> = NodeRef<BorrowType, K, V, marker::Leaf>;
pub type InternalNodeRef<BorrowType, K, V> = NodeRef<BorrowType, K, V, marker::Internal>;
pub type LeafOrInternalNodeRef<BorrowType, K, V> =
    NodeRef<BorrowType, K, V, marker::LeafOrInternal>;

pub struct Handle<Node, Type> {
    node: Node,
    idx: usize,
    _marker: PhantomData<Type>,
}

pub type HandleEdge<Node> = Handle<Node, marker::Edge>;
pub type HandleKV<Node> = Handle<Node, marker::KV>;

impl<Node: Copy, Type> Clone for Handle<Node, Type> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Node: Copy, Type> Copy for Handle<Node, Type> {}

pub type Root<K, V> = NodeRef<marker::Owned, K, V, marker::LeafOrInternal>;

pub enum ForceResult<Leaf, Internal> {
    Leaf(Leaf),
    Internal(Internal),
}

pub type ForceResultNodeRef<BorrowType, K, V> = ForceResult<
    NodeRef<BorrowType, K, V, marker::Leaf>,
    NodeRef<BorrowType, K, V, marker::Internal>,
>;

pub type ForceResultHandle<BorrowType, K, V, Type> = ForceResult<
    Handle<LeafNodeRef<BorrowType, K, V>, Type>,
    Handle<InternalNodeRef<BorrowType, K, V>, Type>,
>;

pub enum LeftOrRight<T> {
    Left(T),
    Right(T),
}

pub struct SplitResult<'a, K, V, NodeType> {
    pub left: NodeRefMut<'a, K, V, NodeType>,
    pub kv: (K, V),
    pub right: NodeRefOwned<K, V, NodeType>,
}

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

/// 容量がいっぱいのノードにキーと値を挿入するときに分割ポイントと挿入位置を計算する。
fn splitpoint(edge_idx: usize) -> (usize, LeftOrRight<usize>) {
    debug_assert!(edge_idx <= CAPACITY);
    if edge_idx < EDGE_IDX_LEFT_OF_CENTER {
        (KV_IDX_CENTER - 1, LeftOrRight::Left(edge_idx))
    } else if edge_idx == EDGE_IDX_LEFT_OF_CENTER {
        (KV_IDX_CENTER, LeftOrRight::Left(edge_idx))
    } else if edge_idx == EDGE_IDX_RIGHT_OF_CENTER {
        (KV_IDX_CENTER, LeftOrRight::Right(0))
    } else {
        (
            KV_IDX_CENTER + 1,
            LeftOrRight::Right(edge_idx - (KV_IDX_CENTER + 1 + 1)),
        )
    }
}

impl<K, V> LeafNode<K, V> {
    /// 新しい`LeafNode`をインプレースに初期化する。
    unsafe fn init(this: *mut Self) {
        ptr::addr_of_mut!((*this).len).write(0);
    }

    fn new<'a, A: Allocator<Self> + 'a>(alloc: A) -> &'a mut Self {
        unsafe {
            let leaf = alloc.allocate();
            LeafNode::init(leaf.as_ptr());
            &mut *leaf.as_ptr()
        }
    }
}

impl<K, V> InternalNode<K, V> {
    /// 新しく`Box<InternalNode<K, V>>`を作成する。
    ///
    /// # Safety
    /// "internal node"の不変条件として、少なくとも一つの有効な辺を持つという
    /// ものがある。この関数はそのような辺のセットアップをしないことに注意する。
    unsafe fn new<'a, A: Allocator<Self> + 'a>(alloc: A) -> &'a mut Self {
        let node = alloc.allocate();
        LeafNode::init(ptr::addr_of_mut!((*node.as_ptr()).data));
        &mut *node.as_ptr()
    }
}

impl<K, V> NodeRef<marker::Owned, K, V, marker::Leaf> {
    pub fn new_leaf<A>(alloc: A) -> Self
    where
        A: Allocator<LeafNode<K, V>>,
    {
        Self::from_new_leaf(LeafNode::new(alloc))
    }

    fn from_new_leaf(leaf: &mut LeafNode<K, V>) -> Self {
        NodeRef {
            height: 0,
            node: leaf.into(),
            _marker: PhantomData,
        }
    }
}

impl<K, V> NodeRef<marker::Owned, K, V, marker::Internal> {
    pub fn new_internal<A>(child: Root<K, V>, alloc: A) -> Self
    where
        A: Allocator<InternalNode<K, V>>,
    {
        let new_node = unsafe { InternalNode::new(alloc) };
        new_node.edges[0].write(child.node);
        unsafe { Self::from_new_internal(new_node, child.height + 1) }
    }

    /// # Safety
    /// `height`は0より大きい
    unsafe fn from_new_internal(internal: &mut InternalNode<K, V>, height: usize) -> Self {
        debug_assert!(height > 0);
        let node = NonNull::from(internal).cast();
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
    /// `internal node`のデータの排他的な参照を借用する。
    fn as_internal_mut(&mut self) -> &mut InternalNode<K, V> {
        unsafe { &mut *NodeRef::as_internal_ptr(self) }
    }
}

impl<BorrowType, K, V, Type> NodeRef<BorrowType, K, V, Type> {
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

    /// 同じノードへの一時的な不変ポインタを返す。
    pub fn reborrow(&self) -> NodeRef<marker::Immut<'_>, K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }

    /// "leaf node"あるいは"internal node"のデータを外に公開する。
    ///
    /// このノードへのほかの参照を無効化しないように、生ポインタで返す。
    fn as_leaf_ptr(this: &Self) -> *mut LeafNode<K, V> {
        this.node.as_ptr()
    }

    fn eq(&self, other: &Self) -> bool {
        let Self {
            node,
            height,
            _marker,
        } = self;
        if node.eq(&other.node) {
            debug_assert_eq!(*height, other.height);
            true
        } else {
            false
        }
    }
}

impl<'a, K: 'a, V: 'a, Type> NodeRef<marker::Immut<'a>, K, V, Type> {
    /// 不変な木の中間ノードまたは葉ノードにおける葉の部分のデータの共有参照を返す。
    fn into_leaf(self) -> &'a LeafNode<K, V> {
        unsafe { &*NodeRef::as_leaf_ptr(&self) }
    }

    /// ノードに格納されているキーのビューを提供する。
    pub fn keys(&self) -> &[K] {
        let leaf = self.into_leaf();
        unsafe { &*(leaf.keys.get_unchecked(..leaf.len as usize) as *const _ as *const [K]) }
    }
}

impl<K, V> NodeRef<marker::Dying, K, V, marker::LeafOrInternal> {
    /// ノードが使用するメモリ領域を解放する。
    /// 解放された後，このノードへのアクセスは未定義の動作となる。
    pub unsafe fn deallocate<A>(self, alloc: A)
    where
        A: Allocator<LeafNode<K, V>> + Allocator<InternalNode<K, V>>,
    {
        let height = self.height;
        let node = self.node;
        unsafe {
            if height > 0 {
                Allocator::<InternalNode<K, V>>::deallocate(&alloc, node.cast());
            } else {
                Allocator::<LeafNode<K, V>>::deallocate(&alloc, node.cast());
            }
        }
    }
}

impl<'a, K, V, Type> NodeRef<marker::Mut<'a>, K, V, Type> {
    /// 同じノードへの一時的な可変ポインタを返す。
    /// このメソッドは非常に危険である。危険が即座に現れない場合があるので注意
    /// する必要がある。
    ///
    /// 可変ポインタはツリー内をどこでも移動することができる。そのため、返された
    /// ポインタを使用して元のポインタをダングリングさせたり、境界外にさせたり、
    /// stacked borrow ruleの違反を引き起こすことができる。
    unsafe fn reborrow_mut(&mut self) -> NodeRef<marker::Mut<'_>, K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }

    /// 木の中間ノードまたは葉ノードにおける葉の部分のデータの排他参照を借用する。
    fn as_leaf_mut(&mut self) -> &mut LeafNode<K, V> {
        unsafe { &mut *Self::as_leaf_ptr(self) }
    }

    /// 木の中間ノードまたは葉ノードにおける葉の部分のデータの排他参照を提供する。
    fn into_leaf_mut(self) -> &'a mut LeafNode<K, V> {
        unsafe { &mut *Self::as_leaf_ptr(&self) }
    }

    /// ライフタイムが削除された休止状態のノードを返す。
    /// 休止状態のノードは後で再び起動できる。
    pub fn dormant(&self) -> NodeRef<marker::DormantMut, K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<'a, K: 'a, V: 'a, Type> NodeRef<marker::Mut<'a>, K, V, Type> {
    /// キーの格納場所への排他参照を借用して返す。
    ///
    /// # Safety
    /// `index`が0..CAPACITYの範囲にあること
    unsafe fn key_area_mut<I, Output: ?Sized>(&mut self, index: I) -> &mut Output
    where
        I: SliceIndex<[MaybeUninit<K>], Output = Output>,
    {
        unsafe {
            self.as_leaf_mut()
                .keys
                .as_mut_slice()
                .get_unchecked_mut(index)
        }
    }

    /// 値の格納場所への排他参照を借用して返す。
    ///
    /// # Safety
    /// `index`が0..CAPACITYの範囲にあること
    unsafe fn val_area_mut<I, Output: ?Sized>(&mut self, index: I) -> &mut Output
    where
        I: SliceIndex<[MaybeUninit<V>], Output = Output>,
    {
        unsafe {
            self.as_leaf_mut()
                .vals
                .as_mut_slice()
                .get_unchecked_mut(index)
        }
    }

    /// ノードの長さへの排他参照を借用して返す。
    pub fn len_mut(&mut self) -> &mut u8 {
        &mut self.as_leaf_mut().len
    }
}

impl<'a, K: 'a, V: 'a> NodeRef<marker::Mut<'a>, K, V, marker::Internal> {
    /// 辺の格納場所への排他参照を借用して返す。
    ///
    /// # Safety
    /// `index`が0..CAPACITY + 1の範囲にあること
    unsafe fn edge_area_mut<I, Output: ?Sized>(&mut self, index: I) -> &mut Output
    where
        I: SliceIndex<[MaybeUninit<BoxedNode<K, V>>], Output = Output>,
    {
        unsafe {
            self.as_internal_mut()
                .edges
                .as_mut_slice()
                .get_unchecked_mut(index)
        }
    }
}

impl<'a, K, V, Type> NodeRef<marker::ValMut<'a>, K, V, Type> {
    /// # Safety
    /// ノードが`idx`個以上の初期化された要素を持つこと
    unsafe fn into_key_val_mut_at(self, idx: usize) -> (&'a K, &'a mut V) {
        let leaf = Self::as_leaf_ptr(&self);
        let keys_head = unsafe { ptr::addr_of!((*leaf).keys) as *const MaybeUninit<K> };
        let vals_head = unsafe { ptr::addr_of_mut!((*leaf).vals) as *mut MaybeUninit<V> };
        let key = unsafe { (*keys_head.add(idx)).assume_init_ref() };
        let val = unsafe { (*vals_head.add(idx)).assume_init_mut() };
        (key, val)
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

impl<K, V, Type> NodeRef<marker::DormantMut, K, V, Type> {
    /// 最初にキャプチャしたユニークな借用に戻す。
    ///
    /// # Safety
    ///
    /// reborrowが終了している必要がある。すなわち、`new`によって返された参照と
    /// そこから派生したすべてのポインタはこれ以上使ってはならない。
    pub unsafe fn awaken<'a>(self) -> NodeRef<marker::Mut<'a>, K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<K, V, Type> NodeRef<marker::Dying, K, V, Type> {
    /// 死んだ葉または中間ノードの葉ノードの部分の排他参照を借用して返す。
    fn as_leaf_dying(&mut self) -> &mut LeafNode<K, V> {
        unsafe { &mut *Self::as_leaf_ptr(self) }
    }
}

impl<BorrowType, K, V> NodeRef<BorrowType, K, V, marker::Leaf> {
    /// このノードが葉ノードであるという静的な情報を削除する。
    pub fn forget_type(self) -> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<BorrowType, K, V> NodeRef<BorrowType, K, V, marker::Internal> {
    /// このノードが中間ノードであるという静的な情報を削除する。
    pub fn forget_type(self) -> NodeRef<BorrowType, K, V, marker::LeafOrInternal> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<BorrowType, K, V> LeafOrInternalNodeRef<BorrowType, K, V> {
    pub fn force(self) -> ForceResultNodeRef<BorrowType, K, V> {
        if self.height == 0 {
            ForceResult::Leaf(NodeRef {
                height: self.height,
                node: self.node,
                _marker: PhantomData,
            })
        } else {
            ForceResult::Internal(NodeRef {
                height: self.height,
                node: self.node,
                _marker: PhantomData,
            })
        }
    }
}

impl<K, V> LeafOrInternalNodeRef<marker::Owned, K, V> {
    /// 新しい所有された木を返す。
    pub fn new<A>(alloc: A) -> Self
    where
        A: Allocator<LeafNode<K, V>>,
    {
        NodeRef::new_leaf(alloc).forget_type()
    }

    /// 以前のルートノードへの辺のみをもつ新しい中間ノードを追加し、それを
    /// ルートノードにして返す。これにより、高さが1増加する。
    pub fn push_internal_level<A>(&mut self, alloc: A) -> NodeRef<marker::Mut<'_>, K, V, marker::Internal>
    where
        A: Allocator<InternalNode<K, V>>,
    {
        super::mem::take_mut(self, |old_root| {
            NodeRef::new_internal(old_root, alloc).forget_type()
        });
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<K, V, Type> NodeRefOwned<K, V, Type> {
    pub fn borrow_mut(&mut self) -> NodeRefMut<'_, K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }

    pub fn borrow_valmut(&mut self) -> NodeRefValMut<'_, K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }

    pub fn into_dying(self) -> NodeRefDying<K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<'a, K: 'a, V: 'a> NodeRefMut<'a, K, V, marker::Leaf> {
    /// キーと値のペアをノードの最後に追加する。追加したペアのハンドルを返す。
    ///
    /// # Safety
    /// 返されたハンドルのライフタイムが境界を超えないこと
    pub unsafe fn push_with_handle<'b>(
        &mut self,
        key: K,
        val: V,
    ) -> Handle<NodeRefMut<'b, K, V, marker::Leaf>, marker::KV> {
        let len = self.len_mut();
        let idx = *len as usize;
        assert!(idx < CAPACITY);
        *len += 1;
        unsafe {
            self.key_area_mut(idx).write(key);
            self.val_area_mut(idx).write(val);
            Handle::new_kv(
                NodeRef {
                    height: self.height,
                    node: self.node,
                    _marker: PhantomData,
                },
                idx,
            )
        }
    }

    /// [`push_with_handle`](Self::push_with_handle)と機能は同じ
    pub fn push(&mut self, key: K, val: V) -> *mut V {
        unsafe { self.push_with_handle(key, val).into_val_mut() }
    }
}

impl<'a, K: 'a, V: 'a> NodeRefMut<'a, K, V, marker::Internal> {
    /// キーと値のペアをノードの最後に追加し、その右の辺を追加する。
    pub fn push(&mut self, key: K, val: V, edge: Root<K, V>) {
        assert!(edge.height == self.height - 1);

        let len = self.len_mut();
        let idx = *len as usize;
        assert!(idx < CAPACITY);
        *len += 1;
        unsafe {
            self.key_area_mut(idx).write(key);
            self.val_area_mut(idx).write(val);
            self.edge_area_mut(idx + 1).write(edge.node);
        }
    }
}

impl<'a, K, V> NodeRefMut<'a, K, V, marker::LeafOrInternal> {
    unsafe fn cast_to_leaf_unchecked(self) -> NodeRefMut<'a, K, V, marker::Leaf> {
        debug_assert!(self.height == 0);
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }

    unsafe fn cast_to_internal_unchecked(self) -> NodeRefMut<'a, K, V, marker::Internal> {
        debug_assert!(self.height > 0);
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }
}

impl<Node, Type> Handle<Node, Type> {
    pub fn into_node(self) -> Node {
        self.node
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}

impl<BorrowType, K, V, NodeType> HandleEdge<NodeRef<BorrowType, K, V, NodeType>> {
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

    pub fn left_kv(self) -> Result<HandleKV<NodeRef<BorrowType, K, V, NodeType>>, Self> {
        if self.idx > 0 {
            Ok(unsafe { Handle::new_kv(self.node, self.idx - 1) })
        } else {
            Err(self)
        }
    }

    pub fn right_kv(self) -> Result<HandleKV<NodeRef<BorrowType, K, V, NodeType>>, Self> {
        if self.idx < self.node.len() {
            Ok(unsafe { Handle::new_kv(self.node, self.idx) })
        } else {
            Err(self)
        }
    }
}

impl<BorrowType, K, V, NodeType> HandleKV<NodeRef<BorrowType, K, V, NodeType>> {
    pub unsafe fn new_kv(node: NodeRef<BorrowType, K, V, NodeType>, idx: usize) -> Self {
        debug_assert!(idx < node.len());
        Handle {
            node,
            idx,
            _marker: PhantomData,
        }
    }

    pub fn left_edge(self) -> HandleEdge<NodeRef<BorrowType, K, V, NodeType>> {
        unsafe { Handle::new_edge(self.node, self.idx) }
    }

    pub fn right_edge(self) -> HandleEdge<NodeRef<BorrowType, K, V, NodeType>> {
        unsafe { Handle::new_edge(self.node, self.idx + 1) }
    }
}

impl<BorrowType, K, V, NodeType, HandleType> PartialEq
    for Handle<NodeRef<BorrowType, K, V, NodeType>, HandleType>
{
    fn eq(&self, other: &Self) -> bool {
        let Self { node, idx, _marker } = self;
        node.eq(&other.node) && *idx == other.idx
    }
}

impl<'a, K: 'a, V: 'a, NodeType> HandleKV<NodeRefImmut<'a, K, V, NodeType>> {
    pub fn into_kv(self) -> (&'a K, &'a V) {
        debug_assert!(self.idx < self.node.len());
        let leaf = self.node.into_leaf();
        let k = unsafe { leaf.keys.get_unchecked(self.idx).assume_init_ref() };
        let v = unsafe { leaf.vals.get_unchecked(self.idx).assume_init_ref() };
        (k, v)
    }
}

impl<'a, K: 'a, V: 'a, NodeType> HandleKV<NodeRef<marker::Mut<'a>, K, V, NodeType>> {
    pub fn key_mut(&mut self) -> &mut K {
        unsafe { self.node.key_area_mut(self.idx).assume_init_mut() }
    }

    pub fn into_val_mut(self) -> &'a mut V {
        debug_assert!(self.idx < self.node.len());
        let leaf = self.node.into_leaf_mut();
        unsafe { leaf.vals.get_unchecked_mut(self.idx).assume_init_mut() }
    }

    pub fn into_kv_mut(self) -> (&'a mut K, &'a mut V) {
        debug_assert!(self.idx < self.node.len());
        let leaf = self.node.into_leaf_mut();
        let k = unsafe { leaf.keys.get_unchecked_mut(self.idx).assume_init_mut() };
        let v = unsafe { leaf.vals.get_unchecked_mut(self.idx).assume_init_mut() };
        (k, v)
    }
}

impl<'a, K, V, NodeType> HandleKV<NodeRefValMut<'a, K, V, NodeType>> {
    pub fn into_kv_valmut(self) -> (&'a K, &'a mut V) {
        unsafe { self.node.into_key_val_mut_at(self.idx) }
    }
}

impl<'a, K: 'a, V: 'a, NodeType> HandleKV<NodeRefMut<'a, K, V, NodeType>> {
    pub fn kv_mut(&mut self) -> (&mut K, &mut V) {
        debug_assert!(self.idx < self.node.len());
        unsafe {
            let leaf = self.node.as_leaf_mut();
            let k = leaf.keys.get_unchecked_mut(self.idx).assume_init_mut();
            let v = leaf.vals.get_unchecked_mut(self.idx).assume_init_mut();
            (k, v)
        }
    }

    pub fn replace_kv(&mut self, k: K, v: V) -> (K, V) {
        let (key, val) = self.kv_mut();
        (std::mem::replace(key, k), std::mem::replace(val, v))
    }
}

impl<K, V, NodeType> HandleKV<NodeRefDying<K, V, NodeType>> {
    /// # Safety
    /// ハンドルが指すノードがまだ解放されていないこと
    pub unsafe fn into_key_val(mut self) -> (K, V) {
        debug_assert!(self.idx < self.node.len());
        let leaf = self.node.as_leaf_dying();
        unsafe {
            let k = leaf.keys.get_unchecked_mut(self.idx).assume_init_read();
            let v = leaf.vals.get_unchecked_mut(self.idx).assume_init_read();
            (k, v)
        }
    }

    /// # Safety
    /// ハンドルが指すノードがまだ解放されていないこと
    #[inline]
    pub unsafe fn drop_key_val(mut self) {
        debug_assert!(self.idx < self.node.len());
        let leaf = self.node.as_leaf_dying();
        unsafe {
            leaf.keys.get_unchecked_mut(self.idx).assume_init_drop();
            leaf.vals.get_unchecked_mut(self.idx).assume_init_drop();
        }
    }
}

impl<BorrowType, K, V, NodeType, HandleType>
    Handle<NodeRef<BorrowType, K, V, NodeType>, HandleType>
{
    pub fn reborrow(&self) -> Handle<NodeRef<marker::Immut<'_>, K, V, NodeType>, HandleType> {
        Handle {
            node: self.node.reborrow(),
            idx: self.idx,
            _marker: PhantomData,
        }
    }
}

impl<'a, K, V, NodeType, HandleType> Handle<NodeRefMut<'a, K, V, NodeType>, HandleType> {
    pub unsafe fn reborrow_mut(&mut self) -> Handle<NodeRefMut<'_, K, V, NodeType>, HandleType> {
        Handle {
            node: self.node.reborrow_mut(),
            idx: self.idx,
            _marker: PhantomData,
        }
    }

    pub fn dormant(&self) -> Handle<NodeRefDormantMut<K, V, NodeType>, HandleType> {
        Handle {
            node: self.node.dormant(),
            idx: self.idx,
            _marker: PhantomData,
        }
    }
}

impl<K, V, NodeType, HandleType> Handle<NodeRefDormantMut<K, V, NodeType>, HandleType> {
    pub unsafe fn awaken<'a>(self) -> Handle<NodeRefMut<'a, K, V, NodeType>, HandleType> {
        Handle {
            node: self.node.awaken(),
            idx: self.idx,
            _marker: PhantomData,
        }
    }
}

impl<'a, K: 'a, V: 'a> HandleEdge<NodeRefMut<'a, K, V, marker::Leaf>> {
    unsafe fn insert_fit(mut self, key: K, val: V) -> HandleKV<NodeRefMut<'a, K, V, marker::Leaf>> {
        debug_assert!(self.node.len() < CAPACITY);
        let new_len = self.node.len() + 1;

        unsafe {
            slice_insert(self.node.key_area_mut(..new_len), self.idx, key);
            slice_insert(self.node.val_area_mut(..new_len), self.idx, val);
            *self.node.len_mut() = new_len as u8;

            Handle::new_kv(self.node, self.idx)
        }
    }
}

impl<'a, K: 'a, V: 'a> HandleEdge<NodeRefMut<'a, K, V, marker::Leaf>> {
    #[allow(clippy::type_complexity)]
    fn insert<A>(
        self,
        key: K,
        val: V,
        alloc: A,
    ) -> (
        Option<SplitResult<'a, K, V, marker::Leaf>>,
        HandleKV<NodeRefDormantMut<K, V, marker::Leaf>>,
    )
    where
        A: Allocator<LeafNode<K, V>>,
    {
        if self.node.len() < CAPACITY {
            let handle = unsafe { self.insert_fit(key, val) };
            (None, handle.dormant())
        } else {
            let (middle_kv_idx, insertion) = splitpoint(self.idx);
            let middle = unsafe { Handle::new_kv(self.node, middle_kv_idx) };
            let mut result = middle.split(alloc);
            let insertion_edge = match insertion {
                LeftOrRight::Left(insert_idx) => unsafe {
                    Handle::new_edge(result.left.reborrow_mut(), insert_idx)
                },
                LeftOrRight::Right(insert_idx) => unsafe {
                    Handle::new_edge(result.right.borrow_mut(), insert_idx)
                },
            };
            let handle = unsafe { insertion_edge.insert_fit(key, val).dormant() };
            (Some(result), handle)
        }
    }
}

impl<'a, K: 'a, V: 'a> HandleEdge<NodeRefMut<'a, K, V, marker::Internal>> {
    fn insert_fit(&mut self, key: K, val: V, edge: Root<K, V>) {
        debug_assert!(self.node.len() < CAPACITY);
        debug_assert!(edge.height == self.node.height - 1);
        let new_len = self.node.len() + 1;

        unsafe {
            slice_insert(self.node.key_area_mut(..new_len), self.idx, key);
            slice_insert(self.node.val_area_mut(..new_len), self.idx, val);
            slice_insert(
                self.node.edge_area_mut(..new_len + 1),
                self.idx + 1,
                edge.node,
            );
            *self.node.len_mut() = new_len as u8;
        }
    }

    fn insert<A>(
        mut self,
        key: K,
        val: V,
        edge: Root<K, V>,
        alloc: A,
    ) -> Option<SplitResult<'a, K, V, marker::Internal>>
    where
        A: Allocator<InternalNode<K, V>>,
    {
        assert!(edge.height == self.node.height - 1);

        if self.node.len() < CAPACITY {
            self.insert_fit(key, val, edge);
            None
        } else {
            let (middle_kv_idx, insertion) = splitpoint(self.idx);
            let middle = unsafe { Handle::new_kv(self.node, middle_kv_idx) };
            let mut result = middle.split(alloc);
            let mut insertion_edge = match insertion {
                LeftOrRight::Left(insert_idx) => unsafe {
                    Handle::new_edge(result.left.reborrow_mut(), insert_idx)
                },
                LeftOrRight::Right(insert_idx) => unsafe {
                    Handle::new_edge(result.right.borrow_mut(), insert_idx)
                },
            };
            insertion_edge.insert_fit(key, val, edge);
            Some(result)
        }
    }
}

impl<'a, K: 'a, V: 'a> HandleKV<NodeRefMut<'a, K, V, marker::Leaf>> {
    pub fn split<A>(mut self, alloc: A) -> SplitResult<'a, K, V, marker::Leaf>
    where
        A: Allocator<LeafNode<K, V>>,
    {
        let new_node = LeafNode::new(alloc);
        let kv = self.split_leaf_data(&mut *new_node);
        SplitResult {
            left: self.node,
            kv,
            right: NodeRef::from_new_leaf(new_node),
        }
    }

    /// ハンドルが指すキーと値をノードから削除してそれと左のエッジのハンドルを返す。
    #[allow(clippy::type_complexity)]
    pub fn remove(mut self) -> ((K, V), HandleEdge<NodeRefMut<'a, K, V, marker::Leaf>>) {
        let old_len = self.node.len();
        unsafe {
            let k = slice_remove(self.node.key_area_mut(..old_len), self.idx);
            let v = slice_remove(self.node.val_area_mut(..old_len), self.idx);
            *self.node.len_mut() = (old_len - 1) as u8;
            ((k, v), self.left_edge())
        }
    }
}

impl<'a, K: 'a, V: 'a> HandleKV<NodeRefMut<'a, K, V, marker::Internal>> {
    pub fn split<A>(mut self, alloc: A) -> SplitResult<'a, K, V, marker::Internal>
    where
        A: Allocator<InternalNode<K, V>>,
    {
        let old_len = self.node.len();
        unsafe {
            let new_node = InternalNode::new(alloc);
            let kv = self.split_leaf_data(&mut new_node.data);
            let new_len = new_node.data.len as usize;
            move_to_slice(
                self.node.edge_area_mut(self.idx + 1..old_len + 1),
                &mut new_node.edges[..new_len + 1],
            );

            let height = self.node.height;
            let right = NodeRef::from_new_internal(new_node, height);
            SplitResult {
                left: self.node,
                kv,
                right,
            }
        }
    }
}

impl<'a, K: 'a, V: 'a, NodeType> HandleKV<NodeRefMut<'a, K, V, NodeType>> {
    fn split_leaf_data(&mut self, new_node: &mut LeafNode<K, V>) -> (K, V) {
        debug_assert!(self.idx < self.node.len());
        let old_len = self.node.len();
        let new_len = old_len - self.idx - 1;
        new_node.len = new_len as u8;
        unsafe {
            let k = self.node.key_area_mut(self.idx).assume_init_read();
            let v = self.node.val_area_mut(self.idx).assume_init_read();

            move_to_slice(
                self.node.key_area_mut(self.idx + 1..old_len),
                &mut new_node.keys[..new_len],
            );
            move_to_slice(
                self.node.val_area_mut(self.idx + 1..old_len),
                &mut new_node.vals[..new_len],
            );

            *self.node.len_mut() = self.idx as u8;
            (k, v)
        }
    }
}

impl<BorrowType: marker::BorrowType, K, V> HandleEdge<InternalNodeRef<BorrowType, K, V>> {
    pub fn descend(self) -> LeafOrInternalNodeRef<BorrowType, K, V> {
        assert!(BorrowType::TRAVERSAL_PERMIT);

        let parent_ptr = NodeRef::as_internal_ptr(&self.node);
        let node = unsafe {
            (*parent_ptr)
                .edges
                .get_unchecked(self.idx)
                .assume_init_read()
        };
        NodeRef {
            node,
            height: self.node.height - 1,
            _marker: PhantomData,
        }
    }
}

impl<BorrowType, K, V> HandleEdge<LeafNodeRef<BorrowType, K, V>> {
    pub fn forget_node_type(self) -> HandleEdge<LeafOrInternalNodeRef<BorrowType, K, V>> {
        unsafe { Handle::new_edge(self.node.forget_type(), self.idx) }
    }
}

impl<BorrowType, K, V> HandleEdge<InternalNodeRef<BorrowType, K, V>> {
    pub fn forget_node_type(self) -> HandleEdge<LeafOrInternalNodeRef<BorrowType, K, V>> {
        unsafe { Handle::new_edge(self.node.forget_type(), self.idx) }
    }
}

impl<BorrowType, K, V> HandleKV<LeafNodeRef<BorrowType, K, V>> {
    pub fn forget_node_type(self) -> HandleKV<LeafOrInternalNodeRef<BorrowType, K, V>> {
        unsafe { Handle::new_kv(self.node.forget_type(), self.idx) }
    }
}

impl<BorrowType, K, V, Type> Handle<LeafOrInternalNodeRef<BorrowType, K, V>, Type> {
    pub fn force(self) -> ForceResultHandle<BorrowType, K, V, Type> {
        match self.node.force() {
            ForceResult::Leaf(node) => ForceResult::Leaf(Handle {
                node,
                idx: self.idx,
                _marker: PhantomData,
            }),
            ForceResult::Internal(node) => ForceResult::Internal(Handle {
                node,
                idx: self.idx,
                _marker: PhantomData,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::btree::alloc::GlobalAlloc;

    use super::*;

    #[test]
    fn insert_into_leaf() {
        let mut node = NodeRef::<_, usize, (), _>::new_leaf(GlobalAlloc);
        for i in 0..CAPACITY {
            let handle = node.borrow_mut().last_edge();
            let result = handle.insert(i, (), GlobalAlloc);
            assert!(result.0.is_none());
        }
        let result = node.borrow_mut().last_edge().insert(CAPACITY, (), GlobalAlloc);
        let mut split_result = result.0.unwrap();
        println!("middle: {:?}", split_result.kv.0);
        println!(
            "left-last: {:?}",
            split_result.left.last_kv().into_kv_mut().0
        );
        println!(
            "right-first: {:?}",
            split_result.right.borrow_mut().first_kv().into_kv_mut().0
        );
        let right = split_result.right;
        unsafe {
            node.into_dying().forget_type().deallocate(GlobalAlloc);
            right.into_dying().forget_type().deallocate(GlobalAlloc);
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
