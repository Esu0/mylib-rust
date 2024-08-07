pub mod operation;
use operation::{Idempotent, Operator};
use std::{
    cmp::Ordering,
    iter,
    ops::{Bound, Deref, DerefMut, RangeBounds},
};

#[derive(Debug, Clone)]
pub struct Segtree<T, OP> {
    len: usize,
    data: Box<[T]>,
    op: OP,
}

impl<T, OP> Segtree<T, OP> {
    fn new_empty(op: OP) -> Self {
        Self {
            len: 0,
            data: Box::new([]),
            op,
        }
    }

    pub fn into_boxed_slice(self) -> Box<[T]> {
        self.data
    }

    pub fn into_vec(self) -> Vec<T> {
        self.data.into_vec()
    }
}

impl<T, OP> Deref for Segtree<T, OP> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.data[self.len..]
    }
}

impl<T, OP: Operator<Query = T>> Segtree<T, OP> {
    fn eval(mut self) -> Self {
        for i in (1..self.len).rev() {
            self.data[i] = self.op.op(&self.data[i * 2], &self.data[i * 2 + 1]);
        }
        self
    }

    pub fn from_iter_op<I: IntoIterator<Item = T>>(iter: I, op: OP) -> Self {
        let iter = iter.into_iter();
        let (size_min, size_max) = iter.size_hint();
        if size_max == Some(0) {
            Self::new_empty(op)
        } else {
            assert_ne!(size_min, 0);
            let half_len_min = size_min.next_power_of_two();
            let half_len_max = size_max.map(usize::next_power_of_two);
            let uninit = if Some(half_len_min) == half_len_max {
                let half_len = half_len_min;
                let data = iter::repeat_with(|| OP::IDENT)
                    .take(half_len)
                    .chain(iter.chain(iter::repeat_with(|| OP::IDENT)).take(half_len))
                    .collect();

                Self {
                    len: half_len,
                    data,
                    op,
                }
            } else {
                let data = iter.collect::<Vec<_>>();
                let half_len = data.len().next_power_of_two();
                let data = iter::repeat_with(|| OP::IDENT)
                    .take(half_len)
                    .chain(
                        data.into_iter()
                            .chain(iter::repeat_with(|| OP::IDENT))
                            .take(half_len),
                    )
                    .collect();
                Self {
                    len: half_len,
                    data,
                    op,
                }
            };
            uninit.eval()
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 戻り値を`(l, r)`とすると以下が保証される。
    ///
    /// * `l <= r <= self.len()`
    fn get_lr<R: RangeBounds<usize>>(&self, range: R) -> (usize, usize) {
        use Bound::*;
        let size = self.len;
        let l = match range.start_bound() {
            Excluded(s) => s
                .checked_add(1)
                .unwrap_or_else(|| panic!("attempted to index slice from after maximum usize")),
            Included(s) => *s,
            Unbounded => 0,
        };
        let r = match range.end_bound() {
            Excluded(e) => *e,
            Included(e) => e
                .checked_add(1)
                .unwrap_or_else(|| panic!("attempted to index slice up to maximum usize")),
            Unbounded => size,
        };
        if l > r {
            panic!("slice index starts at {l} but ends at {r}");
        } else if r > size {
            panic!("range end index {r} out of range for slice of length {size}");
        }
        (l, r)
    }

    pub fn query<R: RangeBounds<usize>>(&self, range: R) -> T {
        let (mut l, mut r) = self.get_lr(range);
        l += self.len;
        r += self.len;
        let mut query_l = OP::IDENT;
        let mut query_r = OP::IDENT;
        while l < r {
            if r & 1 == 1 {
                r -= 1;
                self.op.op_assign_right(&self.data[r], &mut query_r);
            }
            if l & 1 == 1 {
                self.op.op_assign_left(&mut query_l, &self.data[l]);
                l += 1;
            }
            l >>= 1;
            r >>= 1;
        }
        self.op.op_assign_left(&mut query_l, &query_r);
        query_l
    }

    pub fn get_mut(&mut self, index: usize) -> ValMut<'_, T, OP> {
        assert!(index < self.len, "index out of bounds");
        ValMut {
            index: index + self.len,
            segtree: self,
        }
    }

    fn update_val(&mut self, mut i: usize) {
        while i > 1 {
            i >>= 1;
            self.data[i] = self.op.op(&self.data[i * 2], &self.data[i * 2 + 1]);
        }
    }

    pub fn update(&mut self, index: usize, value: T) {
        let i = index + self.len;
        self.data[i] = value;
        self.update_val(i);
    }

    /// `pred(self.query(l..j))`が`true`となる最大の`j`をO(log(n))で求める。
    pub fn upper_bound<P>(&self, l: usize, mut pred: P) -> usize
    where
        P: FnMut(&T) -> bool,
    {
        match l.cmp(&self.len()) {
            Ordering::Equal => return l,
            Ordering::Greater => {
                panic!("index {l} out of range for slice of length {}", self.len())
            }
            _ => {}
        };
        let stop = self.data.len() / ((self.len - l + 1).next_power_of_two() >> 1) - 1;
        let mut l = l + self.len;
        let mut l_query = OP::IDENT;
        loop {
            while l & 1 == 0 {
                l >>= 1;
            }
            let next_query = self.op.op(&l_query, &self.data[l]);
            if pred(&next_query) {
                l_query = next_query;
            } else {
                break;
            }
            if l == stop {
                return self.len;
            }
            l = (l >> 1) + 1;
        }
        while l < self.len {
            l <<= 1;
            let next_query = self.op.op(&l_query, &self.data[l]);
            if pred(&next_query) {
                l_query = next_query;
                l += 1;
            }
        }
        l - self.len
    }

    /// `pred(self.query(j..r))`が`true`となる最小の`j`をO(log(n))で求める。
    pub fn lower_bound<P>(&self, r: usize, mut pred: P) -> usize
    where
        P: FnMut(&T) -> bool,
    {
        if r > self.len {
            panic!("index {r} out of range for slice of length {}", self.len())
        }
        if r == 0 {
            return 0;
        }
        let stop = self.len >> r.ilog2();
        let mut r = r + self.len - 1;
        let mut r_query = OP::IDENT;
        loop {
            while r & 1 == 1 {
                r >>= 1;
            }
            let next_query = self.op.op(&self.data[r], &r_query);
            if pred(&next_query) {
                r_query = next_query;
            } else {
                break;
            }
            if r == stop {
                return 0;
            }
            r = (r >> 1) - 1;
        }
        while r < self.len {
            r = (r << 1) + 1;
            let next_query = self.op.op(&self.data[r], &r_query);
            if pred(&next_query) {
                r_query = next_query;
                r -= 1;
            }
        }
        r + 1 - self.len
    }
}

impl<T, OP: Idempotent<Query = T>> Segtree<T, OP> {
    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        self.data[1..].fill(value);
    }

