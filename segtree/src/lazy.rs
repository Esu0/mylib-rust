use std::{iter, ops::RangeBounds};

use super::{
    get_lr,
    operation::{Map, Operator},
    Segtree,
};

pub struct LazySegtree<T, F, OP, M> {
    data: Box<[T]>,
    lazy: Box<[F]>,
    op: OP,
    map: M,
}

unsafe fn borrow_from_slice_two_mut<T>(slice: &mut [T], i: usize, j: usize) -> (&mut T, &mut T) {
    debug_assert_ne!(i, j);
    debug_assert!(i < slice.len());
    debug_assert!(j < slice.len());
    let ptr = slice.as_mut_ptr();
    (&mut *ptr.add(i), &mut *ptr.add(j))
}

impl<T, F, OP, M> LazySegtree<T, F, OP, M> {
    pub const fn len(&self) -> usize {
        self.lazy.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.lazy.is_empty()
    }
}

impl<T, F, OP: Operator<Query = T>, M: Map<OP = OP, Elem = F>> LazySegtree<T, F, OP, M> {
    pub fn from_iter_op<I>(iter: I, op: OP, map: M) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let Segtree { len, data, op } = Segtree::from_iter_op(iter, op);
        let lazy = iter::repeat_with(|| M::IDENT).take(len).collect();
        Self {
            data,
            lazy,
            op,
            map,
        }
    }

    fn push(&mut self, i: usize) -> F {
        let lazy = std::mem::replace(&mut self.lazy[i], M::IDENT);
        self.map.apply_assign(&mut self.data[i * 2], &lazy);
        self.map.apply_assign(&mut self.data[i * 2 + 1], &lazy);
        if i < self.len() / 2 {
            self.map.composite_assign(&mut self.lazy[i * 2], &lazy);
            self.map.composite_assign(&mut self.lazy[i * 2 + 1], &lazy);
        }
        lazy
    }

    fn update_all(&mut self) {
        let len_half = self.len().div_ceil(2);
        for i in 1..len_half {
            let (p, ch1) = unsafe { borrow_from_slice_two_mut(&mut self.lazy, i, i * 2) };
            self.map.composite_assign(ch1, p);
            let (p, ch2) = unsafe { borrow_from_slice_two_mut(&mut self.lazy, i, i * 2 + 1) };
            self.map.composite_assign(ch2, p);
            *p = M::IDENT;
        }
        for (i, lazy) in self.lazy.iter_mut().enumerate().skip(len_half) {
            self.map.apply_assign(&mut self.data[i * 2], lazy);
            self.map.apply_assign(&mut self.data[i * 2 + 1], lazy);
            *lazy = M::IDENT;
        }
        for i in (1..self.len()).rev() {
            let q = self.op.op(&self.data[i * 2], &self.data[i * 2 + 1]);
            self.data[i] = q;
        }
    }

    pub fn into_boxed_slice(mut self) -> Box<[T]> {
        self.update_all();
        self.data
    }

    pub fn into_vec(self) -> Vec<T> {
        self.into_boxed_slice().into_vec()
    }

    pub fn update_range<R: RangeBounds<usize>>(&mut self, range: R, m: M::Elem) {
        let (l_orig, r_orig) = get_lr(self.len(), range);
        if l_orig == r_orig {
            return;
        }
        let l = l_orig + self.len();
        let r = r_orig - 1 + self.len();
        let mut i = self.len().ilog2();
        while (l >> i) == (r >> i) {
            self.push(l >> i);
            i -= 1;
        }
        // for i in (0..r_orig.ilog2()).rev() {

        // }
        // if i == 0 {
        //     self.map.apply_assign(&mut self.data[l], &m);
        //     return;
        // }
        {
            let mut i = i;
            let t = l.trailing_zeros();
            if i > t {
                let l = l >> i;
                self.push(l);
                i -= 1;
            }
            while i > t {
                let l = l >> i;
                self.push(l);
                if l & 1 == 0 {
                    self.map.apply_assign(&mut self.data[l + 1], &m);
                    self.map.composite_assign(&mut self.lazy[l + 1], &m);
                }
                i -= 1;
            }
            self.map.apply_assign(&mut self.data[l >> i], &m);
            if i != 0 {
                self.map.composite_assign(&mut self.lazy[l >> i], &m);
            }
        }
        {
            let mut i = i;
            let t = r.trailing_ones();
            if i > t {
                let r = r >> i;
                self.push(r);
                i -= 1;
            }
            while i > t {
                let r = r >> i;
                self.push(r);
                if r & 1 != 0 {
                    self.map.composite_assign(&mut self.lazy[r - 1], &m);
                }
                i -= 1;
            }
            self.map.apply_assign(&mut self.data[r >> i], &m);
            if i != 0 {
                self.map.composite_assign(&mut self.lazy[r >> i], &m);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::operation;
    use super::*;

    #[test]
    fn build_test() {
        let segtree = LazySegtree::from_iter_op(0..14, operation::min(), operation::update());
        assert_eq!(segtree.len(), 16);
        assert_eq!(
            &segtree.data[1..],
            &[
                0,
                0,
                8,
                0,
                4,
                8,
                12,
                0,
                2,
                4,
                6,
                8,
                10,
                12,
                i32::MAX,
                0,
                1,
                2,
                3,
                4,
                5,
                6,
                7,
                8,
                9,
                10,
                11,
                12,
                13,
                i32::MAX,
                i32::MAX
            ]
        );
    }

    #[test]
    fn update_range_test() {
        let mut segtree = LazySegtree::from_iter_op(0..14, operation::min(), operation::range_add());
        segtree.update_range(3..7, 1);
        segtree.update_range(8..12, 1);
        segtree.update_range(6.., -2);
        segtree.update_all();
        {
            let mut i = 1;
            let mut j = 0;
            while i < segtree.len() {
                for _ in 0..(1 << j) {
                    eprint!("{:?} ", segtree.lazy[i]);
                    i += 1;
                }
                eprintln!();
                j += 1;
            }
        }
        {
            let mut i = 1;
            let mut j = 0;
            while i < segtree.data.len() {
                for _ in 0..(1 << j) {
                    eprint!("{:?} ", segtree.data[i]);
                    i += 1;
                }
                eprintln!();
                j += 1;
            }
        }
    }
}
