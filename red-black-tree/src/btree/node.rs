use std::{alloc::{self, handle_alloc_error, Layout}, marker::PhantomData, mem::{size_of, MaybeUninit}, ptr::{self, NonNull}};

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
    height: u8,
    node: NonNull<LeafNode<K, V>>,
    _marker: PhantomData<(BorrowType, Type)>,
}

pub type Root<K, V> = NodeRef<marker::Owned, K, V, marker::LeafOrInternal>;

impl<K, V> LeafNode<K, V> {
    /// 新しい`LeafNode`をインプレースに初期化する。
    unsafe fn init(this: *mut Self) {
        ptr::addr_of_mut!((*this).len).write(0);
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

    pub enum Val {}
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
            NonNull::new(alloc::alloc(layout)).unwrap_or_else(|| handle_alloc_error(layout)).cast()
        }
    };
    unsafe { Box::from_raw(ptr.as_ptr()) }
}

unsafe fn assume_init_box<T>(val: Box<MaybeUninit<T>>) -> Box<T> {
    let ptr = Box::into_raw(val) as *mut T;
    unsafe { Box::from_raw(ptr) }
}