    /// fは呼ばれるたびに同じ値を返す必要がある。そうでない場合、セグメント木の性質が壊れる。
    pub fn fill_with<F: FnMut() -> T>(&mut self, f: F) {
        self.data[1..].fill_with(f);
    }
}

pub struct ValMut<'a, T, OP: Operator<Query = T>> {
    segtree: &'a mut Segtree<T, OP>,
    index: usize,
}

impl<'a, T, OP: Operator<Query = T>> Deref for ValMut<'a, T, OP> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.segtree.data[self.index]
    }
}

impl<'a, T, OP: Operator<Query = T>> DerefMut for ValMut<'a, T, OP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.segtree.data[self.index]
    }
}

impl<'a, T, OP: Operator<Query = T>> Drop for ValMut<'a, T, OP> {
    fn drop(&mut self) {
        self.segtree.update_val(self.index);
    }
}

impl<I, OP> FromIterator<I> for Segtree<I, OP>
where
    OP: Default + Operator<Query = I>,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        Self::from_iter_op(iter, OP::default())
    }
}

impl<T, OP> From<Segtree<T, OP>> for Box<[T]> {
    fn from(value: Segtree<T, OP>) -> Self {
        value.into_boxed_slice()
    }
}

impl<T, OP> From<Segtree<T, OP>> for Vec<T> {
    fn from(value: Segtree<T, OP>) -> Self {
        value.into_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_test() {
        let segtree = Segtree::from_iter_op([1u32, 2, 3, 4, 5, 6], operation::min());
        assert_eq!(
            &segtree.data[1..],
            &[
                1,
                1,
                5,
                1,
                3,
                5,
                u32::MAX,
                1,
                2,
                3,
                4,
                5,
                6,
                u32::MAX,
                u32::MAX
            ]
        );
    }

    #[test]
    fn sum_query_test() {
        let segtree = [-4, 6, -3, 2, 1, 1, 7]
            .into_iter()
            .collect::<Segtree<_, operation::Add<_>>>();

        assert_eq!(segtree.query(..), 10);
        assert_eq!(segtree.query(3..), 11);
        assert_eq!(segtree.query(3..6), 4);
        assert_eq!(segtree.query(..3), -1);

        assert_eq!(segtree.query(0..1), -4);
        assert_eq!(segtree.query(0..=0), -4);
        assert_eq!(segtree.query(0..=1), 2);
        assert_eq!(segtree.query(0..0), 0);
        assert_eq!(segtree.query(1..1), 0);
        assert_eq!(segtree.query(7..7), 0);
        assert_eq!(segtree.query(6..8), 7);
    }

    #[test]
    fn min_query_test() {
        let segtree = [23i32, 12, -3, 0, 3, -2, 7, 8]
            .into_iter()
            .collect::<Segtree<_, operation::Min<_>>>();

        assert_eq!(segtree.query(..), -3);
        assert_eq!(segtree.query(3..), -2);
        assert_eq!(segtree.query(..2), 12);
        assert_eq!(segtree.query(3..5), 0);

        assert_eq!(segtree.query(0..1), 23);
        assert_eq!(segtree.query(0..=0), 23);
        assert_eq!(segtree.query(0..=1), 12);
        assert_eq!(segtree.query(0..0), i32::MAX);
        assert_eq!(segtree.query(1..1), i32::MAX);
        assert_eq!(segtree.query(7..7), i32::MAX);
        assert_eq!(segtree.query(7..8), 8);
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_test1() {
        let segtree = [1, 2, 3, 4, 5, 6, 7]
            .into_iter()
            .collect::<Segtree<_, operation::Add<_>>>();
        segtree.query(0..9);
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_test2() {
        let segtree = [1, 2, 3, 4, 5, 6, 7]
            .into_iter()
            .collect::<Segtree<_, operation::Add<_>>>();
        segtree.query(9..);
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_test3() {
        let segtree = [1, 2, 3, 4, 5, 6, 7]
            .into_iter()
            .collect::<Segtree<_, operation::Add<_>>>();
        segtree.query(0..=8);
    }

    #[test]
    #[should_panic]
    #[allow(clippy::reversed_empty_ranges)]
    fn out_of_bounds_test4() {
        let segtree = [1, 2, 3, 4, 5, 6, 7]
            .into_iter()
            .collect::<Segtree<_, operation::Add<_>>>();
        segtree.query(5..4);
    }

    #[test]
    fn update_test() {
        let mut segtree = [-4, 6, -3, 2, 1, 1, 7]
            .into_iter()
            .collect::<Segtree<_, operation::Add<_>>>();

        assert_eq!(segtree.query(..), 10);
        assert_eq!(segtree.query(3..), 11);
        assert_eq!(segtree.query(3..6), 4);
        assert_eq!(segtree.query(..3), -1);

        *segtree.get_mut(2) = 3;
        assert_eq!(segtree.query(..), 16);
        assert_eq!(segtree.query(3..), 11);
        assert_eq!(segtree.query(3..6), 4);
        assert_eq!(segtree.query(..3), 5);
        println!("{segtree:?}");
    }

    #[test]
    fn fill_test() {
        let mut segtree = [100, 200, 15, 40]
            .into_iter()
            .collect::<Segtree<_, operation::Min<_>>>();

        assert_eq!(segtree.query(..), 15);
        segtree.fill(10);
        assert_eq!(segtree.query(..), 10);
        assert_eq!(segtree.query(1..), 10);
        assert_eq!(segtree.query(1..3), 10);

        segtree.fill_with(|| -20);
        assert_eq!(segtree.query(..), -20);
        assert_eq!(segtree.query(1..), -20);
        assert_eq!(segtree.query(1..3), -20);
    }

    #[test]
    fn partition_point_test() {
        let segtree = [3u32, 5, 2, 1, 9, 11, 15, 3]
            .into_iter()
            .collect::<Segtree<_, operation::Add<_>>>();

        assert_eq!(segtree.upper_bound(0, |v| *v <= 20), 5);
        assert_eq!(segtree.upper_bound(1, |v| *v <= 20), 5);
        assert_eq!(segtree.upper_bound(4, |v| *v <= 25), 6);
        assert_eq!(segtree.upper_bound(3, |v| *v <= 100), 8);
        assert_eq!(segtree.upper_bound(8, |v| *v <= 20), 8);
    }

    #[test]
    fn max_query_test() {
        let mut segtree = [23i32, 12, -3, 0, 3, -2, 7, 8]
            .into_iter()
            .collect::<Segtree<_, operation::Max<_>>>();

        assert_eq!(segtree.query(..), 23);
        assert_eq!(segtree.query(1..), 12);
        assert_eq!(segtree.query(2..), 8);
        assert_eq!(segtree.query(1..6), 12);
        assert_eq!(segtree.query(2..6), 3);
        assert_eq!(segtree.query(2..=6), 7);

        segtree.update(2, 5);
        assert_eq!(segtree.query(..), 23);
        assert_eq!(segtree.query(2..), 8);
        assert_eq!(segtree.query(2..6), 5);
        assert_eq!(segtree.query(2..=6), 7);

        segtree.update(0, 10);
        eprintln!("{:?}", &segtree[..]);
        // assert_eq!(segtree.partition_point(0, |v| *v < 12), 1);
        // assert_eq!(segtree.partition_point(0, |v| *v < 13), 8);
        // assert_eq!(segtree.partition_point(1, |v| *v < 12), 1);
        assert_eq!(segtree.upper_bound(2, |v| *v < 12), 8);
        assert_eq!(segtree.upper_bound(2, |v| *v < 7), 6);
    }

    #[test]
    #[should_panic]
    fn partition_point_panic() {
        let segtree = [3u32, 5, 2, 1, 9, 11, 15, 3]
            .into_iter()
            .collect::<Segtree<_, operation::Add<_>>>();
        segtree.upper_bound(9, |v| *v <= 20);
    }
}
